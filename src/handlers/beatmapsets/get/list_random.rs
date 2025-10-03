use axum::{extract::{State, Query}, Json, http::StatusCode};
use db::db::DatabaseManager;
use dto::common::ApiResponse;
use dto::filters::{Filters, RatingFilter, SkillsetFilter, BeatmapFilter, BeatmapTechnicalFilter, RatesFilter};
use dto::models::beatmaps::short::types::Beatmapset;
use dto::models::beatmaps::short::query::find_random_with_filters;
use serde::Deserialize;

/// GET /api/beatmapsets/random - Returns 9 random beatmapsets with optional filters
#[utoipa::path(
    get,
    path = "/api/beatmapsets/random",
    params(
        ("rating[rating_type]" = Option<String>, Query, description = "Rating type filter", example = "overall"),
        ("rating[rating_min]" = Option<f64>, Query, description = "Min rating", example = 6.5),
        ("rating[rating_max]" = Option<f64>, Query, description = "Max rating", example = 9.5),
        ("skillset[pattern_type]" = Option<String>, Query, description = "Skillset type filter", example = "stream"),
        ("skillset[pattern_min]" = Option<f64>, Query, description = "Skillset min", example = 0.2),
        ("skillset[pattern_max]" = Option<f64>, Query, description = "Skillset max", example = 0.8),
        ("beatmap[search_term]" = Option<String>, Query, description = "Search on artist/title/creator", example = "Camellia"),
        ("beatmap[total_time_min]" = Option<i32>, Query, description = "Min total time (ms)", example = 60000),
        ("beatmap[total_time_max]" = Option<i32>, Query, description = "Max total time (ms)", example = 240000),
        ("beatmap[bpm_min]" = Option<f64>, Query, description = "Min BPM", example = 120.0),
        ("beatmap[bpm_max]" = Option<f64>, Query, description = "Max BPM", example = 220.0),
        ("beatmap_technical[od_min]" = Option<f64>, Query, description = "Min Overall Difficulty", example = 5.0),
        ("beatmap_technical[od_max]" = Option<f64>, Query, description = "Max Overall Difficulty", example = 10.0),
        ("beatmap_technical[status]" = Option<String>, Query, description = "Beatmap status", example = "ranked"),
        ("rates[drain_time_min]" = Option<i32>, Query, description = "Min drain time (seconds)", example = 60),
        ("rates[drain_time_max]" = Option<i32>, Query, description = "Max drain time (seconds)", example = 300)
    ),
    responses(
        (status = 200, description = "Random beatmapsets", body = dto::common::ApiResponse<Vec<dto::models::beatmaps::short::types::Beatmapset>>),
        (status = 500, description = "Internal error")
    ),
    tag = "Beatmaps"
)]
#[axum::debug_handler]
pub async fn handler(
    State(db): State<DatabaseManager>,
    Query(q): Query<BeatmapListQuery>,
) -> Result<Json<ApiResponse<Vec<Beatmapset>>>, StatusCode> {
    let filters = q.into_filters();
    let pool = db.get_pool();

    match find_random_with_filters(pool, filters).await {
        Ok(list) => Ok(Json(ApiResponse {
            message: "ok".to_string(),
            status: "200".to_string(),
            data: Some(list),
        })),
        Err(err) => {
            tracing::error!(error = %err, "failed to fetch random beatmaps");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct BeatmapListQuery {
    // Rating
    #[serde(alias = "rating[rating_type]", alias = "rating.rating_type")]
    pub rating_type: Option<String>,
    #[serde(alias = "rating[rating_min]", alias = "rating.rating_min")]
    pub rating_min: Option<f64>,
    #[serde(alias = "rating[rating_max]", alias = "rating.rating_max")]
    pub rating_max: Option<f64>,

    // Skillset
    #[serde(alias = "skillset[pattern_type]", alias = "skillset.pattern_type")]
    pub pattern_type: Option<String>,
    #[serde(alias = "skillset[pattern_min]", alias = "skillset.pattern_min")]
    pub pattern_min: Option<f64>,
    #[serde(alias = "skillset[pattern_max]", alias = "skillset.pattern_max")]
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

    // Beatmap Technical
    #[serde(alias = "beatmap_technical[od_min]", alias = "beatmap_technical.od_min")]
    pub od_min: Option<f64>,
    #[serde(alias = "beatmap_technical[od_max]", alias = "beatmap_technical.od_max")]
    pub od_max: Option<f64>,
    #[serde(alias = "beatmap_technical[status]", alias = "beatmap_technical.status")]
    pub status: Option<String>,

    // Rates
    #[serde(alias = "rates[drain_time_min]", alias = "rates.drain_time_min")]
    pub drain_time_min: Option<i32>,
    #[serde(alias = "rates[drain_time_max]", alias = "rates.drain_time_max")]
    pub drain_time_max: Option<i32>,
}

impl BeatmapListQuery {
    fn into_filters(self) -> Filters {
        let rating = if self.rating_type.is_some() || self.rating_min.is_some() || self.rating_max.is_some() {
            Some(RatingFilter { rating_type: self.rating_type, rating_min: self.rating_min, rating_max: self.rating_max })
        } else { None };

        let skillset = if self.pattern_type.is_some() || self.pattern_min.is_some() || self.pattern_max.is_some() {
            Some(SkillsetFilter { pattern_type: self.pattern_type, pattern_min: self.pattern_min, pattern_max: self.pattern_max })
        } else { None };

        let beatmap = if self.search_term.is_some() || self.total_time_min.is_some() || self.total_time_max.is_some() || self.bpm_min.is_some() || self.bpm_max.is_some() {
            Some(BeatmapFilter { search_term: self.search_term, total_time_min: self.total_time_min, total_time_max: self.total_time_max, bpm_min: self.bpm_min, bpm_max: self.bpm_max })
        } else { None };

        let beatmap_technical = if self.od_min.is_some() || self.od_max.is_some() || self.status.is_some() {
            Some(BeatmapTechnicalFilter { od_min: self.od_min, od_max: self.od_max, status: self.status })
        } else { None };

        let rates = if self.drain_time_min.is_some() || self.drain_time_max.is_some() {
            Some(RatesFilter { drain_time_min: self.drain_time_min, drain_time_max: self.drain_time_max })
        } else { None };

        Filters {
            rating,
            skillset,
            beatmap,
            beatmap_technical,
            rates,
            page: None,
            per_page: None
        }
    }
}
