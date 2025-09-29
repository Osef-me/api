use axum::{extract::{State, Path, Query}, Json, http::StatusCode};
use db::db::DatabaseManager;
use dto::models::beatmaps::simple::query::find_by_osu_id::find_by_osu_id;
use serde::Deserialize;

/// GET /api/beatmapsets/{osu_id}
#[utoipa::path(
    get,
    path = "/api/beatmapsets/{osu_id}",
    params(
        ("osu_id" = i32, Path, description = "Beatmapset osu ID", example = 123456),
        ("rating_type" = Option<String>, Query, description = "Filter by rating type", example = "overall")
    ),
    responses(
        (status = 200, description = "Beatmapset with difficulties", body = dto::models::beatmaps::simple::types::Beatmapset),
        (status = 404, description = "Beatmapset not found"),
        (status = 500, description = "Internal error")
    ),
    tag = "Beatmaps"
)]
#[axum::debug_handler]
pub async fn handler(
    State(db): State<DatabaseManager>,
    Path(osu_id): Path<i32>,
    Query(params): Query<BeatmapQuery>,
) -> Result<Json<dto::models::beatmaps::simple::types::Beatmapset>, StatusCode> {
    let pool = db.get_pool();

    match find_by_osu_id(pool, osu_id, params.rating_type).await {
        Ok(Some(beatmapset)) => Ok(Json(beatmapset)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(err) => {
            tracing::error!(error = %err, "failed to fetch beatmapset with osu_id {}", osu_id);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct BeatmapQuery {
    #[serde(alias = "rating_type")]
    pub rating_type: Option<String>,
}
