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
            post(handlers::beatmapsets::batch::checksums::handler),
        )
        .route(
            "/beatmaps/{beatmap_osu_id}/rates/{centirate}",
            get(handlers::beatmapsets::rate::handler),
        )
        .route(
            "/beatmapsets",
            get(handlers::beatmapsets::get::list::handler),
        )
        .route(
            "/beatmapsets/{osu_id}",
            get(handlers::beatmapsets::get::by_osu_id::handler),
        )
        .with_state(db)
}
