use crate::handlers::scores::{get::by_beatmap_osu_id_and_rate, post::submit_score};
use axum::{Router, routing::{get, post}};
use db::db::DatabaseManager;

pub fn router(db: DatabaseManager) -> Router<DatabaseManager> {
    Router::new()
        .route("/scores/submit", post(submit_score::handler))
        .route("/scores/beatmap/{osu_id}/{rate}", get(by_beatmap_osu_id_and_rate::handler))
        .with_state(db)
}
