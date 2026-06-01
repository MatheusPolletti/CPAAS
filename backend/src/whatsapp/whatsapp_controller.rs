use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    auth::auth_extractor::AuthenticatedUser,
    dto::PaginationQuery,
    whatsapp::{
        whatsapp_dto::{
            MediaQuery, TicketListResponse, WhatsAppInbound, WhatsAppStatusWebhook,
            WhatsappChatMessageResponse, WhatsappChatThreadResponse,
        },
        whatsapp_service::WhatsappService,
    },
};
use axum::{
    Form, Json, Router,
    extract::{Multipart, Path, Query, State},
    response::IntoResponse,
    routing::{get, post},
};
use reqwest::StatusCode;

pub async fn send_whatsapp(
    _user: AuthenticatedUser,
    State(whatsapp_service): State<Arc<WhatsappService>>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut to = String::new();
    let mut message = String::new();
    let mut media_url: Option<String> = None;
    let mut media_type: Option<String> = None;
    let mut ticket_id: Option<i32> = None;
    let mut sender_name: Option<String> = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();

        if name == "to" {
            to = field.text().await.unwrap_or_default();
        } else if name == "ticket_id" {
            ticket_id = field.text().await.unwrap_or_default().parse().ok();
        } else if name == "sender_name" {
            sender_name = Some(field.text().await.unwrap_or_default());
        } else if name == "message" {
            message = field.text().await.unwrap_or_default();
        } else if name == "file" {
            if let Some(content_type) = field.content_type() {
                media_type = Some(content_type.to_string());
            }

            let file_name = field.file_name().unwrap_or("arquivo.bin").to_string();
            let ext = file_name.split('.').last().unwrap_or("bin");

            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis();

            let saved_filename = format!("{}.{}", timestamp, ext);
            let path = format!("uploads/{}", saved_filename);

            if let Ok(bytes) = field.bytes().await {
                if !bytes.is_empty() {
                    std::fs::create_dir_all("uploads").ok();
                    let _ = std::fs::write(&path, bytes);

                    media_url = Some(format!(
                        "https://dinghy-drainable-headstand.ngrok-free.dev/uploads/{}",
                        saved_filename
                    ));
                }
            }
        }
    }

    match whatsapp_service
        .send_whatsapp(
            &to,
            &message,
            media_url,
            media_type,
            ticket_id,
            &sender_name,
        )
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
            payload.media_url.as_deref(),
            payload.media_type.as_deref(),
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

pub async fn list_active_tickets(
    _user: AuthenticatedUser,
    State(whatsapp_service): State<Arc<WhatsappService>>,
) -> impl IntoResponse {
    match whatsapp_service.get_active_tickets().await {
        Ok(tickets) => {
            let response = TicketListResponse { tickets };

            (StatusCode::OK, Json(response)).into_response()
        }
        Err(msg) => (StatusCode::INTERNAL_SERVER_ERROR, Json(msg)).into_response(),
    }
}

pub async fn list_messages_ticket(
    _user: AuthenticatedUser,
    State(whatsapp_service): State<Arc<WhatsappService>>,
    Path(ticket_id): Path<i32>,
    Query(pagination): Query<PaginationQuery>,
) -> impl IntoResponse {
    let page = pagination.page.unwrap_or(0);

    match whatsapp_service.get_chat_thread(ticket_id, page).await {
        Ok(models) => {
            let messages = models
                .into_iter()
                .map(|m| WhatsappChatMessageResponse {
                    id: m.id,
                    direction: m.direction,
                    body: m.body,
                    status: m.status,
                    created_at: m.created_at,
                    media_url: m.media_url,
                    media_type: m.media_type,
                })
                .collect();

            let response = WhatsappChatThreadResponse {
                contact: ticket_id.to_string(),
                messages,
            };

            (StatusCode::OK, Json(response)).into_response()
        }
        Err(msg) => (StatusCode::INTERNAL_SERVER_ERROR, Json(msg)).into_response(),
    }
}

pub async fn get_whatsapp_media(
    State(whatsapp_service): State<Arc<WhatsappService>>,
    Query(query): Query<MediaQuery>,
) -> impl IntoResponse {
    match whatsapp_service.fetch_media(&query.url).await {
        Ok((bytes, content_type)) => (
            StatusCode::OK,
            [(axum::http::header::CONTENT_TYPE, content_type)],
            bytes,
        )
            .into_response(),
        Err(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response(),
    }
}

pub fn router() -> Router<crate::AppState> {
    Router::new()
        .route("/send", post(send_whatsapp))
        .route("/webhook", post(receive_whatsapp_message))
        .route("/tickets", get(list_active_tickets))
        .route("/chat/{ticket_id}", get(list_messages_ticket))
        .route("/status", post(receive_whatsapp_status))
        .route("/media", get(get_whatsapp_media))
}
