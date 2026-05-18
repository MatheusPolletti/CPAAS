use reqwest::Client;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection};

use crate::entities::whatsapp;

#[derive(Clone)]
pub struct WhatsappService {
    twilio_account_sid: String,
    twilio_auth_token: String,
    twilio_phone_number: String,
    db: DatabaseConnection,
}

impl WhatsappService {
    pub fn new(sid: String, auth: String, phone: String, db: DatabaseConnection) -> Self {
        Self {
            twilio_account_sid: sid,
            twilio_auth_token: auth,
            twilio_phone_number: phone,
            db,
        }
    }

    pub async fn send_whatsapp(&self, to: &String, message: &String) -> Result<(), String> {
        let client = Client::new();

        let url = format!(
            "https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json",
            self.twilio_account_sid
        );

        let to_whatsapp = format!("whatsapp:{}", to);
        let from_whatsapp = format!("whatsapp:{}", self.twilio_phone_number);

        let params = [
            ("To", to_whatsapp),
            ("From", from_whatsapp),
            ("Body", message.clone()),
        ];

        let res = client
            .post(&url)
            .basic_auth(&self.twilio_account_sid, Some(&self.twilio_auth_token))
            .form(&params)
            .send()
            .await
            .map_err(|e| format!("Erro de rede: {}", e))?;

        let is_success = res.status().is_success();
        let status_string = if is_success { "sent" } else { "failed" };

        let whatsapp_log = whatsapp::ActiveModel {
            direction: Set("outbound".to_string()),
            from_number: Set(self.twilio_phone_number.clone()),
            to_number: Set(to.clone()),
            body: Set(Some(message.clone())),
            status: Set(status_string.to_string()),
            user_id: Set(None),
            ..Default::default()
        };

        whatsapp_log
            .insert(&self.db)
            .await
            .map_err(|e| format!("Erro ao salvar log do WhatsApp: {:?}", e))?;

        if is_success {
            Ok(())
        } else {
            let error_text = res.text().await.unwrap_or_default();
            Err(format!("Erro da API Twilio: {}", error_text))
        }
    }

    pub async fn handle_receive(
        &self,
        from: &str,
        message: &str,
        sender_name: &Option<String>,
    ) -> Result<(), String> {
        println!("WhatsApp recebido de {:?}: {}", sender_name, message);

        let incoming_log = whatsapp::ActiveModel {
            direction: Set("inbound".to_string()),
            from_number: Set(from.to_string()),
            to_number: Set(self.twilio_phone_number.clone()),
            body: Set(Some(message.to_string())),
            status: Set("received".to_string()),

            sender_name: Set(sender_name.clone()),

            user_id: Set(None),
            ..Default::default()
        };

        incoming_log
            .insert(&self.db)
            .await
            .map_err(|e| format!("Erro ao salvar no banco: {:?}", e))?;

        Ok(())
    }
}
