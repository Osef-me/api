use crate::middleware::anti_kiddie::get_attack_stats;
use axum::{Json, http::StatusCode};
use serde_json::{Value, json};
use std::time::Instant;

/// Endpoint pour voir les statistiques des tentatives d'attaque (debug uniquement)
pub async fn security_stats() -> Result<Json<Value>, StatusCode> {
    let stats = get_attack_stats();
    let now = Instant::now();

    let mut formatted_stats = Vec::new();

    for (ip, attempt) in stats {
        let time_since_first = now.duration_since(attempt.first_attempt).as_secs();
        let time_since_last = now.duration_since(attempt.last_attempt).as_secs();

        let ban_status = if let Some(banned_until) = attempt.banned_until {
            if now < banned_until {
                let remaining = banned_until.duration_since(now).as_secs();
                format!("BANNED ({}s remaining)", remaining)
            } else {
                "BAN_EXPIRED".to_string()
            }
        } else {
            "NOT_BANNED".to_string()
        };

        formatted_stats.push(json!({
            "ip": ip,
            "attempt_count": attempt.count,
            "first_attempt_ago_seconds": time_since_first,
            "last_attempt_ago_seconds": time_since_last,
            "ban_status": ban_status
        }));
    }

    // Trier par nombre de tentatives (les plus actifs en premier)
    formatted_stats.sort_by(|a, b| {
        b["attempt_count"]
            .as_u64()
            .unwrap_or(0)
            .cmp(&a["attempt_count"].as_u64().unwrap_or(0))
    });

    Ok(Json(json!({
        "message": "🛡️ Security Statistics",
        "total_tracked_ips": formatted_stats.len(),
        "attacks": formatted_stats,
        "legend": {
            "attempt_count": "Number of suspicious requests from this IP",
            "ban_status": "Current ban status (BANNED/NOT_BANNED/BAN_EXPIRED)",
            "first_attempt_ago_seconds": "Seconds since first suspicious activity",
            "last_attempt_ago_seconds": "Seconds since last suspicious activity"
        }
    })))
}
