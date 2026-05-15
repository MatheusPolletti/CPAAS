use dotenv::dotenv;
use std::env;

pub struct Config {
    pub secret: String,
    pub database_url: String,
    pub port: u16,
    pub frontend_url: String,
    pub twilio_account_sid: String,
    pub twilio_auth_token: String,
    pub twilio_phone_number: String,
}

impl Config {
    pub fn from_env() -> Self {
        dotenv().ok();

        let port_str = env::var("PORT").unwrap_or_else(|_| "3000".to_string());

        Self {
            secret: env::var("SECRET").expect("SECRET não definida no .env"),
            database_url: env::var("DATABASE_URL").expect("Banco de dados não definido"),
            port: port_str.parse().expect("A porta deve ser válida"),
            frontend_url: env::var("FRONTEND_URL").expect("Erro ao pegar url do front"),
            twilio_account_sid: env::var("TWILIO_ACCOUNT_SID")
                .expect("Erro ao pegar dados da conta Twilio"),
            twilio_auth_token: env::var("TWILIO_AUTH_TOKEN")
                .expect("Erro ao pegar dados da conta Twilio"),
            twilio_phone_number: env::var("TWILIO_PHONE_NUMBER").expect("Erro ao pegar número"),
        }
    }
}
