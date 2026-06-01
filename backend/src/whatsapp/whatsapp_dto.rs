use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct WhatsAppInbound {
    #[serde(rename = "From")]
    pub from: String,
    #[serde(rename = "Body")]
    pub message: String,
    #[serde(rename = "ProfileName")]
    pub profile_name: Option<String>,
    #[serde(rename = "MessageSid")]
    pub message_sid: String,

    #[serde(rename = "MediaUrl0")]
    pub media_url: Option<String>,
    #[serde(rename = "MediaContentType0")]
    pub media_type: Option<String>,
}

#[derive(Deserialize)]
pub struct WhatsAppStatusWebhook {
    #[serde(rename = "MessageSid")]
    pub message_sid: String,
    #[serde(rename = "MessageStatus")]
    pub status: String,
}

#[derive(Serialize)]
pub struct WhatsappChatMessageResponse {
    pub id: i32,
    pub direction: String,
    pub body: Option<String>,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,

    pub media_url: Option<String>,
    pub media_type: Option<String>,
}

#[derive(Serialize)]
pub struct WhatsappChatThreadResponse {
    pub contact: String,
    pub messages: Vec<WhatsappChatMessageResponse>,
}

#[derive(Deserialize)]
pub struct MediaQuery {
    pub url: String,
}

#[derive(Serialize, Deserialize)]
pub struct TicketPreview {
    pub ticket_id: i32,
    pub contact_number: String,
    pub profile_name: Option<String>,
    pub last_message_body: Option<String>,
    pub last_message_date: String,
    pub status: String,
}

#[derive(Serialize, Deserialize)]
pub struct TicketListResponse {
    pub tickets: Vec<TicketPreview>,
}
