use super::types::{ScoreResponse, UserInfo, PerformanceInfo, HitsInfo};
use db::scores::score_display::ScoreDisplay;

impl From<ScoreDisplay> for ScoreResponse {
    fn from(score: ScoreDisplay) -> Self {
        Self {
            id: score.id,
            user: UserInfo {
                id: score.user_id,
                username: score.username.unwrap_or_else(|| format!("User_{}", score.user_id)),
            },
            beatmap_id: score.beatmap_id,
            rate: score.rate,
            mods: score.mods,
            rank: score.rank,
            created_at: score.created_at,
            performance: PerformanceInfo {
                score: score.score,
                accuracy: score.accuracy,
                max_combo: score.max_combo,
                perfect: score.perfect,
                pause_count: score.pause_count,
            },
            hits: HitsInfo {
                count_300: score.count_300,
                count_100: score.count_100,
                count_50: score.count_50,
                count_miss: score.count_miss,
                count_katu: score.count_katu,
                count_geki: score.count_geki,
            },
        }
    }
}
