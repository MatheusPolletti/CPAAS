use axum::{Json, Router, extract::State, response::IntoResponse, routing::post};
use reqwest::StatusCode;
use std::sync::Arc;

use crate::{
    auth::auth_extractor::AuthenticatedUser,
    contact::{contact_dto::SaveContactRequest, contact_service::ContactService},
};

pub async fn save_contact_handler(
    _user: AuthenticatedUser,
    State(sms_service): State<Arc<ContactService>>,
    Json(payload): Json<SaveContactRequest>,
) -> impl IntoResponse {
    match sms_service
        .save_contact_name(&payload.phone_number, &payload.name)
        .await
    {
        Ok(_) => StatusCode::OK.into_response(),
        Err(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response(),
    }
}

pub fn router() -> Router<crate::AppState> {
    Router::new().route("/save", post(save_contact_handler))
}
