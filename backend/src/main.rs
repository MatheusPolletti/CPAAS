mod auth_config;
mod auth_controller;
mod auth_service;
mod database_service;

pub mod entities;

use std::sync::Arc;

use auth_config::Config;
use auth_service::AuthService;
use axum::{Router, routing::post};
use tokio::net::TcpListener;

use crate::database_service::connect_db;

#[tokio::main]
async fn main() {
    let config = Config::from_env();

    let _connection = connect_db(&config.database_url).await;

    let auth_service = AuthService::new(config.secret, _connection);

    let app_state = Arc::new(auth_service);

    let app = Router::new()
        .route("/register", post(auth_controller::register))
        .route("/login", post(auth_controller::login))
        .with_state(app_state);

    let address = format!("0.0.0.0:{}", config.port);

    let listener = TcpListener::bind(&address).await.unwrap();

    println!("Rodando na porta 3000");

    axum::serve(listener, app).await.unwrap();
}
