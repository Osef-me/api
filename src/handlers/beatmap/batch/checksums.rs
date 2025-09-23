use axum::{Json, extract::State, http::StatusCode};
use db::db::DatabaseManager;
use db::models::beatmaps::pending_beatmap::PendingBeatmapRow;
use dto::common::ApiResponse;
use dto::common::Empty;
use dto::models::pending_beatmap::batch::types::BatchChecksumsRequestDto;
use tracing::error;

#[utoipa::path(
    post,
        path = "/api/beatmaps/imports",
    request_body = BatchChecksumsRequestDto,
    responses(
        (status = 200, description = "Checksums enqueued for processing", body = ApiResponse<Empty>),
        (status = 400, description = "Bad request", body = ApiResponse<Empty>),
        (status = 500, description = "Internal server error")
    ),
    tag = "Beatmaps"
)]
pub async fn handler(
    State(db): State<DatabaseManager>,
    Json(payload): Json<BatchChecksumsRequestDto>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    let batch: Vec<PendingBeatmapRow> = payload
        .checksums
        .into_iter()
        .take(50)
        .map(|h| PendingBeatmapRow {
            id: 1,
            osu_hash: h,
            osu_id: None,
            created_at: None,
        })
        .collect();

    if batch.is_empty() {
        return Ok(Json(ApiResponse::error("400", "No checksum provided")));
    }

    let inserted = PendingBeatmapRow::bulk_insert(db.get_pool(), &batch)
        .await
        .map_err(|e| {
            error!(error = %e, "bulk insert failed");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(ApiResponse::ok(
        format!("{} checksums added to processing queue", inserted),
        None,
    )))
}
