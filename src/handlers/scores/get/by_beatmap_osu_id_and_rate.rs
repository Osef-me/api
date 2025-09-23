use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use db::db::DatabaseManager;
use db::scores::score_display::ScoreDisplay;
use super::types::ScoreResponse;

pub async fn handler(
    State(db): State<DatabaseManager>,
    Path((osu_id, rate)): Path<(i32, f64)>,
) -> Result<Json<Vec<ScoreResponse>>, StatusCode> {
    let scores = ScoreDisplay::find_top_scores_by_beatmap_osu_id_and_rate(
        db.get_pool(),
        osu_id,
        rate,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Convertir ScoreDisplay en ScoreResponse
    let response_scores: Vec<ScoreResponse> = scores
        .into_iter()
        .map(|score| score.into())
        .collect();

    Ok(Json(response_scores))
}
