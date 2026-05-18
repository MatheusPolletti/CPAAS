use std::sync::Arc;

use axum::{
    Form, Json, Router,
    extract::State,
    response::IntoResponse,
    routing::{get, post},
};
use reqwest::StatusCode;

use crate::{
    auth::auth_extractor::AuthenticatedUser,
    whatsapp::{
        whatsapp_dto::{WhatsAppInbound, WhatsAppOutbound},
        whatsapp_service::WhatsappService,
    },
};

pub async fn send_whatsapp(
    _user: AuthenticatedUser,
    State(whatsapp_service): State<Arc<WhatsappService>>,
    Json(payload): Json<WhatsAppOutbound>,
) -> impl IntoResponse {
    match whatsapp_service
        .send_whatsapp(&payload.to, &payload.message)
        .await
    {
        Ok(_) => (StatusCode::OK).into_response(),
        Err(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response(),
    }
}

pub async fn receive_whatsapp_message(
    State(whatsapp_service): State<Arc<WhatsappService>>,
    Form(payload): Form<WhatsAppInbound>,
) -> impl IntoResponse {
    match whatsapp_service
        .handle_receive(&payload.from, &payload.message, &payload.profile_name)
        .await
    {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Erro ao receber a mensagem",
        )
            .into_response(),
    }
}

pub async fn list_contacts() {}

pub async fn list_messages_contact() {}

pub fn router() -> Router<crate::AppState> {
    Router::new()
        .route("/send", post(send_whatsapp))
        .route("/webhook", post(receive_whatsapp_message))
        .route("/contacts", get(list_contacts))
        .route("/chat/{contact_number}", get(list_messages_contact))
}
