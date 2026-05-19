use reqwest::Client;
use sea_orm::DatabaseConnection;

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

    pub async fn call(&self, to: &String) -> Result<(), String> {
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
            Ok(())
        } else {
            let error_text = res.text().await.unwrap_or_default();
            Err(format!("Erro da API Twilio Voice: {}", error_text))
        }
    }
}
