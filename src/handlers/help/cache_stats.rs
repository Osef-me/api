use crate::middleware::cache::get_cache_stats;
use axum::{Json, http::StatusCode};
use serde_json::{Value, json};

/// Endpoint pour voir les statistiques du cache
pub async fn cache_stats() -> Result<Json<Value>, StatusCode> {
    let stats = get_cache_stats();

    // Calculer des métriques supplémentaires
    let total_cached_items: u64 = stats.cache_sizes.values().sum();
    let memory_estimation = total_cached_items * 1024; // Estimation simple: 1KB par item

    let performance_rating = if stats.hit_rate >= 80.0 {
        "🚀 Excellent"
    } else if stats.hit_rate >= 60.0 {
        "✅ Good"
    } else if stats.hit_rate >= 40.0 {
        "⚠️ Average"
    } else if stats.hit_rate >= 20.0 {
        "🟡 Poor"
    } else {
        "🔴 Critical"
    };

    Ok(Json(json!({
        "message": "💾 Cache Statistics",
        "performance": {
            "hit_rate_percentage": format!("{:.2}%", stats.hit_rate),
            "performance_rating": performance_rating,
            "total_requests": stats.total_requests,
            "cache_hits": stats.hits,
            "cache_misses": stats.misses
        },
        "cache_buckets": {
            "global_stats": {
                "size": stats.cache_sizes.get("global_stats").unwrap_or(&0),
                "description": "Global beatmap statistics (TTL: 10min)",
                "ttl_seconds": 600
            },
            "filtered_queries": {
                "size": stats.cache_sizes.get("filtered_queries").unwrap_or(&0),
                "description": "Filtered beatmap queries (TTL: 5min)",
                "ttl_seconds": 300
            },
            "individual_beatmaps": {
                "size": stats.cache_sizes.get("individual_beatmaps").unwrap_or(&0),
                "description": "Individual beatmap data (TTL: 30min)",
                "ttl_seconds": 1800
            },
            "pending_status": {
                "size": stats.cache_sizes.get("pending_status").unwrap_or(&0),
                "description": "Pending beatmap status (TTL: 30sec)",
                "ttl_seconds": 30
            }
        },
        "memory": {
            "total_cached_items": total_cached_items,
            "estimated_memory_usage_bytes": memory_estimation,
            "estimated_memory_usage_mb": format!("{:.2} MB", memory_estimation as f64 / 1024.0 / 1024.0)
        },
        "recommendations": generate_recommendations(&stats),
        "cache_efficiency": {
            "bytes_saved_estimation": stats.hits * 2048, // Estimation: 2KB par hit sauvé
            "database_queries_avoided": stats.hits,
            "response_time_improvement": "~50-90% faster for cached responses"
        }
    })))
}

/// Génère des recommandations basées sur les stats du cache
fn generate_recommendations(stats: &crate::middleware::cache::CacheStats) -> Vec<String> {
    let mut recommendations = Vec::new();

    if stats.hit_rate < 30.0 && stats.total_requests > 100 {
        recommendations.push("🔴 Hit rate is very low. Consider increasing TTL values or checking if routes are properly cacheable.".to_string());
    }

    if stats.hit_rate > 90.0 && stats.total_requests > 1000 {
        recommendations.push("🚀 Excellent hit rate! Cache is working optimally.".to_string());
    }

    let total_items: u64 = stats.cache_sizes.values().sum();
    if total_items > 10000 {
        recommendations.push(
            "⚠️ High number of cached items. Consider reducing TTL or max capacity to save memory."
                .to_string(),
        );
    }

    if total_items < 10 && stats.total_requests > 100 {
        recommendations.push("🟡 Very few items cached despite many requests. Check if caching logic is working correctly.".to_string());
    }

    if let Some(&global_size) = stats.cache_sizes.get("global_stats") {
        if global_size == 0 && stats.total_requests > 50 {
            recommendations.push(
                "💡 Global stats cache is empty. Consider pre-warming with /api/beatmap/count."
                    .to_string(),
            );
        }
    }

    if recommendations.is_empty() {
        recommendations.push("✅ Cache performance looks good! Keep monitoring.".to_string());
    }

    recommendations
}
