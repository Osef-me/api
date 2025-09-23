use axum::{Json, extract::Path, extract::State, http::StatusCode};
use db::db::DatabaseManager;
use db::extended::complete::types::BeatmapsetCompleteExtended;
use serde::Serialize;

#[derive(Serialize)]
pub struct BeatmapByIdExtendedResponse {
    pub beatmap: BeatmapsetCompleteExtended,
}

pub async fn handler(
    State(db): State<DatabaseManager>,
    Path(id): Path<i32>,
) -> Result<Json<BeatmapByIdExtendedResponse>, StatusCode> {
    let pool = db.get_pool();

    let beatmap = BeatmapsetCompleteExtended::find_by_beatmapset_osu_id(pool, id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(BeatmapByIdExtendedResponse {
        beatmap: beatmap.unwrap(),
    }))
}
