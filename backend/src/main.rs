mod auth;
mod config;
mod database_service;
mod sms;

pub mod entities;

use std::sync::Arc;

use axum::{Router, extract::FromRef, http::HeaderValue};
use config::Config;
use tokio::net::TcpListener;

use crate::{
    auth::{auth_controller, auth_service::AuthService},
    database_service::connect_db,
    sms::sms_service::SmsService,
};
use axum::http::{Method, header};
use tower_http::cors::CorsLayer;

#[derive(Clone)]
pub struct AppState {
    pub auth_service: Arc<AuthService>,
    pub sms_service: Arc<SmsService>,
}

impl FromRef<AppState> for Arc<AuthService> {
    fn from_ref(state: &AppState) -> Self {
        state.auth_service.clone()
    }
}

impl FromRef<AppState> for Arc<SmsService> {
    fn from_ref(state: &AppState) -> Self {
        state.sms_service.clone()
    }
}

#[tokio::main]
async fn main() {
    let config = Config::from_env();

    let _connection = connect_db(&config.database_url).await;

    let auth_service = AuthService::new(config.secret, _connection.clone());

    let sms_service = SmsService::new(
        config.twilio_account_sid.clone(),
        config.twilio_auth_token.clone(),
        config.twilio_phone_number.clone(),
        _connection,
    );

    let app_state = AppState {
        auth_service: Arc::new(auth_service),
        sms_service: Arc::new(sms_service),
    };

    let frontend_origin = config
        .frontend_url
        .parse::<HeaderValue>()
        .expect("URL do frontend inválida");

    let cors = CorsLayer::new()
        .allow_origin(frontend_origin)
        .allow_methods([Method::GET, Method::POST, Method::PUT])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);

    let app = Router::new()
        .nest("/auth", auth_controller::router())
        .nest("/sms", crate::sms::sms_controller::router())
        .with_state(app_state)
        .layer(cors);

    let address = format!("0.0.0.0:{}", config.port);

    let listener = TcpListener::bind(&address).await.unwrap();

    println!("Rodando na porta 5000");

    axum::serve(listener, app).await.unwrap();
}
