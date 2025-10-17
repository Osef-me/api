use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use db::db::DatabaseManager;
use dto::models::rate::find_rate_by_beatmap_osu_id_and_centirate;

/// GET /api/beatmaps/{beatmap_osu_id}/rates/{centirate}
#[utoipa::path(
    get,
    path = "/api/beatmaps/{beatmap_osu_id}/rates/{centirate}",
    params(
        ("beatmap_osu_id" = i32, Path, description = "Beatmap osu ID", example = 123456),
        ("centirate" = i32, Path, description = "Centirate value", example = 100)
    ),
    responses(
        (status = 200, description = "Rate data", body = dto::models::rate::Rates),
        (status = 404, description = "Rate not found"),
        (status = 500, description = "Internal error")
    ),
    tag = "Beatmaps"
)]
#[axum::debug_handler]
pub async fn handler(
    State(db): State<DatabaseManager>,
    Path((beatmap_osu_id, centirate)): Path<(i32, i32)>,
) -> Result<Json<dto::models::rate::Rates>, StatusCode> {
    let pool = db.get_pool();

    match find_rate_by_beatmap_osu_id_and_centirate(pool, beatmap_osu_id, centirate).await {
        Ok(Some(rate)) => Ok(Json(rate)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(err) => {
            tracing::error!(error = %err, "failed to fetch rate for beatmap {} with centirate {}", beatmap_osu_id, centirate);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
