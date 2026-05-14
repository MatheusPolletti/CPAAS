use dotenv::dotenv;
use std::env;

pub struct Config {
    pub secret: String,
    pub database_url: String,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Self {
        dotenv().ok();

        let port_str = env::var("PORT").unwrap_or_else(|_| "3000".to_string());

        Self {
            secret: env::var("SECRET").expect("SECRET não definida no .env"),
            database_url: env::var("DATABASE_URL").expect("Banco de dados não definido"),
            port: port_str.parse().expect("A porta deve ser válida"),
        }
    }
}
