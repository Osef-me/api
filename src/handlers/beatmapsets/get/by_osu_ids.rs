use axum::{extract::State, Json, http::StatusCode};
use db::db::DatabaseManager;
use dto::common::ApiResponse;
use dto::models::beatmaps::full::types::Beatmap;
use dto::models::beatmaps::simple::query::find_by_osu_ids::find_beatmaps_with_rate100_by_osu_ids;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct BeatmapOsuIdsRequest {
    pub osu_ids: Vec<i32>,
}

/// POST /api/beatmaps/rates/100
#[utoipa::path(
    post,
    path = "/api/beatmaps/rates/100",
    request_body = BeatmapOsuIdsRequest,
    responses(
        (status = 200, description = "Beatmaps with rate 100 found", body = dto::common::ApiResponse<Vec<dto::models::beatmaps::full::types::Beatmap>>),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal error"),
    ),
    tag = "Beatmaps"
)]
#[axum::debug_handler]
pub async fn handler(
    State(db): State<DatabaseManager>,
    Json(request): Json<BeatmapOsuIdsRequest>,
) -> Result<Json<ApiResponse<Vec<Beatmap>>>, StatusCode> {
    let pool = db.get_pool();
    match find_beatmaps_with_rate100_by_osu_ids(pool, request.osu_ids).await {
        Ok(beatmaps) => Ok(Json(ApiResponse::ok("ok", Some(beatmaps)))),
        Err(err) => {
            tracing::error!(error = %err, "failed to fetch beatmaps with rate 100");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}


