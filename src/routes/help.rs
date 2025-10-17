//! # Help Routes Module
//!
//! Ce module configure les routes d'aide et de diagnostic de l'API.

use crate::handlers::help::ping;

use axum::{Router, routing::get};
use db::db::DatabaseManager;
/// Créer le routeur pour les routes d'aide
pub fn router() -> Router<DatabaseManager> {
    Router::new().route("/help/ping", get(ping))
}
