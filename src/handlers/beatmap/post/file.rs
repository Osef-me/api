use axum::{
    body::Body,
    extract::{Json, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use db::db::DatabaseManager;
use db::extended::beatmap_rate::query::by_hash_and_rate;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct FileRequest {
    pub hash: String,
    pub rate: f64,
}

#[axum::debug_handler]
pub async fn handler(
    State(db): State<DatabaseManager>,
    Json(request): Json<FileRequest>,
) -> impl IntoResponse {
    let hash = request.hash;
    let rate = request.rate;
    
    // Chercher directement la beatmap avec hash et rate en une seule requête
    println!("Searching for beatmap with hash: {} and rate: {}", hash, rate);
    let (_, osu_id, rate_hash) = match by_hash_and_rate::find_beatmap_info_by_hash_and_rate(db.get_pool(), &hash, rate).await {
        Ok(Some(info)) => info,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    
    // Construire le chemin du fichier avec le hash de la beatmap_rate
    let file_path = PathBuf::from("public")
        .join("beatmap")
        .join(osu_id.to_string())
        .join(format!("{}.br", rate_hash));
    
    // Vérifier que le fichier existe
    println!("File path: {}", file_path.display());

    if !file_path.exists() {
        return StatusCode::NOT_FOUND.into_response();
    }
    
    // Lire le fichier
    let file_content = match tokio::fs::read(&file_path).await {
        Ok(content) => content,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    
    // Créer la réponse avec les bons headers
    match Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}.br\"", rate_hash))
        .body(Body::from(file_content))
    {
        Ok(response) => response.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
