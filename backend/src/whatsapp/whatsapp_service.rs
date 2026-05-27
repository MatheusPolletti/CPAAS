use std::collections::HashSet;

use reqwest::Client;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, Condition, DatabaseConnection, EntityTrait,
    IntoActiveModel, QueryFilter, QueryOrder, QuerySelect,
};

use crate::{entities::whatsapp, whatsapp::whatsapp_dto::WhatsappContactPreview};

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

    pub async fn fetch_media(&self, media_url: &str) -> Result<(Vec<u8>, String), String> {
        if !media_url.starts_with("https://api.twilio.com/") {
            return Err("URL de mídia não autorizada".to_string());
        }

        let client = Client::new();

        let res = client
            .get(media_url)
            .basic_auth(&self.twilio_account_sid, Some(&self.twilio_auth_token))
            .send()
            .await
            .map_err(|e| format!("Erro de rede ao buscar mídia: {}", e))?;

        let content_type = res
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_string();

        if res.status().is_success() {
            let bytes = res
                .bytes()
                .await
                .map_err(|e| format!("Erro ao ler bytes da mídia: {}", e))?;

            Ok((bytes.to_vec(), content_type))
        } else {
            Err("Falha ao autorizar download na Twilio".to_string())
        }
    }

    pub async fn send_whatsapp(
        &self,
        to: &String,
        message: &String,
        media_url: Option<String>,
        media_type: Option<String>,
    ) -> Result<(), String> {
        let client = Client::new();

        let url = format!(
            "https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json",
            self.twilio_account_sid
        );

        let to_whatsapp = format!("whatsapp:{}", to);
        let from_whatsapp = format!("whatsapp:{}", self.twilio_phone_number);

        let mut params = vec![("To", to_whatsapp), ("From", from_whatsapp)];

        if !message.trim().is_empty() {
            params.push(("Body", message.clone()));
        }

        if let Some(ref link) = media_url {
            params.push(("MediaUrl", link.clone()));
        }

        let res = client
            .post(&url)
            .basic_auth(&self.twilio_account_sid, Some(&self.twilio_auth_token))
            .form(&params)
            .send()
            .await
            .map_err(|e| format!("Erro de rede: {}", e))?;

        let status_code = res.status();
        let response_text = res.text().await.unwrap_or_default();

        if status_code.is_success() {
            let parsed: serde_json::Value =
                serde_json::from_str(&response_text).unwrap_or_default();

            let twilio_sid = parsed["sid"].as_str().map(|s| s.to_string());

            let whatsapp_log = whatsapp::ActiveModel {
                direction: Set("outbound".to_string()),
                from_number: Set(self.twilio_phone_number.clone()),
                to_number: Set(to.clone()),
                body: Set(Some(message.clone())),
                status: Set("queued".to_string()),
                sender_name: Set(Some("Você".to_string())),
                twilio_sid: Set(twilio_sid),
                user_id: Set(None),
                media_url: Set(media_url),
                media_type: Set(media_type),
                ..Default::default()
            };

            whatsapp_log
                .insert(&self.db)
                .await
                .map_err(|e| format!("Erro ao salvar log do WhatsApp: {:?}", e))?;

            Ok(())
        } else {
            Err(format!("Erro da API Twilio: {}", response_text))
        }
    }

    pub async fn handle_receive(
        &self,
        from: &str,
        message: &str,
        sender_name: &Option<String>,
        message_sid: &str,

        media_url: Option<&str>,
        media_type: Option<&str>,
    ) -> Result<(), String> {
        let clean_from = from.strip_prefix("whatsapp:").unwrap_or(from);

        let incoming_log = whatsapp::ActiveModel {
            direction: Set("inbound".to_string()),
            from_number: Set(clean_from.to_string()),
            to_number: Set(self.twilio_phone_number.clone()),
            body: Set(Some(message.to_string())),
            status: Set("received".to_string()),
            sender_name: Set(sender_name.clone()),
            twilio_sid: Set(Some(message_sid.to_string())),
            user_id: Set(None),

            media_url: Set(media_url.map(|s| s.to_string())),
            media_type: Set(media_type.map(|s| s.to_string())),
            ..Default::default()
        };

        incoming_log
            .insert(&self.db)
            .await
            .map_err(|e| format!("Erro ao salvar no banco: {:?}", e))?;

        Ok(())
    }

    pub async fn update_message_status(&self, sid: &String, status: &String) -> Result<(), String> {
        let message_option = whatsapp::Entity::find()
            .filter(whatsapp::Column::TwilioSid.eq(sid))
            .one(&self.db)
            .await
            .map_err(|e| format!("Erro ao buscar mensagem no banco: {:?}", e))?;

        if let Some(message) = message_option {
            let mut active_msg = message.into_active_model();

            active_msg.status = Set(status.clone());

            active_msg
                .update(&self.db)
                .await
                .map_err(|e| format!("Erro ao dar update no status: {:?}", e))?;
        }

        Ok(())
    }

    pub async fn get_unique_contacts(&self) -> Result<Vec<WhatsappContactPreview>, String> {
        let outbound_contacts: Vec<String> = whatsapp::Entity::find()
            .filter(whatsapp::Column::Direction.eq("outbound"))
            .select_only()
            .column(whatsapp::Column::ToNumber)
            .into_tuple()
            .all(&self.db)
            .await
            .map_err(|e| format!("Erro no banco: {:?}", e))?;

        let inbound_contacts: Vec<String> = whatsapp::Entity::find()
            .filter(whatsapp::Column::Direction.eq("inbound"))
            .select_only()
            .column(whatsapp::Column::FromNumber)
            .into_tuple()
            .all(&self.db)
            .await
            .map_err(|e| format!("Erro no banco: {:?}", e))?;

        let mut unique_numbers = HashSet::new();
        unique_numbers.extend(outbound_contacts);
        unique_numbers.extend(inbound_contacts);

        let mut inbox: Vec<WhatsappContactPreview> = Vec::new();

        for number in unique_numbers {
            let condition = Condition::any()
                .add(whatsapp::Column::FromNumber.eq(&number))
                .add(whatsapp::Column::ToNumber.eq(&number));

            let last_msg_option = whatsapp::Entity::find()
                .filter(condition)
                .order_by_desc(whatsapp::Column::CreatedAt)
                .one(&self.db)
                .await
                .map_err(|e| format!("Erro no banco: {:?}", e))?;

            if let Some(msg) = last_msg_option {
                let resolved_profile_name = if msg.direction == "inbound" {
                    msg.sender_name
                } else {
                    let last_inbound = whatsapp::Entity::find()
                        .filter(whatsapp::Column::FromNumber.eq(&number))
                        .filter(whatsapp::Column::Direction.eq("inbound"))
                        .filter(whatsapp::Column::SenderName.is_not_null())
                        .order_by_desc(whatsapp::Column::CreatedAt)
                        .one(&self.db)
                        .await
                        .unwrap_or(None);

                    last_inbound.and_then(|m| m.sender_name)
                };

                inbox.push(WhatsappContactPreview {
                    contact_number: number,
                    profile_name: resolved_profile_name,
                    last_message_body: msg.body,
                    last_message_date: msg.created_at,
                    direction: msg.direction,
                    status: msg.status,
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
    ) -> Result<Vec<whatsapp::Model>, String> {
        let condition = Condition::any()
            .add(whatsapp::Column::FromNumber.eq(contact_number))
            .add(whatsapp::Column::ToNumber.eq(contact_number));

        let page_size = 10;

        let mut messages = whatsapp::Entity::find()
            .filter(condition)
            .order_by_desc(whatsapp::Column::CreatedAt)
            .limit(page_size)
            .offset(page * page_size)
            .all(&self.db)
            .await
            .map_err(|e| format!("Erro no banco: {:?}", e))?;

        messages.reverse();

        Ok(messages)
    }
}
