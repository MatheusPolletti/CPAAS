use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use std::sync::Arc;

use crate::{
    auth::auth_extractor::AuthenticatedUser,
    contact::{contact_dto::SaveContactRequest, contact_service::ContactService},
};

pub async fn save_contact_handler(
    _user: AuthenticatedUser,
    State(contact_service): State<Arc<ContactService>>,
    Json(payload): Json<SaveContactRequest>,
) -> impl IntoResponse {
    match contact_service
        .save_contact_name(
            &payload.phone_number,
            &payload.name,
            payload.company.as_deref(),
        )
        .await
    {
        Ok(_) => StatusCode::OK.into_response(),
        Err(msg) if msg == "Este número de telefone já está cadastrado." => {
            (StatusCode::CONFLICT, msg).into_response()
        }
        Err(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response(),
    }
}

pub async fn get_contacts(
    _user: AuthenticatedUser,
    State(contact_service): State<Arc<ContactService>>,
) -> impl IntoResponse {
    match contact_service.get_contacts().await {
        Ok(contacts) => (StatusCode::OK, Json(contacts)).into_response(),
        Err(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response(),
    }
}

pub fn router() -> Router<crate::AppState> {
    Router::new()
        .route("/save", post(save_contact_handler))
        .route("/contacts", get(get_contacts))
}
