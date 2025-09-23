//! # Beatmap Routes Module
//!
//! Ce module configure les routes de beatmap.

use crate::handlers;
use axum::{Router, routing::get};
use db::db::DatabaseManager;

pub fn router(db: DatabaseManager) -> Router<DatabaseManager> {
    Router::new()
        .route(
            "/pending_beatmap/status/{id}",
            get(handlers::pending_beatmap::get::status_by_osu_id::handler),
        )
        .with_state(db)
}
