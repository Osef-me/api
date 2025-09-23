//! # Beatmap Routes Module
//!
//! Ce module configure les routes de beatmap.

use crate::handlers;
use axum::{
    Router,
    routing::{post},
};
use db::db::DatabaseManager;

pub fn router(db: DatabaseManager) -> Router<DatabaseManager> {
    Router::new()
        .route(
            "/beatmaps/imports",
            post(handlers::beatmap::batch::checksums::handler),
        )
        .with_state(db)
}
