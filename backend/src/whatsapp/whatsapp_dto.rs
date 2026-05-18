use serde::Deserialize;

#[derive(Deserialize)]
pub struct WhatsAppOutbound {
    pub to: String,
    pub message: String,
}

#[derive(Deserialize)]
pub struct WhatsAppInbound {
    #[serde(rename = "From")]
    pub from: String,
    #[serde(rename = "Body")]
    pub message: String,
    #[serde(rename = "ProfileName")]
    pub profile_name: Option<String>,
}
