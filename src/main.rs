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
use crate::middleware::anti_kiddie::{anti_kiddie_middleware, cleanup_old_entries};
use crate::middleware::cache::{cache_middleware, cleanup_cache_stats, warm_cache};
use crate::middleware::logging::setup_middleware;
use axum::{Router, middleware::from_fn};
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

    // Démarrer les tâches de nettoyage
    tokio::spawn(cleanup_old_entries());
    info!("🛡️ Anti-kiddie cleanup task started");

    tokio::spawn(cleanup_cache_stats());
    info!("💾 Cache stats cleanup task started");

    // Pré-chauffer le cache
    tokio::spawn(warm_cache());
    info!("🔥 Cache warming task started");

    let app = Router::new().merge(routes::create_router(db)).layer(
        ServiceBuilder::new()
            .layer(from_fn(cache_middleware)) // Cache en premier (plus proche de la réponse)
            .layer(from_fn(anti_kiddie_middleware)) // Sécurité après cache
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
