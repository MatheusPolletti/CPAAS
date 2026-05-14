use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::auth_service::AuthService;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub id: i32,
    pub username: String,
    pub email: String,
}

pub type AppState = Arc<AuthService>;

pub async fn register(
    State(auth_service): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> impl IntoResponse {
    match auth_service
        .register_user(&payload.username, &payload.email, &payload.password)
        .await
    {
        Ok(user) => {
            let response = UserResponse {
                id: user.id,
                username: user.username,
                email: user.email,
            };
            (StatusCode::CREATED, Json(response)).into_response()
        }
        Err(e) => {
            let error_msg = format!("Erro ao criar usuário: {:?}", e);
            (StatusCode::BAD_REQUEST, error_msg).into_response()
        }
    }
}

pub async fn login(
    State(auth_service): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    match auth_service.login(&payload.email, &payload.password).await {
        Ok(user) => {
            let response = UserResponse {
                id: user.id,
                username: user.username,
                email: user.email,
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(message) => (StatusCode::UNAUTHORIZED, message).into_response(),
    }
}
