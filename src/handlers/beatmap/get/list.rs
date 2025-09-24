use axum::{extract::{State, Query}, Json, http::StatusCode};
use db::db::DatabaseManager;
use dto::common::{PaginatedResponse, Pagination};
use dto::filters::{Filters, RatingFilter, PatternFilter, BeatmapFilter};
use dto::models::beatmaps::short::types::Beatmapset;
use dto::models::beatmaps::short::query::find_all::{find_all_with_filters, count_with_filters};
use serde::Deserialize;

/// GET /api/beatmaps
#[utoipa::path(
    get,
    path = "/api/beatmaps",
    params(
        ("page" = Option<usize>, Query, description = "Page index (0-based)", example = 0),
        ("per_page" = Option<usize>, Query, description = "Items per page", example = 20),
        ("rating[rating_type]" = Option<String>, Query, description = "Rating type filter", example = "overall"),
        ("rating[rating_min]" = Option<f64>, Query, description = "Min rating", example = 6.5),
        ("rating[rating_max]" = Option<f64>, Query, description = "Max rating", example = 9.5),
        ("pattern[pattern_type]" = Option<String>, Query, description = "Pattern type filter", example = "stream"),
        ("pattern[pattern_min]" = Option<f64>, Query, description = "Pattern min", example = 0.2),
        ("pattern[pattern_max]" = Option<f64>, Query, description = "Pattern max", example = 0.8),
        ("beatmap[search_term]" = Option<String>, Query, description = "Search on artist/title/creator", example = "Camellia"),
        ("beatmap[total_time_min]" = Option<i32>, Query, description = "Min total time (ms)", example = 60000),
        ("beatmap[total_time_max]" = Option<i32>, Query, description = "Max total time (ms)", example = 240000),
        ("beatmap[bpm_min]" = Option<f64>, Query, description = "Min BPM", example = 120.0),
        ("beatmap[bpm_max]" = Option<f64>, Query, description = "Max BPM", example = 220.0)
    ),
    responses(
        (status = 200, description = "List beatmaps", body = dto::common::PaginatedResponse<dto::models::beatmaps::short::types::Beatmapset>),
        (status = 500, description = "Internal error")
    ),
    tag = "Beatmaps"
)]
#[axum::debug_handler]
pub async fn handler(
    State(db): State<DatabaseManager>,
    Query(q): Query<BeatmapListQuery>,
) -> Result<Json<PaginatedResponse<Beatmapset>>, StatusCode> {
    let filters = q.into_filters();
    let pool = db.get_pool();
    let page = filters.page.unwrap_or(0) as u32;
    let per_page = filters.per_page.unwrap_or(20) as u32;
    let total = match count_with_filters(pool, &filters).await {
        Ok(t) => t as u64,
        Err(err) => {
            tracing::error!(error = %err, "failed to count beatmaps list");
            0
        }
    };
    match find_all_with_filters(pool, filters).await {
        Ok(list) => Ok(Json(PaginatedResponse {
            message: "ok".to_string(),
            status: "200".to_string(),
            data: list,
            pagination: Pagination { page, per_page, total },
        })),
        Err(err) => {
            tracing::error!(error = %err, "failed to fetch beatmaps list");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct BeatmapListQuery {
    // Pagination
    pub page: Option<usize>,
    pub per_page: Option<usize>,

    // Rating
    #[serde(alias = "rating[rating_type]", alias = "rating.rating_type")]
    pub rating_type: Option<String>,
    #[serde(alias = "rating[rating_min]", alias = "rating.rating_min")]
    pub rating_min: Option<f64>,
    #[serde(alias = "rating[rating_max]", alias = "rating.rating_max")]
    pub rating_max: Option<f64>,

    // Pattern
    #[serde(alias = "pattern[pattern_type]", alias = "pattern.pattern_type")]
    pub pattern_type: Option<String>,
    #[serde(alias = "pattern[pattern_min]", alias = "pattern.pattern_min")]
    pub pattern_min: Option<f64>,
    #[serde(alias = "pattern[pattern_max]", alias = "pattern.pattern_max")]
    pub pattern_max: Option<f64>,

    // Beatmap
    #[serde(alias = "beatmap[search_term]", alias = "beatmap.search_term")]
    pub search_term: Option<String>,
    #[serde(alias = "beatmap[total_time_min]", alias = "beatmap.total_time_min")]
    pub total_time_min: Option<i32>,
    #[serde(alias = "beatmap[total_time_max]", alias = "beatmap.total_time_max")]
    pub total_time_max: Option<i32>,
    #[serde(alias = "beatmap[bpm_min]", alias = "beatmap.bpm_min")]
    pub bpm_min: Option<f64>,
    #[serde(alias = "beatmap[bpm_max]", alias = "beatmap.bpm_max")]
    pub bpm_max: Option<f64>,
}

impl BeatmapListQuery {
    fn into_filters(self) -> Filters {
        let rating = if self.rating_type.is_some() || self.rating_min.is_some() || self.rating_max.is_some() {
            Some(RatingFilter { rating_type: self.rating_type, rating_min: self.rating_min, rating_max: self.rating_max })
        } else { None };

        let pattern = if self.pattern_type.is_some() || self.pattern_min.is_some() || self.pattern_max.is_some() {
            Some(PatternFilter { pattern_type: self.pattern_type, pattern_min: self.pattern_min, pattern_max: self.pattern_max })
        } else { None };

        let beatmap = if self.search_term.is_some() || self.total_time_min.is_some() || self.total_time_max.is_some() || self.bpm_min.is_some() || self.bpm_max.is_some() {
            Some(BeatmapFilter { search_term: self.search_term, total_time_min: self.total_time_min, total_time_max: self.total_time_max, bpm_min: self.bpm_min, bpm_max: self.bpm_max })
        } else { None };

        Filters { rating, pattern, beatmap, page: self.page, per_page: self.per_page }
    }
}


