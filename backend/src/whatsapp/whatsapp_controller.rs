use std::sync::Arc;

use axum::{
    Form, Json, Router,
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::{get, post},
};
use reqwest::StatusCode;

use crate::{
    auth::auth_extractor::AuthenticatedUser,
    dto::PaginationQuery,
    whatsapp::{
        whatsapp_dto::{
            WhatsAppInbound, WhatsAppOutbound, WhatsAppStatusWebhook, WhatsappChatMessageResponse,
            WhatsappChatThreadResponse, WhatsappContactListResponse,
        },
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
        .handle_receive(
            &payload.from,
            &payload.message,
            &payload.profile_name,
            &payload.message_sid,
        )
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

pub async fn receive_whatsapp_status(
    State(whatsapp_service): State<Arc<WhatsappService>>,
    Form(payload): Form<WhatsAppStatusWebhook>,
) -> impl IntoResponse {
    match whatsapp_service
        .update_message_status(&payload.message_sid, &payload.status)
        .await
    {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
pub async fn list_contacts(
    _user: AuthenticatedUser,
    State(whatsapp_service): State<Arc<WhatsappService>>,
) -> impl IntoResponse {
    match whatsapp_service.get_unique_contacts().await {
        Ok(contacts) => {
            let response = WhatsappContactListResponse { contacts };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response(),
    }
}

pub async fn list_messages_contact(
    _user: AuthenticatedUser,
    State(whatsapp_service): State<Arc<WhatsappService>>,
    Path(contact_number): Path<String>,
    Query(pagination): Query<PaginationQuery>,
) -> impl IntoResponse {
    let page = pagination.page.unwrap_or(0);

    match whatsapp_service
        .get_chat_thread(&contact_number, page)
        .await
    {
        Ok(models) => {
            let messages = models
                .into_iter()
                .map(|m| WhatsappChatMessageResponse {
                    id: m.id,
                    direction: m.direction,
                    body: m.body,
                    status: m.status,
                    created_at: m.created_at,
                })
                .collect();

            let response = WhatsappChatThreadResponse {
                contact: contact_number,
                messages,
            };

            (StatusCode::OK, Json(response)).into_response()
        }
        Err(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response(),
    }
}

pub fn router() -> Router<crate::AppState> {
    Router::new()
        .route("/send", post(send_whatsapp))
        .route("/webhook", post(receive_whatsapp_message))
        .route("/contacts", get(list_contacts))
        .route("/chat/{contact_number}", get(list_messages_contact))
        .route("/status", post(receive_whatsapp_status))
}
