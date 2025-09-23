use axum::{Json, extract::State, http::StatusCode};
use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use db::db::DatabaseManager;
use db::scores::metadata::ScoreMetadata;
use db::scores::replay::Replay;
use db::scores::score::Score;
use db::short::beatmap::query::by_checksum;
use db::extended::beatmap_rate::query::by_hash as beatmap_rate_by_hash;
use db::users::ban::Ban;
use db::users::device_token::DeviceToken;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct SubmitScoreRequest {
    pub token: Uuid,
    pub hwid: String,
    pub hash: String,           // Hash unique de la beatmap fourni par le client
    pub replay: Option<String>, // Base64 encoded replay data (optional)
    pub score: ScoreData,
    pub score_metadata: ScoreMetadataData,
}

#[derive(Deserialize)]
pub struct ScoreData {
    pub hash: String,           // Hash unique du score fourni par le client
    pub mods: i64,
    pub rank: String,
}

#[derive(Deserialize)]
pub struct ScoreMetadataData {
    pub skin: Option<String>,
    pub pause_count: i32,
    pub started_at: NaiveDateTime,
    pub ended_at: NaiveDateTime,
    pub time_paused: i32, // used for anticheat in seconds
    pub score: i32,
    pub accuracy: BigDecimal,
    pub max_combo: i32,
    pub perfect: bool,
    pub count_300: i32,
    pub count_100: i32,
    pub count_50: i32,
    pub count_miss: i32,
    pub count_katu: i32,
    pub count_geki: i32,
}

#[derive(Serialize)]
pub struct SubmitScoreResponse {
    pub message: String,
    pub status: String,
    pub score_id: Option<i32>,
}

pub async fn handler(
    State(db): State<DatabaseManager>,
    Json(payload): Json<SubmitScoreRequest>,
) -> Result<Json<SubmitScoreResponse>, StatusCode> {
    // 1. Validation basique (token + doublons)
    let device_token = DeviceToken::find_by_token(db.get_pool(), payload.token)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    println!("je crash pas la");
    println!("Avant exists_by_hash");
    let exists_result = Score::exists_by_hash(db.get_pool(), &payload.score.hash).await;
    println!("Après exists_by_hash: {:?}", exists_result);
    
    if exists_result.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? {
        return Ok(Json(SubmitScoreResponse {
            message: "Score already exists".to_string(),
            status: "409".to_string(),
            score_id: None,
        }));
    }
    println!("la non plus");
    // 2. Sauvegarde rapide (replay + métadonnées + score pending)
    let user_id = device_token.discord_id.ok_or(StatusCode::UNAUTHORIZED)?;

    let replay_id = if let Some(replay_data) = &payload.replay {
        let replay_path = save_replay(replay_data, &payload.score.hash)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let replay = Replay::insert(db.get_pool(), &payload.score.hash, &replay_path)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Some(replay.id)
    } else {
        None
    };
    println!("la non plus 2");
    let score_metadata = ScoreMetadata::insert(
        db.get_pool(),
        payload.score_metadata.skin.as_deref(),
        payload.score_metadata.pause_count,
        payload.score_metadata.started_at,
        payload.score_metadata.ended_at,
        payload.score_metadata.time_paused,
        payload.score_metadata.score,
        payload.score_metadata.accuracy.clone(),
        payload.score_metadata.max_combo,
        payload.score_metadata.perfect,
        payload.score_metadata.count_300,
        payload.score_metadata.count_100,
        payload.score_metadata.count_50,
        payload.score_metadata.count_miss,
        payload.score_metadata.count_katu,
        payload.score_metadata.count_geki,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    println!("la non plus 3");
    
    // Récupérer la beatmap par checksum
    println!("Recherche de la beatmap avec hash: {}", payload.hash);
    let beatmap_info = by_checksum::find_id_and_osu_id_by_checksum(db.get_pool(), &payload.hash)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let (beatmap_id, rate) = match beatmap_info {
        Some((id, _osu_id)) => {
            println!("Beatmap trouvée avec ID: {}", id);
            (id, BigDecimal::from(1)) // Rate par défaut pour les beatmaps normales
        },
        None => {
            println!("Aucune beatmap trouvée avec ce hash, recherche dans beatmap_rate");
            // Chercher dans beatmap_rate
            let beatmap_rates = beatmap_rate_by_hash::find_by_hash(db.get_pool(), &payload.hash)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            
            if let Some(beatmap_rate) = beatmap_rates.first() {
                println!("Beatmap trouvée dans beatmap_rate avec ID: {} et rate: {}", beatmap_rate.beatmap_id, beatmap_rate.rate);
                (beatmap_rate.beatmap_id, beatmap_rate.rate.clone())
            } else {
                println!("Aucune beatmap trouvée nulle part avec ce hash");
                return Err(StatusCode::NOT_FOUND);
            }
        }
    };
    
    println!("Avant Score::insert avec beatmap_id: {} et rate: {}", beatmap_id, rate);
    let score_result = Score::insert(
        db.get_pool(),
        user_id,
        beatmap_id,
        score_metadata.id,
        replay_id,
        rate, // rate récupéré de la beatmap ou beatmap_rate
        Some(&payload.hwid),
        payload.score.mods,
        &payload.score.hash,
        &payload.score.rank,
        "pending",
    )
    .await;
    println!("Après Score::insert: {:?}", score_result);
    let score = score_result.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    println!("la non plus 4");
    Ok(Json(SubmitScoreResponse {
        message: "Score submitted successfully".to_string(),
        status: "200".to_string(),
        score_id: Some(score.id),
    }))
}

async fn save_replay(
    replay_base64: &str,
    score_hash: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    // Décoder le base64
    use base64::{Engine as _, engine::general_purpose};
    let replay_data = general_purpose::STANDARD.decode(replay_base64)?;

    // Créer le dossier public/replay s'il n'existe pas
    let replay_dir = Path::new("public/replay");
    if !replay_dir.exists() {
        fs::create_dir_all(replay_dir).await?;
    }

    // Créer le chemin du fichier
    let replay_path = replay_dir.join(format!("{}.or", score_hash));

    // Sauvegarder le fichier
    fs::write(&replay_path, replay_data).await?;

    Ok(replay_path.to_string_lossy().to_string())
}
