use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(length(
        min = 3,
        message = "O nome de usuário deve ter pelo menos 3 caracteres."
    ))]
    pub username: String,

    #[validate(email(message = "O formato do e-mail é inválido."))]
    pub email: String,

    #[validate(length(min = 8, message = "A senha deve ter no mínimo 8 caracteres."))]
    pub password: String,
}

#[derive(Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email(message = "E-mail inválido."))]
    pub email: String,
    #[validate(length(min = 1, message = "A senha não pode estar vazia."))]
    pub password: String,
}

#[derive(Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Serialize)]
pub struct RegisterSuccessMessage {
    pub success: bool,
    pub message: String,
}

#[derive(Serialize)]
pub struct UserProfile {
    pub id: i32,
    pub username: String,
    pub email: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackendToken {
    pub access_token: String,
    pub expires_in: i64,
    pub refresh_token: String,
    pub refresh_expires_in: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponseData {
    pub user: UserProfile,
    pub backend_token: BackendToken,
}

#[derive(Serialize)]
pub struct LoginResponseWrapper {
    pub success: bool,
    pub data: LoginResponseData,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshResponseData {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub refresh_expires_in: i64,
}

#[derive(Serialize)]
pub struct RefreshResponseWrapper {
    pub success: bool,
    pub data: RefreshResponseData,
}
