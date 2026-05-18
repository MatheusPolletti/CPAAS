use crate::entities::sms;
use crate::sms::sms_dto::ContactPreview;
use reqwest::Client;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection};
use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use std::collections::HashSet;

#[derive(Clone)]
pub struct SmsService {
    twilio_account_sid: String,
    twilio_auth_token: String,
    twilio_phone_number: String,
    db: DatabaseConnection,
}

impl SmsService {
    pub fn new(sid: String, auth: String, phone: String, db: DatabaseConnection) -> Self {
        Self {
            twilio_account_sid: sid,
            twilio_auth_token: auth,
            twilio_phone_number: phone,
            db,
        }
    }

    pub async fn send_sms(&self, to: &str, body: &str) -> Result<(), String> {
        let client = Client::new();

        let url = format!(
            "https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json",
            self.twilio_account_sid
        );

        let params = [
            ("To", to),
            ("From", self.twilio_phone_number.as_str()),
            ("Body", body),
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

        let sms_log = sms::ActiveModel {
            direction: Set("outbound".to_string()),
            from_number: Set(self.twilio_phone_number.clone()),
            to_number: Set(to.to_string()),
            body: Set(Some(body.to_string())),
            status: Set(Some(status_string.to_string())),
            user_id: Set(None),
            ..Default::default()
        };

        sms_log
            .insert(&self.db)
            .await
            .map_err(|e| format!("Erro ao salvar log: {:?}", e))?;

        if is_success {
            println!("✅ SMS enviado com sucesso para {}", to);
            Ok(())
        } else {
            let error_text = res.text().await.unwrap_or_default();
            Err(format!("Erro da API Twilio: {}", error_text))
        }
    }

    pub async fn save_incoming_sms(&self, from: &str, body: &str) -> Result<(), String> {
        println!("Recebido {}", body);

        let incoming_log = sms::ActiveModel {
            direction: Set("inbound".to_string()),
            from_number: Set(from.to_string()),
            to_number: Set(self.twilio_phone_number.clone()),
            body: Set(Some(body.to_string())),
            status: Set(Some("received".to_string())),
            user_id: Set(None),
            ..Default::default()
        };

        incoming_log
            .insert(&self.db)
            .await
            .map_err(|e| format!("Erro ao salvar webhook: {:?}", e))?;

        Ok(())
    }

    pub async fn get_unique_contacts(&self) -> Result<Vec<ContactPreview>, String> {
        let outbound_contacts: Vec<String> = sms::Entity::find()
            .filter(sms::Column::Direction.eq("outbound"))
            .select_only()
            .column(sms::Column::ToNumber)
            .into_tuple()
            .all(&self.db)
            .await
            .map_err(|e| format!("Erro no banco: {:?}", e))?;

        let inbound_contacts: Vec<String> = sms::Entity::find()
            .filter(sms::Column::Direction.eq("inbound"))
            .select_only()
            .column(sms::Column::FromNumber)
            .into_tuple()
            .all(&self.db)
            .await
            .map_err(|e| format!("Erro no banco: {:?}", e))?;

        let mut unique_numbers = HashSet::new();
        unique_numbers.extend(outbound_contacts);
        unique_numbers.extend(inbound_contacts);

        let mut inbox: Vec<ContactPreview> = Vec::new();

        for number in unique_numbers {
            let condition = Condition::any()
                .add(sms::Column::FromNumber.eq(&number))
                .add(sms::Column::ToNumber.eq(&number));

            let last_msg_option = sms::Entity::find()
                .filter(condition)
                .order_by_desc(sms::Column::CreatedAt)
                .one(&self.db)
                .await
                .map_err(|e| format!("Erro no banco: {:?}", e))?;

            if let Some(msg) = last_msg_option {
                inbox.push(ContactPreview {
                    contact_number: number,
                    last_message_body: msg.body,
                    last_message_date: msg.created_at,
                    direction: msg.direction,
                });
            }
        }

        inbox.sort_by(|a, b| b.last_message_date.cmp(&a.last_message_date));

        Ok(inbox)
    }

    pub async fn get_chat_thread(
        &self,
        contact_number: &str,
        page: u64,
    ) -> Result<Vec<sms::Model>, String> {
        let condition = Condition::any()
            .add(sms::Column::FromNumber.eq(contact_number))
            .add(sms::Column::ToNumber.eq(contact_number));

        let page_size = 10;

        let mut messages = sms::Entity::find()
            .filter(condition)
            .order_by_desc(sms::Column::CreatedAt)
            .limit(page_size)
            .offset(page * page_size)
            .all(&self.db)
            .await
            .map_err(|e| format!("Erro no banco: {:?}", e))?;

        messages.reverse();

        Ok(messages)
    }
}
