use axum::{extract::{State, Path, Query}, Json, http::StatusCode};
use db::db::DatabaseManager;
use dto::common::ApiResponse;
use dto::models::beatmaps::simple::{
    types::Beatmapset,
    query::find_by_osu_id::find_by_osu_id,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct BeatmapsetQuery {
    pub rating_type: Option<String>,
}

/// GET /api/beatmapsets/{osu_id}
#[utoipa::path(
    get,
    path = "/api/beatmapsets/{osu_id}",
    params(
        ("osu_id" = i32, Path, description = "Beatmapset osu_id"),
        ("rating_type" = Option<String>, Query, description = "Rating type filter (osu, etterna, quaver, malody, interlude, sunnyxxy)"),
    ),
    responses(
        (status = 200, description = "Beatmapset found", body = dto::common::ApiResponse<dto::models::beatmaps::simple::types::Beatmapset>),
        (status = 404, description = "Beatmapset not found"),
        (status = 500, description = "Internal error"),
    ),
    tag = "Beatmapsets"
)]
#[axum::debug_handler]
pub async fn handler(
    State(db): State<DatabaseManager>,
    Path(osu_id): Path<i32>,
    Query(query): Query<BeatmapsetQuery>,
) -> Result<Json<ApiResponse<Beatmapset>>, StatusCode> {
    let pool = db.get_pool();
    match find_by_osu_id(pool, osu_id, query.rating_type).await {
        Ok(Some(set)) => Ok(Json(ApiResponse::ok("ok", Some(set)))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(err) => {
            tracing::error!(error = %err, osu_id, "failed to fetch full beatmapset by osu_id");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}


