use axum::{Json, extract::Query, extract::State, http::StatusCode};
use db::db::DatabaseManager;
use db::filters::Filters;
use db::short::complete::types::BeatmapsetCompleteShort;
use serde::Serialize;

#[derive(Serialize)]
pub struct BeatmapFiltersResponse {
    pub beatmaps: Vec<BeatmapsetCompleteShort>,
    pub total: usize,
    pub page: usize,
    pub per_page: usize,
    pub total_pages: usize,
}

pub async fn handler(
    State(db): State<DatabaseManager>,
    Query(query): Query<Filters>,
) -> Result<Json<BeatmapFiltersResponse>, StatusCode> {
    let pool = db.get_pool();

    // Pagination - utiliser les paramètres des filtres
    let per_page = query.per_page.unwrap_or(10);
    let page = query.page.unwrap_or(1);

    let total = BeatmapsetCompleteShort::count_by_filters(pool, &query)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let beatmaps = BeatmapsetCompleteShort::find_by_filters(pool, &query)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Calculer le nombre total de pages
    let total_pages = (total + per_page as i64 - 1) / per_page as i64;

    Ok(Json(BeatmapFiltersResponse {
        beatmaps,
        total: total as usize,
        page,
        per_page,
        total_pages: total_pages as usize,
    }))
}
