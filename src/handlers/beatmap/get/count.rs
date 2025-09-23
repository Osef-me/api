use axum::{Json, extract::State, http::StatusCode};
use db::db::DatabaseManager;
use db::extended::beatmap::query::get_all_stats;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
pub struct BeatmapCountResponse {
    pub total_beatmaps: i64,
    pub total_beatmapsets: i64,
    pub patterns: HashMap<String, u64>,
}

pub async fn handler(
    State(db): State<DatabaseManager>,
) -> Result<Json<BeatmapCountResponse>, StatusCode> {
    let pool = db.get_pool();

    // Récupérer toutes les statistiques en une seule requête optimisée
    let (total_beatmaps, total_beatmapsets, patterns) = tokio::time::timeout(
        std::time::Duration::from_millis(200), // Timeout de 200ms
        get_all_stats(pool),
    )
    .await
    .map_err(|_| StatusCode::REQUEST_TIMEOUT)?
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total_beatmaps = total_beatmaps.unwrap_or(0);
    let total_beatmapsets = total_beatmapsets.unwrap_or(0);

    Ok(Json(BeatmapCountResponse {
        total_beatmaps,
        total_beatmapsets,
        patterns,
    }))
}
