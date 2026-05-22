use axum::{
    Form, Json, Router,
    extract::State,
    response::IntoResponse,
    routing::{get, post},
};
use reqwest::StatusCode;
use std::sync::Arc;

use crate::{
    auth::auth_extractor::AuthenticatedUser,
    call::{
        call_dto::{CallOutbound, VoiceInbound},
        call_service::CallService,
    },
};

pub async fn call(
    user: AuthenticatedUser,
    State(call_service): State<Arc<CallService>>,
    Json(payload): Json<CallOutbound>,
) -> impl IntoResponse {
    let user_id = user.0.sub;

    match call_service.call(&payload.to, user_id).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response(),
    }
}

pub async fn get_call_twiml() -> impl IntoResponse {
    let twiml_response = r#"<?xml version="1.0" encoding="UTF-8"?>
    <Response>
        <Say language="pt-BR" voice="Polly.Camila-Neural">
            Olá! Obrigado por ligar para a Pró I T Cloud Solutions. 
            Este é um teste do nosso sistema automático feito em Rust.
        </Say>
        
        <Play>http://demo.twilio.com/docs/classic.mp3</Play>
        
        <Say language="pt-BR" voice="Polly.Camila-Neural">
            Ligação finalizada com sucesso. Até logo!
        </Say>
    </Response>"#;

    (
        axum::http::StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, "application/xml")],
        twiml_response,
    )
}

pub async fn receive_inbound_call(
    State(call_service): State<Arc<CallService>>,
    Form(payload): Form<VoiceInbound>,
) -> impl IntoResponse {
    println!("📞 Ligação recebida do número: {}", payload.from);

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
        Ok(history) => (axum::http::StatusCode::OK, Json(history)).into_response(),
        Err(msg) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, msg).into_response(),
    }
}

pub fn router() -> Router<crate::AppState> {
    Router::new()
        .route("/call", post(call))
        .route("/twiml", post(get_call_twiml))
        .route("/inbound", post(receive_inbound_call))
        .route("/history", get(get_history))
}
