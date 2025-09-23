use axum::{
    extract::{ConnectInfo, Request},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::time::sleep;
use tracing::{info, warn};

/// Structure pour tracker les tentatives d'attaque par IP
#[derive(Debug, Clone)]
pub struct AttackAttempt {
    pub count: u32,
    pub first_attempt: Instant,
    pub last_attempt: Instant,
    pub banned_until: Option<Instant>,
}

// Store global pour tracker les IPs suspectes
lazy_static::lazy_static! {
    static ref ATTACK_TRACKER: Arc<Mutex<HashMap<String, AttackAttempt>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

/// Patterns d'attaques communes que les script kiddies utilisent
const ATTACK_PATTERNS: &[&str] = &[
    // SQL Injection classique
    "' OR 1=1",
    "' OR 1=1--",
    "' OR 1=1#",
    "' OR 1=1/*",
    "' OR '1'='1",
    "' OR 'x'='x",
    "' OR 'a'='a",
    "' UNION SELECT",
    "' UNION ALL SELECT",
    "' DROP TABLE",
    "' DELETE FROM",
    "' INSERT INTO",
    "' UPDATE SET",
    "admin'--",
    "admin'#",
    "' AND 1=1--",
    "' AND 1=2--",
    "'; --",
    "'; #",
    "';--",
    "';#",
    "' or 1=1--",
    "' or '1'='1",
    "' or 'a'='a",
    " OR 1=1",
    " OR '1'='1",
    "1' OR '1'='1",
    "1' OR 1=1--",
    // XSS basique
    "<script>",
    "</script>",
    "javascript:",
    "onload=",
    "onerror=",
    "onclick=",
    "onmouseover=",
    "alert(",
    "document.cookie",
    // Path Traversal
    "../",
    "..\\",
    "....//",
    "....\\\\",
    "/etc/passwd",
    "/etc/shadow",
    "\\windows\\system32",
    // Command Injection
    "; cat /etc/passwd",
    "| cat /etc/passwd",
    "&& cat /etc/passwd",
    "; ls -la",
    "| ls -la",
    "&& ls -la",
    "; whoami",
    "| whoami",
    "&& whoami",
    // PHP-specific (pour les vrais noobs)
    "<?php",
    "<?=",
    "eval(",
    "system(",
    "exec(",
    "shell_exec(",
    "passthru(",
    "file_get_contents(",
    "include(",
    "require(",
    // Autres tentatives classiques
    "null",
    "undefined",
    "NaN",
    "Infinity",
    "/admin",
    "/administrator",
    "/wp-admin",
    "/phpmyadmin",
    "/.env",
    "/config.php",
    "/database.php",
];

/// Headers suspects utilisés par les outils automatisés
const SUSPICIOUS_USER_AGENTS: &[&str] = &[
    "sqlmap",
    "nmap",
    "nikto",
    "dirb",
    "dirbuster",
    "gobuster",
    "wfuzz",
    "burp",
    "owasp",
    "python-requests",
    "curl/7", // Version trop basique
    "wget",
    "masscan",
    "nessus",
    "openvas",
    "acunetix",
];

/// Vérifie si une IP est actuellement bannie
fn is_ip_banned(ip: &str) -> bool {
    let tracker = ATTACK_TRACKER.lock().unwrap();
    if let Some(attempt) = tracker.get(ip) {
        if let Some(banned_until) = attempt.banned_until {
            return Instant::now() < banned_until;
        }
    }
    false
}

/// Enregistre une tentative d'attaque et retourne le délai à appliquer
fn record_attack_attempt(ip: &str) -> Duration {
    let mut tracker = ATTACK_TRACKER.lock().unwrap();
    let now = Instant::now();

    let attempt = tracker.entry(ip.to_string()).or_insert(AttackAttempt {
        count: 0,
        first_attempt: now,
        last_attempt: now,
        banned_until: None,
    });

    attempt.count += 1;
    attempt.last_attempt = now;

    // Calculer le délai selon le nombre de tentatives
    let delay = match attempt.count {
        1 => Duration::from_secs(5),  // 5 secondes pour la première
        2 => Duration::from_secs(15), // 15 secondes pour la deuxième
        3 => {
            // Ban de 1 heure après la 3ème tentative
            attempt.banned_until = Some(now + Duration::from_secs(3600));
            Duration::from_secs(30) // 30 secondes avant de dire qu'il est banni
        }
        _ => Duration::from_secs(60), // 1 minute pour les suivantes (s'il insiste)
    };

    warn!(
        "🚨 Attack attempt #{} from IP: {} - Applying {}s delay",
        attempt.count,
        ip,
        delay.as_secs()
    );

    delay
}

/// Vérifie si la requête contient des patterns d'attaque
fn contains_attack_patterns(request: &Request) -> bool {
    // Vérifier l'URL complète
    let uri = request.uri().to_string().to_lowercase();
    for pattern in ATTACK_PATTERNS {
        if uri.contains(&pattern.to_lowercase()) {
            return true;
        }
    }

    // Vérifier les query parameters (avec URL decode)
    if let Some(query) = request.uri().query() {
        // Décoder l'URL pour détecter les patterns encodés
        let decoded_query = urlencoding::decode(query).unwrap_or_else(|_| query.into());
        let query_lower = decoded_query.to_lowercase();

        // Debug log pour voir ce qu'on analyse
        tracing::debug!(
            "🔍 Analyzing query: '{}' (decoded: '{}')",
            query,
            decoded_query
        );

        for pattern in ATTACK_PATTERNS {
            let pattern_lower = pattern.to_lowercase();
            if query_lower.contains(&pattern_lower) {
                tracing::warn!(
                    "🚨 Attack pattern detected: '{}' in query: '{}'",
                    pattern,
                    decoded_query
                );
                return true;
            }
        }

        // Vérifications supplémentaires pour les injections SQL courantes
        if query_lower.contains("'")
            && (query_lower.contains("or")
                || query_lower.contains("and")
                || query_lower.contains("union")
                || query_lower.contains("select")
                || query_lower.contains("drop")
                || query_lower.contains("--")
                || query_lower.contains("#"))
        {
            tracing::warn!(
                "🚨 SQL injection pattern detected in query: '{}'",
                decoded_query
            );
            return true;
        }
    }

    // Vérifier les headers
    let headers = request.headers();

    // User-Agent suspect
    if let Some(user_agent) = headers.get("user-agent") {
        if let Ok(ua_str) = user_agent.to_str() {
            let ua_lower = ua_str.to_lowercase();
            for suspicious_ua in SUSPICIOUS_USER_AGENTS {
                if ua_lower.contains(suspicious_ua) {
                    return true;
                }
            }
        }
    }

    // Referer suspect (souvent vide ou bizarre chez les script kiddies)
    if let Some(referer) = headers.get("referer") {
        if let Ok(ref_str) = referer.to_str() {
            let ref_lower = ref_str.to_lowercase();
            for pattern in ATTACK_PATTERNS {
                if ref_lower.contains(&pattern.to_lowercase()) {
                    return true;
                }
            }
        }
    }

    false
}

/// Middleware principal anti-script-kiddies
pub async fn anti_kiddie_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let ip = addr.ip().to_string();

    // Vérifier si l'IP est déjà bannie
    if is_ip_banned(&ip) {
        warn!("🔒 Banned IP {} tried to access: {}", ip, request.uri());

        // Faire attendre 30 secondes même pour les IPs bannies (pour les faire chier)
        sleep(Duration::from_secs(30)).await;

        return Ok(Response::builder()
            .status(StatusCode::FORBIDDEN)
            .header("Content-Type", "application/json")
            .body(format!(
                r#"{{"error":"Access denied","message":"Your IP {} has been banned for suspicious activity","code":"BANNED","timestamp":"{}"}}"#,
                ip,
                chrono::Utc::now().to_rfc3339()
            ).into())
            .unwrap());
    }

    // Vérifier si la requête contient des patterns d'attaque
    if contains_attack_patterns(&request) {
        let delay = record_attack_attempt(&ip);

        info!(
            "🎯 Script kiddie detected! IP: {}, URI: {}, User-Agent: {:?}",
            ip,
            request.uri(),
            request.headers().get("user-agent")
        );

        // Faire attendre le script kiddie (le délai augmente à chaque tentative)
        sleep(delay).await;

        // Réponse troll pour les faire croire qu'ils progressent
        let troll_responses = [
            r#"{"error":"Invalid parameter","hint":"Try adding ?debug=true"}"#,
            r#"{"error":"Access denied","message":"Authentication required","login_url":"/admin/login"}"#,
            r#"{"error":"Database connection failed","retry_after":300}"#,
            r#"{"error":"Rate limit exceeded","message":"Too many requests"}"#,
            r#"{"status":"processing","message":"Please wait...","eta":"5 minutes"}"#,
        ];

        let tracker = ATTACK_TRACKER.lock().unwrap();
        let attempt_count = tracker.get(&ip).map(|a| a.count).unwrap_or(1);
        let response_index = (attempt_count as usize - 1) % troll_responses.len();

        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("Content-Type", "application/json")
            .header("X-Your-IP", &ip) // On leur renvoie leur IP comme demandé 😈
            .header("X-Attempt-Count", attempt_count.to_string())
            .body(troll_responses[response_index].into())
            .unwrap());
    }

    // Si tout va bien, continuer normalement
    Ok(next.run(request).await)
}

/// Fonction pour nettoyer périodiquement les anciennes entrées
pub async fn cleanup_old_entries() {
    let mut interval = tokio::time::interval(Duration::from_secs(3600)); // Nettoyage toutes les heures

    loop {
        interval.tick().await;

        let mut tracker = ATTACK_TRACKER.lock().unwrap();
        let now = Instant::now();

        // Supprimer les entrées plus anciennes que 24h (sauf si bannies)
        tracker.retain(|ip, attempt| {
            if let Some(banned_until) = attempt.banned_until {
                if now > banned_until {
                    info!("🔓 Unbanning IP: {}", ip);
                    false // Supprimer l'entrée, le ban est fini
                } else {
                    true // Garder, encore banni
                }
            } else {
                // Garder seulement si moins de 24h
                now.duration_since(attempt.last_attempt) < Duration::from_secs(86400)
            }
        });

        info!(
            "🧹 Cleaned up attack tracker. Active entries: {}",
            tracker.len()
        );
    }
}

/// Fonction pour obtenir des stats sur les attaques (pour le debug/monitoring)
pub fn get_attack_stats() -> HashMap<String, AttackAttempt> {
    ATTACK_TRACKER.lock().unwrap().clone()
}
