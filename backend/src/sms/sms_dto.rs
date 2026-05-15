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

// A representação de uma mensagem individual na linha do tempo
#[derive(serde::Serialize)]
pub struct ChatMessageResponse {
    pub id: i32,
    pub direction: String,
    pub body: Option<String>,
    pub status: Option<String>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
}

// A resposta para a rota que lista os contatos únicos
#[derive(serde::Serialize)]
pub struct ContactListResponse {
    pub contacts: Vec<String>,
}

// A resposta para a rota que abre a conversa com alguém
#[derive(serde::Serialize)]
pub struct ChatThreadResponse {
    pub contact: String,
    pub messages: Vec<ChatMessageResponse>,
}
