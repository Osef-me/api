//! # Help Routes Module
//!
//! Ce module configure les routes d'aide et de diagnostic de l'API.

use crate::handlers::help::{cache_stats, ping, security_stats};

use axum::{Router, routing::get};
use db::db::DatabaseManager;
/// Créer le routeur pour les routes d'aide
pub fn router() -> Router<DatabaseManager> {
    Router::new()
        .route("/help/ping", get(ping))
        .route("/help/security-stats", get(security_stats))
        .route("/help/cache-stats", get(cache_stats))
}
