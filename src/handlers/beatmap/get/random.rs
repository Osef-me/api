use axum::{Json, extract::Query, extract::State, http::StatusCode};
use db::db::DatabaseManager;
use db::filters::Filters;
use db::short::complete::types::BeatmapsetCompleteShort;
use serde::Serialize;

#[derive(Serialize)]
pub struct BeatmapRandomResponse {
    pub beatmaps: Vec<BeatmapsetCompleteShort>,
    pub count: usize,
}

pub async fn handler(
    State(db): State<DatabaseManager>,
    Query(query): Query<Filters>,
) -> Result<Json<BeatmapRandomResponse>, StatusCode> {
    let pool = db.get_pool();

    let beatmaps = BeatmapsetCompleteShort::random_by_filters(pool, &query)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let len = beatmaps.len();
    Ok(Json(BeatmapRandomResponse {
        beatmaps,
        count: len,
    }))
}
