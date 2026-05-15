use axum::{
    extract::{FromRef, FromRequestParts},
    http::{StatusCode, request::Parts},
};
use jsonwebtoken::{DecodingKey, Validation, decode};
use std::sync::Arc;

use crate::auth::{auth_service::AuthService, jwt_utils::Claims};

pub struct AuthenticatedUser(pub Claims);

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    Arc<AuthService>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth_service = Arc::from_ref(state);

        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .ok_or((StatusCode::UNAUTHORIZED, "Token não fornecido".to_string()))?;

        if !auth_header.starts_with("Bearer ") {
            return Err((
                StatusCode::UNAUTHORIZED,
                "Formato de token inválido".to_string(),
            ));
        }

        let token = &auth_header[7..];

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(auth_service.get_pepper().as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                "Token inválido ou expirado".to_string(),
            )
        })?;

        if token_data.claims.token_type != "access" {
            return Err((
                StatusCode::UNAUTHORIZED,
                "Tipo de token inválido".to_string(),
            ));
        }

        Ok(AuthenticatedUser(token_data.claims))
    }
}
