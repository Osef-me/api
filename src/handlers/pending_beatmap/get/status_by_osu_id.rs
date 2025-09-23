use axum::{Json, extract::Path, extract::State, http::StatusCode};
use db::db::DatabaseManager;
use db::pending_beatmap::PendingBeatmap;
use serde::Serialize;

#[derive(Serialize)]
pub struct PendingBeatmapStatusResponse {
    pub position: i64,
    pub total: i64,
}

pub async fn handler(
    State(db): State<DatabaseManager>,
    Path(id): Path<i32>,
) -> Result<Json<PendingBeatmapStatusResponse>, StatusCode> {
    let pool = db.get_pool();

    let position: i64 = PendingBeatmap::position_by_osu_id(pool, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    let total: i64 = PendingBeatmap::count(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(PendingBeatmapStatusResponse {
        position: position,
        total: total,
    }))
}
