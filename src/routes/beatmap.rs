//! # Beatmap Routes Module
//!
//! Ce module configure les routes de beatmap.

use crate::handlers;
use axum::{
    Router,
    routing::{post, get},
};
use db::db::DatabaseManager;

pub fn router(db: DatabaseManager) -> Router<DatabaseManager> {
    Router::new()
        .route(
            "/beatmaps/imports",
            post(handlers::beatmap::batch::checksums::handler),
        )
        .route(
            "/beatmaps",
            get(handlers::beatmap::get::list::handler),
        )
        .with_state(db)
}
