use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct VoiceInbound {
    #[serde(rename = "From")]
    pub from: String,

    #[serde(rename = "To")]
    pub to: String,

    #[serde(rename = "CallSid")]
    pub call_sid: String,
}

#[derive(Deserialize)]
pub struct VoiceConnectRequest {
    #[serde(rename = "To")]
    pub to: String,
    #[serde(rename = "From")]
    pub from: Option<String>,
    #[serde(rename = "CallSid")]
    pub call_sid: String,
}

#[derive(Serialize)]
pub struct VoiceTokenResponse {
    pub token: String,
}

#[derive(Deserialize)]
pub struct CallStatusWebhook {
    #[serde(rename = "CallSid")]
    pub call_sid: String,

    #[serde(rename = "ParentCallSid")]
    pub parent_call_sid: Option<String>,

    #[serde(rename = "CallStatus")]
    pub call_status: String,
}

#[derive(Serialize)]
pub struct CallHistoryResponse {
    pub id: i32,
    pub call_sid: String,
    pub from_number: String,
    pub to_number: String,
    pub direction: String,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
}
