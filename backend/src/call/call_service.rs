use reqwest::Client;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection};

use crate::call::call_dto::TwilioCallResponse;
use crate::entities::{calls, calls::Entity as CallEntity};
use sea_orm::{EntityTrait, QueryOrder};

#[derive(Clone)]
pub struct CallService {
    twilio_account_sid: String,
    twilio_auth_token: String,
    twilio_phone_number: String,
    db: DatabaseConnection,
}

impl CallService {
    pub fn new(sid: String, auth: String, phone: String, db: DatabaseConnection) -> Self {
        Self {
            twilio_account_sid: sid,
            twilio_auth_token: auth,
            twilio_phone_number: phone,
            db,
        }
    }

    pub async fn call(&self, to: &String, user_id: i32) -> Result<(), String> {
        let client = Client::new();

        let url = format!(
            "https://api.twilio.com/2010-04-01/Accounts/{}/Calls.json",
            self.twilio_account_sid
        );

        let meu_ngrok_url = "https://dinghy-drainable-headstand.ngrok-free.dev/call/twiml";

        let params = [
            ("To", to.as_str()),
            ("From", self.twilio_phone_number.as_str()),
            ("Url", meu_ngrok_url),
        ];

        let res = client
            .post(&url)
            .basic_auth(&self.twilio_account_sid, Some(&self.twilio_auth_token))
            .form(&params)
            .send()
            .await
            .map_err(|e| format!("Erro de rede: {}", e))?;

        if res.status().is_success() {
            let twilio_resp: TwilioCallResponse = res
                .json()
                .await
                .map_err(|_| "Erro ao ler resposta da Twilio".to_string())?;

            let new_call = calls::ActiveModel {
                call_sid: Set(twilio_resp.sid),
                from_number: Set(self.twilio_phone_number.clone()),
                to_number: Set(to.clone()),
                direction: Set("outbound".to_string()),
                status: Set(twilio_resp.status),
                user_id: Set(Some(user_id)),
                ..Default::default()
            };

            // 3. Inserimos no banco
            new_call
                .insert(&self.db)
                .await
                .map_err(|e| format!("Erro ao salvar no banco: {}", e))?;

            Ok(())
        } else {
            let error_text = res.text().await.unwrap_or_default();
            Err(format!("Erro da API Twilio Voice: {}", error_text))
        }
    }

    pub async fn register_inbound(&self, sid: &str, from: &str, to: &str) -> Result<(), String> {
        let new_call = calls::ActiveModel {
            call_sid: Set(sid.to_string()),
            from_number: Set(from.to_string()),
            to_number: Set(to.to_string()),
            direction: Set("inbound".to_string()),
            status: Set("in-progress".to_string()),
            ..Default::default()
        };

        new_call
            .insert(&self.db)
            .await
            .map_err(|e| format!("Erro ao salvar inbound no banco: {}", e))?;

        Ok(())
    }

    pub async fn get_call_history(&self) -> Result<Vec<calls::Model>, String> {
        CallEntity::find()
            .order_by_desc(calls::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(|e| format!("Erro ao buscar o histórico de ligações: {}", e))
    }
}
