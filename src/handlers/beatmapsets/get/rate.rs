
use axum::{extract::{Path, State}, Json, http::StatusCode};
use db::db::DatabaseManager;
use dto::common::ApiResponse;
use dto::models::beatmaps::full::{
    types::Rates,
    query::by_osu_id::find_rate_by_osu_id_and_centirate,
};

/// GET /api/beatmaps/{osu_id}/rate/{centirate}
#[utoipa::path(
    get,
    path = "/api/beatmaps/{osu_id}/rate/{centirate}",
    params(
        ("osu_id" = i32, Path, description = "Beatmap osu_id"),
        ("centirate" = i32, Path, description = "Centirate value")
    ),
    responses(
        (status = 200, description = "Rate with ratings", body = dto::common::ApiResponse<dto::models::beatmaps::full::types::Rates>),
        (status = 404, description = "Rate not found"),
        (status = 500, description = "Internal error"),
    ),
    tag = "Beatmaps"
)]
#[axum::debug_handler]
pub async fn handler(
    State(db): State<DatabaseManager>,
    Path((osu_id, centirate)): Path<(i32, i32)>,
) -> Result<Json<ApiResponse<Rates>>, StatusCode> {
    let pool = db.get_pool();
    match find_rate_by_osu_id_and_centirate(pool, osu_id, centirate).await {
        Ok(Some(rate)) => Ok(Json(ApiResponse::ok("ok", Some(rate)))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(err) => {
            tracing::error!(error = %err, osu_id, centirate, "failed to fetch rate with ratings");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
