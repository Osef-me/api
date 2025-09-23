use axum::{Json, extract::Path, extract::State, http::StatusCode};
use db::db::DatabaseManager;
use dto::common::ApiResponse;
use dto::models::pending_beatmap::status::query::by_osu_id::find_status_by_osu_id;
use dto::models::pending_beatmap::status::types::PendingStatusDto;

#[utoipa::path(
    get,
    path = "/api/pending_beatmap/status/{id}",
    params((
        "id" = i32,
        Path,
        description = "Beatmap ID from the official osu! API (not our internal ID)."
    )),
    responses(
        (status = 200, description = "Status fetched", body = ApiResponse<PendingStatusDto>),
        (status = 404, description = "Not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "PendingBeatmap"
)]
pub async fn handler(
    State(db): State<DatabaseManager>,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<PendingStatusDto>>, StatusCode> {
    let pool = db.get_pool();

    let Some(status) = find_status_by_osu_id(pool, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? else {
        return Err(StatusCode::NOT_FOUND);
    };

    Ok(Json(ApiResponse::ok("ok", Some(status))))
}
