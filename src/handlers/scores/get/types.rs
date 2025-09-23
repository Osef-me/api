use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ScoreResponse {
    // Informations de base
    pub id: i32,
    pub user: UserInfo,
    pub beatmap_id: i32,
    pub rate: BigDecimal,
    pub mods: i64,
    pub rank: String,
    pub created_at: Option<NaiveDateTime>,
    
    // Performance
    pub performance: PerformanceInfo,
    
    // Hits
    pub hits: HitsInfo,
}

#[derive(Debug, Clone, Serialize)]
pub struct UserInfo {
    pub id: i64,
    pub username: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PerformanceInfo {
    pub score: i32,
    pub accuracy: BigDecimal,
    pub max_combo: i32,
    pub perfect: bool,
    pub pause_count: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct HitsInfo {
    pub count_300: i32,
    pub count_100: i32,
    pub count_50: i32,
    pub count_miss: i32,
    pub count_katu: i32,
    pub count_geki: i32,
}
