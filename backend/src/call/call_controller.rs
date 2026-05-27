use axum::response::IntoResponse;
use axum::{
    Form, Json, Router,
    extract::State,
    routing::{get, post},
};
use std::sync::Arc;

use crate::call::call_dto::{CallHistoryResponse, CallStatusWebhook};
use crate::{
    auth::auth_extractor::AuthenticatedUser,
    call::{
        call_dto::{VoiceConnectRequest, VoiceInbound, VoiceTokenResponse},
        call_service::CallService,
    },
};

pub async fn get_call_twiml(
    State(call_service): State<Arc<CallService>>,
    Form(payload): Form<VoiceConnectRequest>,
) -> impl IntoResponse {
    let to = payload.to.trim();
    if to.is_empty() {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            [(axum::http::header::CONTENT_TYPE, "application/xml")],
            "<Response></Response>",
        )
            .into_response();
    }

    let _ = call_service
        .register_outbound_webrtc(&payload.call_sid, to, payload.from)
        .await;

    let caller_id = call_service.get_caller_id();

    let status_webhook_url = "https://dinghy-drainable-headstand.ngrok-free.dev/call/status";

    let twiml_response = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
        <Response>
            <Dial callerId="{}">
                <Number statusCallback="{}" statusCallbackEvent="completed answered">{}</Number>
            </Dial>
        </Response>"#,
        caller_id, status_webhook_url, to
    );

    (
        axum::http::StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, "application/xml")],
        twiml_response,
    )
        .into_response()
}

pub async fn call_status_webhook(
    State(call_service): State<Arc<CallService>>,
    Form(payload): Form<CallStatusWebhook>, // O novo DTO que criamos
) -> impl IntoResponse {
    // Se a Twilio mandar o ParentCallSid, usamos ele (pois é o ID do WebRTC que salvamos)
    // Se não mandar, usamos o CallSid normal.
    let target_sid = payload.parent_call_sid.unwrap_or(payload.call_sid);

    // Mandamos atualizar o banco
    let _ = call_service
        .update_call_status(&target_sid, &payload.call_status)
        .await;

    // A Twilio só quer um "OK" vazio como confirmação
    (
        axum::http::StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, "application/xml")],
        "<Response></Response>",
    )
        .into_response()
}

pub async fn receive_inbound_call(
    State(call_service): State<Arc<CallService>>,
    Form(payload): Form<VoiceInbound>,
) -> impl IntoResponse {
    let _ = call_service
        .register_inbound(&payload.call_sid, &payload.from, &payload.to)
        .await;

    let ultimos_digitos = if payload.from.len() >= 4 {
        &payload.from[payload.from.len() - 4..]
    } else {
        "desconhecido"
    };

    let twiml_response = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
        <Response>
            <Say language="pt-BR" voice="Polly.Camila-Neural">
                Olá! Bem vindo à Pró I T Cloud Solutions.
                Nós identificamos o seu número final {}.
                Por favor, aguarde na linha enquanto localizamos o seu cadastro.
            </Say>
            
            <Play>http://com.twilio.music.guitars.s3.amazonaws.com/Pitx_-_A_Nightingales_Song.mp3</Play>
        </Response>"#,
        ultimos_digitos
    );

    (
        axum::http::StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, "application/xml")],
        twiml_response,
    )
}

pub async fn get_history(
    _user: AuthenticatedUser,
    State(call_service): State<Arc<CallService>>,
) -> impl IntoResponse {
    match call_service.get_call_history().await {
        Ok(models) => {
            let history: Vec<CallHistoryResponse> = models
                .into_iter()
                .map(|m| CallHistoryResponse {
                    id: m.id,
                    call_sid: m.call_sid,
                    from_number: m.from_number,
                    to_number: m.to_number,
                    direction: m.direction,
                    status: m.status,
                    created_at: m.created_at,
                })
                .collect();

            Json(history).into_response()
        }
        Err(msg) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, msg).into_response(),
    }
}

pub async fn get_voice_token(
    user: AuthenticatedUser,
    State(call_service): State<Arc<CallService>>,
) -> impl IntoResponse {
    let identity = format!("user-{}", user.0.sub);
    match call_service.generate_voice_token(&identity) {
        Ok(token) => (
            axum::http::StatusCode::OK,
            Json(VoiceTokenResponse { token }),
        )
            .into_response(),
        Err(msg) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, msg).into_response(),
    }
}

pub fn router() -> Router<crate::AppState> {
    Router::new()
        .route("/twiml", post(get_call_twiml))
        .route("/inbound", post(receive_inbound_call))
        .route("/history", get(get_history))
        .route("/token", get(get_voice_token))
        .route("/status", post(call_status_webhook))
}
