use axum::{Json, extract::State, http::StatusCode};
use db::db::DatabaseManager;
use db::extended::beatmap::BeatmapExtended;
use db::short::beatmap::BeatmapShort;
use db::weekly::maps::weekly_maps::WeeklyMaps;
use db::weekly::weekly::Weekly;
use reqwest;
use rosu_map::Beatmap;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::str::FromStr;

#[derive(Deserialize)]
pub struct SetupWeekRequest {
    pub week_number: i32,
    pub beatmap_checksums: [String; 7], // Exactement 7 MD5 checksums
}

#[derive(Serialize)]
pub struct SetupWeekResponse {
    pub message: String,
    pub status: String,
    pub weekly_id: Option<i32>,
    pub added_beatmaps: Option<Vec<i32>>,
}

pub async fn handler(
    State(db): State<DatabaseManager>,
    Json(payload): Json<SetupWeekRequest>,
) -> Result<Json<SetupWeekResponse>, StatusCode> {
    let pool = db.get_pool();

    // 1. Vérifier si une semaine avec ce numéro existe déjà
    let week_name = format!("Week {}", payload.week_number);
    if Weekly::exists_by_name(pool, &week_name)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        return Ok(Json(SetupWeekResponse {
            message: "Week already exists".to_string(),
            status: "409".to_string(),
            weekly_id: None,
            added_beatmaps: None,
        }));
    }

    // 2. Vérifier que tous les checksums existent dans la base de données
    let mut beatmap_ids = Vec::new();
    let mut osu_ids = Vec::new();
    for checksum in &payload.beatmap_checksums {
        // Vérifier l'existence
        if !BeatmapExtended::exists_by_checksum(pool, checksum)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        {
            return Ok(Json(SetupWeekResponse {
                message: format!("Beatmap with checksum {} not found", checksum),
                status: "404".to_string(),
                weekly_id: None,
                added_beatmaps: None,
            }));
        }

        // Récupérer l'ID interne et l'osu_id de la beatmap
        let (beatmap_id, osu_id) = BeatmapShort::find_id_and_osu_id_by_checksum(pool, checksum)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

        beatmap_ids.push(beatmap_id);
        osu_ids.push(osu_id);
    }

    // 3. Créer la semaine
    let weekly = Weekly::insert(pool, &week_name)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 4. Ajouter toutes les beatmaps à la semaine
    let weekly_maps = WeeklyMaps::bulk_insert(pool, &beatmap_ids, weekly.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(SetupWeekResponse {
        message: "Week created successfully".to_string(),
        status: "200".to_string(),
        weekly_id: Some(weekly.id),
        added_beatmaps: Some(beatmap_ids),
    }))
}

async fn fetch_osu_file(osu_id: i32) -> Result<String, Box<dyn std::error::Error>> {
    let url = format!("https://osu.ppy.sh/osu/{}", osu_id);
    let response = reqwest::get(&url).await?;

    if !response.status().is_success() {
        return Err(format!(
            "Failed to fetch osu file for ID {}: {}",
            osu_id,
            response.status()
        )
        .into());
    }

    let content = response.text().await?;
    Ok(content)
}
