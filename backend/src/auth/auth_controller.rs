use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use std::sync::Arc;

use crate::auth::{
    auth_dto::{
        LoginRequest, LoginResponse, RefreshRequest, RefreshResponse, RegisterRequest,
        RegisterSuccessMessage, UserResponse,
    },
    auth_extractor::AuthenticatedUser,
    auth_service::AuthService,
};

pub async fn register(
    State(auth_service): State<Arc<AuthService>>,
    Json(payload): Json<RegisterRequest>,
) -> impl IntoResponse {
    match auth_service
        .register_user(&payload.username, &payload.email, &payload.password)
        .await
    {
        Ok(_) => {
            let response = RegisterSuccessMessage {
                success: true,
                message: "Usuário criado com sucesso! Redirecionando...".to_string(),
            };
            (StatusCode::CREATED, Json(response)).into_response()
        }
        Err(_) => {
            let response = RegisterSuccessMessage {
                success: false,
                message: format!("Erro ao registrar"),
            };
            (StatusCode::BAD_REQUEST, Json(response)).into_response()
        }
    }
}

pub async fn login(
    State(auth_service): State<Arc<AuthService>>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    match auth_service.login(&payload.email, &payload.password).await {
        Ok((user, access_token, refresh_token)) => {
            let response = LoginResponse {
                user: UserResponse {
                    id: user.id,
                    username: user.username,
                    email: user.email,
                },
                access_token,
                refresh_token,
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(message) => (StatusCode::UNAUTHORIZED, message).into_response(),
    }
}

pub async fn refresh(
    State(auth_service): State<Arc<AuthService>>,
    Json(payload): Json<RefreshRequest>,
) -> impl IntoResponse {
    match auth_service.refresh_session(&payload.refresh_token).await {
        Ok((access_token, refresh_token)) => {
            let response = RefreshResponse {
                access_token,
                refresh_token,
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(message) => (StatusCode::UNAUTHORIZED, message).into_response(),
    }
}

pub async fn logout(
    State(auth_service): State<Arc<AuthService>>,
    user: AuthenticatedUser,
) -> impl IntoResponse {
    match auth_service.logout(user.0.sub).await {
        Ok(_) => {
            let response = RegisterSuccessMessage {
                success: true,
                message: "Logout realizado com sucesso.".to_string(),
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response(),
    }
}

pub fn router() -> axum::Router<crate::AppState> {
    axum::Router::new()
        .route("/register", axum::routing::post(register))
        .route("/login", axum::routing::post(login))
        .route("/refresh", axum::routing::post(refresh))
        .route("/logout", axum::routing::post(logout))
}
