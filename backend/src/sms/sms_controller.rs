use axum::{
    Form, Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use std::sync::Arc;

use crate::{
    auth::auth_extractor::AuthenticatedUser,
    dto::PaginationQuery,
    sms::{
        sms_dto::{
            ChatMessageResponse, ChatThreadResponse, ContactListResponse, SendSmsRequest,
            TwilioWebhook,
        },
        sms_service::SmsService,
    },
};

pub async fn send_sms_handler(
    _user: AuthenticatedUser,
    State(sms_service): State<Arc<SmsService>>,
    Json(payload): Json<SendSmsRequest>,
) -> impl IntoResponse {
    match sms_service.send_sms(&payload.to, &payload.body).await {
        Ok(_) => (StatusCode::OK).into_response(),
        Err(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response(),
    }
}

pub async fn receive_sms_handler(
    State(sms_service): State<Arc<SmsService>>,
    Form(payload): Form<TwilioWebhook>,
) -> impl IntoResponse {
    println!("Recebido");

    match sms_service
        .save_incoming_sms(&payload.from, &payload.body)
        .await
    {
        Ok(_) => (
            StatusCode::OK,
            [(axum::http::header::CONTENT_TYPE, "application/xml")],
            "<Response></Response>",
        )
            .into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            [(axum::http::header::CONTENT_TYPE, "application/xml")],
            "<Response></Response>",
        )
            .into_response(),
    }
}

pub async fn list_contacts_handler(
    _user: AuthenticatedUser,
    State(sms_service): State<Arc<SmsService>>,
) -> impl IntoResponse {
    match sms_service.get_unique_contacts().await {
        Ok(contacts) => {
            let response = ContactListResponse { contacts };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response(),
    }
}

pub async fn get_chat_handler(
    _user: AuthenticatedUser,
    State(sms_service): State<Arc<SmsService>>,
    Path(contact_number): Path<String>,
    Query(pagination): Query<PaginationQuery>,
) -> impl IntoResponse {
    let page = pagination.page.unwrap_or(0);

    match sms_service.get_chat_thread(&contact_number, page).await {
        Ok(models) => {
            let messages = models
                .into_iter()
                .map(|m| ChatMessageResponse {
                    id: m.id,
                    direction: m.direction,
                    body: m.body,
                    status: m.status,
                    created_at: m.created_at,
                })
                .collect();

            let response = ChatThreadResponse {
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
        .route("/send", post(send_sms_handler))
        .route("/webhook", post(receive_sms_handler))
        .route("/contacts", get(list_contacts_handler)) // GET /sms/contacts
        .route("/chat/{contact_number}", get(get_chat_handler)) // GET /sms/chat/+551199999999
}
