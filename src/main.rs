//! # Template Axum SQLx API
//!
//! Ce module est le point d'entrée principal de l'application.
//! Il configure et démarre le serveur HTTP avec Axum.
//!
//! ## Fonctionnalités
//! - Configuration depuis variables d'environnement (.env)
//! - Initialisation de la base de données
//! - Configuration du logging
//! - Configuration CORS
//! - Gestion des erreurs

mod config;
mod handlers;
mod middleware;
mod routes;

use crate::config::Config;
use crate::middleware::logging::setup_middleware;
use axum::{Router};
use db::db::DatabaseManager;
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tracing::info;

#[tokio::main]
async fn main() {
    let config = Config::load().expect("Failed to load configuration");

    let mut db = DatabaseManager::new();
    db.connect(&config.database)
        .await
        .expect("Failed to connect to database");

    let app = Router::new().merge(routes::create_router(db)).layer(
        ServiceBuilder::new()
            .layer(CorsLayer::permissive()),
    );

    let app = setup_middleware(app);

    let addr: SocketAddr = config
        .server_address()
        .parse()
        .expect("Invalid server address");
    
    info!("listening on {}", addr);
    
    axum::serve(
        tokio::net::TcpListener::bind(addr).await.unwrap(),
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
