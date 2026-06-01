use reqwest::Client;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, Condition, DatabaseConnection, EntityTrait,
    IntoActiveModel, QueryFilter, QueryOrder, QuerySelect,
};

use crate::{entities::whatsapp, whatsapp::whatsapp_dto::TicketPreview};

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
        ticket_id: Option<i32>,
        sender_name: &Option<String>,
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
            params.push(("Body", message.to_string()));
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
                sender_name: Set(sender_name.clone().or(Some("Atendente".to_string()))),
                twilio_sid: Set(twilio_sid),
                ticket_id: Set(ticket_id),
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
        use crate::entities::{contacts, tickets, whatsapp};

        let clean_from = from.strip_prefix("whatsapp:").unwrap_or(from);

        let contact = contacts::Entity::find()
            .filter(contacts::Column::PhoneNumber.eq(clean_from))
            .one(&self.db)
            .await
            .map_err(|e| format!("Erro ao buscar contato: {:?}", e))?;

        let contact_id = match contact {
            Some(c) => c.id,
            None => {
                // Se o número não existe, criamos um contato automaticamente
                // Usamos o nome do perfil do WhatsApp, ou "Desconhecido" se vier vazio
                let fallback_name = sender_name
                    .clone()
                    .unwrap_or_else(|| "Desconhecido".to_string());

                let new_contact = contacts::ActiveModel {
                    phone_number: Set(clean_from.to_string()),
                    name: Set(fallback_name),
                    ..Default::default()
                };

                let res = new_contact
                    .insert(&self.db)
                    .await
                    .map_err(|e| format!("Erro ao criar contato automático: {:?}", e))?;
                res.id
            }
        };

        // Buscamos se esse cliente já tem um ticket com status "open"
        let open_ticket = tickets::Entity::find()
            .filter(tickets::Column::ContactId.eq(contact_id))
            .filter(tickets::Column::Status.eq("open"))
            .one(&self.db)
            .await
            .map_err(|e| format!("Erro ao buscar ticket: {:?}", e))?;

        let ticket_id = match open_ticket {
            Some(t) => t.id,
            None => {
                // Se não tem ticket aberto, abrimos um novo protocolo!
                let new_ticket = tickets::ActiveModel {
                    contact_id: Set(contact_id),
                    status: Set("open".to_string()),
                    subject: Set(Some("Atendimento via WhatsApp".to_string())),
                    ..Default::default()
                };

                let res = new_ticket
                    .insert(&self.db)
                    .await
                    .map_err(|e| format!("Erro ao abrir novo ticket: {:?}", e))?;
                res.id
            }
        };

        // --- PASSO 3: Salvar a mensagem amarrada a tudo ---
        let incoming_log = whatsapp::ActiveModel {
            direction: Set("inbound".to_string()),
            from_number: Set(clean_from.to_string()),
            to_number: Set(self.twilio_phone_number.clone()),
            body: Set(Some(message.to_string())),
            status: Set("received".to_string()),
            sender_name: Set(sender_name.clone()),
            twilio_sid: Set(Some(message_sid.to_string())),
            user_id: Set(None),
            contact_id: Set(Some(contact_id)),
            ticket_id: Set(Some(ticket_id)),

            media_url: Set(media_url.map(|s| s.to_string())),
            media_type: Set(media_type.map(|s| s.to_string())),
            ..Default::default()
        };

        incoming_log
            .insert(&self.db)
            .await
            .map_err(|e| format!("Erro ao salvar mensagem no banco: {:?}", e))?;

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

    pub async fn get_active_tickets(&self) -> Result<Vec<TicketPreview>, String> {
        use crate::entities::{contacts, tickets, whatsapp};

        let condition = Condition::any()
            .add(tickets::Column::Status.eq("open"))
            .add(tickets::Column::Status.eq("pending"));

        let active_tickets = tickets::Entity::find()
            .filter(condition)
            .order_by_desc(tickets::Column::UpdatedAt)
            .all(&self.db)
            .await
            .map_err(|e| format!("Erro ao buscar tickets: {:?}", e))?;

        let mut inbox: Vec<TicketPreview> = Vec::new();

        for ticket in active_tickets {
            let contact = contacts::Entity::find_by_id(ticket.contact_id)
                .one(&self.db)
                .await
                .map_err(|e| format!("Erro ao buscar contato: {:?}", e))?;

            let contact_data = match contact {
                Some(c) => c,
                None => continue,
            };

            let last_msg = whatsapp::Entity::find()
                .filter(whatsapp::Column::TicketId.eq(ticket.id))
                .order_by_desc(whatsapp::Column::CreatedAt)
                .one(&self.db)
                .await
                .map_err(|e| format!("Erro ao buscar mensagens do ticket: {:?}", e))?;

            // Resolvemos o que mostrar no preview (texto ou aviso de mídia)
            let (body, date) = match last_msg {
                Some(msg) => {
                    let text = msg
                        .body
                        .unwrap_or_else(|| "Mídia recebida/enviada".to_string());
                    (Some(text), msg.created_at.to_string())
                }
                None => (None, ticket.created_at.to_string()),
            };

            inbox.push(TicketPreview {
                ticket_id: ticket.id,
                contact_number: contact_data.phone_number,
                profile_name: Some(contact_data.name),
                last_message_body: body,
                last_message_date: date,
                status: ticket.status,
            });
        }

        inbox.sort_by(|a, b| b.last_message_date.cmp(&a.last_message_date));

        Ok(inbox)
    }

    pub async fn get_chat_thread(
        &self,
        ticket_id: i32,
        page: u64,
    ) -> Result<Vec<whatsapp::Model>, String> {
        let page_size = 10;

        let mut messages = whatsapp::Entity::find()
            // 👇 Filtramos de forma cirúrgica: apenas mensagens DESTE chamado!
            .filter(whatsapp::Column::TicketId.eq(ticket_id))
            .order_by_desc(whatsapp::Column::CreatedAt)
            .limit(page_size)
            .offset(page * page_size)
            .all(&self.db)
            .await
            .map_err(|e| format!("Erro no banco ao buscar histórico do ticket: {:?}", e))?;

        messages.reverse();

        Ok(messages)
    }
}
