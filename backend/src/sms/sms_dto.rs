use serde::Deserialize;

#[derive(Deserialize)]
pub struct SendSmsRequest {
    pub to: String,
    pub body: String,
}

#[derive(Deserialize)]
pub struct TwilioWebhook {
    #[serde(rename = "From")]
    pub from: String,
    #[serde(rename = "Body")]
    pub body: String,
}

#[derive(serde::Serialize)]
pub struct ChatMessageResponse {
    pub id: i32,
    pub direction: String,
    pub body: Option<String>,
    pub status: Option<String>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
}

#[derive(serde::Serialize)]
pub struct ContactPreview {
    pub contact_number: String,
    pub last_message_body: Option<String>,
    pub last_message_date: chrono::DateTime<chrono::FixedOffset>,
    pub direction: String,
}

#[derive(serde::Serialize)]
pub struct ContactListResponse {
    pub contacts: Vec<ContactPreview>,
}

#[derive(serde::Serialize)]
pub struct ChatThreadResponse {
    pub contact: String,
    pub messages: Vec<ChatMessageResponse>,
}

#[derive(serde::Deserialize)]
pub struct PaginationQuery {
    pub page: Option<u64>,
}
