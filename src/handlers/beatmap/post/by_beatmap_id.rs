use axum::{Json, extract::State, http::StatusCode};
use db::db::DatabaseManager;
use db::pending_beatmap::PendingBeatmap;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct PostBeatmapByIdRequest {
    pub id: i32,
}

#[derive(Serialize)]
pub struct PostBeatmapByIdResponse {
    pub message: String,
    pub status: String,
}
/*
pub async fn handler(
    State(db): State<DatabaseManager>,
    Json(payload): Json<PostBeatmapByIdRequest>,
) -> Result<Json<PostBeatmapByIdResponse>, StatusCode> {
    if payload.id == 0 {
        return Ok(Json(PostBeatmapByIdResponse {
            message: "No Id provided".to_string(),
            status: "400".to_string(),
        }));
    }
    let beatmap = OsuApiService::instance()
        .beatmap_by_osu_id(payload.id)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    println!("DEBUG: Beatmap: {:?}", payload.id);
    let _ = PendingBeatmap::insert(
        db.get_pool(),
        &beatmap.checksum.ok_or(StatusCode::BAD_REQUEST)?,
        Some(payload.id),
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(PostBeatmapByIdResponse {
        message: "Beatmap added to queue".to_string(),
        status: "200".to_string(),
    }))
}
*/
