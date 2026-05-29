mod auth;
mod call;
mod config;
mod contact;
mod database_service;
mod dto;
mod sms;
mod whatsapp;

pub mod entities;

use std::sync::Arc;

use axum::{Router, extract::FromRef, http::HeaderValue};
use config::Config;
use tokio::net::TcpListener;

use crate::{
    auth::{auth_controller, auth_service::AuthService},
    call::{call_controller, call_service::CallService},
    contact::{contact_controller, contact_service::ContactService},
    database_service::connect_db,
    sms::{sms_controller, sms_service::SmsService},
    whatsapp::{whatsapp_controller, whatsapp_service::WhatsappService},
};
use axum::http::{Method, header};
use tower_http::{cors::CorsLayer, services::ServeDir};

#[derive(Clone)]
pub struct AppState {
    pub auth_service: Arc<AuthService>,
    pub sms_service: Arc<SmsService>,
    pub whatsapp_service: Arc<WhatsappService>,
    pub call_service: Arc<CallService>,
    pub contact_service: Arc<ContactService>,
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

impl FromRef<AppState> for Arc<WhatsappService> {
    fn from_ref(state: &AppState) -> Self {
        state.whatsapp_service.clone()
    }
}

impl FromRef<AppState> for Arc<ContactService> {
    fn from_ref(state: &AppState) -> Self {
        state.contact_service.clone()
    }
}

impl FromRef<AppState> for Arc<CallService> {
    fn from_ref(state: &AppState) -> Self {
        state.call_service.clone()
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
        _connection.clone(),
    );

    let whatsapp_service = WhatsappService::new(
        config.twilio_account_sid.clone(),
        config.twilio_auth_token.clone(),
        config.twilio_whatsapp_number.clone(),
        _connection.clone(),
    );

    let call_service = CallService::new(
        config.twilio_account_sid.clone(),
        config.twilio_phone_number.clone(),
        config.twilio_api_key_sid.clone(),
        config.twilio_api_key_secret.clone(),
        config.twiml_app_sid.clone(),
        _connection.clone(),
    );

    let contact_service = ContactService::new(_connection.clone());

    let app_state = AppState {
        auth_service: Arc::new(auth_service),
        sms_service: Arc::new(sms_service),
        whatsapp_service: Arc::new(whatsapp_service),
        call_service: Arc::new(call_service),
        contact_service: Arc::new(contact_service),
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
        .nest("/sms", sms_controller::router())
        .nest("/whatsapp", whatsapp_controller::router())
        .nest("/call", call_controller::router())
        .nest("/contact", contact_controller::router())
        .nest_service("/uploads", ServeDir::new("uploads"))
        .with_state(app_state)
        .layer(cors);

    let address = format!("0.0.0.0:{}", config.port);

    let listener = TcpListener::bind(&address).await.unwrap();

    println!("Rodando na porta 5000");

    axum::serve(listener, app).await.unwrap();
}
