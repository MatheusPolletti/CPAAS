use serde::Deserialize;

#[derive(Deserialize)]
pub struct CallOutbound {
    pub to: String,
}

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
pub struct TwilioCallResponse {
    pub sid: String,
    pub status: String,
}
