use axum::{
    body::Body,
    extract::Request,
    http::{HeaderMap, Method, StatusCode, Uri},
    middleware::Next,
    response::Response,
};
use moka::future::Cache;
use serde::Serialize;
use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tracing::{debug, info, warn};

/// Structure pour une réponse mise en cache
#[derive(Debug, Clone)]
struct CachedResponse {
    body: Vec<u8>,
    headers: HeaderMap,
    status: StatusCode,
    created_at: Instant,
}

/// Statistiques du cache
#[derive(Debug, Clone, Serialize)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub total_requests: u64,
    pub hit_rate: f64,
    pub cache_sizes: HashMap<String, u64>,
}

/// Configuration de cache pour différents types de routes
#[derive(Debug, Clone)]
struct CacheConfig {
    ttl: Duration,
    max_capacity: u64,
    enabled: bool,
}

impl CacheConfig {
    fn new(ttl_seconds: u64, max_capacity: u64) -> Self {
        Self {
            ttl: Duration::from_secs(ttl_seconds),
            max_capacity,
            enabled: true,
        }
    }
}

/// Store principal du cache avec différents buckets
pub struct CacheStore {
    // Cache pour les stats globales (TTL long)
    global_stats: Cache<String, CachedResponse>,

    // Cache pour les requêtes filtrées (TTL moyen)
    filtered_queries: Cache<String, CachedResponse>,

    // Cache pour les beatmaps individuels (TTL long)
    individual_beatmaps: Cache<String, CachedResponse>,

    // Cache pour les statuts pending (TTL court)
    pending_status: Cache<String, CachedResponse>,

    // Statistiques
    stats: Arc<Mutex<CacheStats>>,
}

impl CacheStore {
    pub fn new() -> Self {
        Self {
            global_stats: Cache::builder()
                .time_to_live(Duration::from_secs(600)) // 10 minutes
                .max_capacity(10)
                .build(),

            filtered_queries: Cache::builder()
                .time_to_live(Duration::from_secs(300)) // 5 minutes
                .max_capacity(1000)
                .build(),

            individual_beatmaps: Cache::builder()
                .time_to_live(Duration::from_secs(1800)) // 30 minutes
                .max_capacity(5000)
                .build(),

            pending_status: Cache::builder()
                .time_to_live(Duration::from_secs(30)) // 30 secondes
                .max_capacity(1000)
                .build(),

            stats: Arc::new(Mutex::new(CacheStats {
                hits: 0,
                misses: 0,
                total_requests: 0,
                hit_rate: 0.0,
                cache_sizes: HashMap::new(),
            })),
        }
    }

    /// Récupère les statistiques du cache
    pub fn get_stats(&self) -> CacheStats {
        let mut stats = self.stats.lock().unwrap();
        stats
            .cache_sizes
            .insert("global_stats".to_string(), self.global_stats.entry_count());
        stats.cache_sizes.insert(
            "filtered_queries".to_string(),
            self.filtered_queries.entry_count(),
        );
        stats.cache_sizes.insert(
            "individual_beatmaps".to_string(),
            self.individual_beatmaps.entry_count(),
        );
        stats.cache_sizes.insert(
            "pending_status".to_string(),
            self.pending_status.entry_count(),
        );

        if stats.total_requests > 0 {
            stats.hit_rate = (stats.hits as f64 / stats.total_requests as f64) * 100.0;
        }

        stats.clone()
    }

    /// Enregistre un hit dans les statistiques
    fn record_hit(&self) {
        let mut stats = self.stats.lock().unwrap();
        stats.hits += 1;
        stats.total_requests += 1;
    }

    /// Enregistre un miss dans les statistiques
    fn record_miss(&self) {
        let mut stats = self.stats.lock().unwrap();
        stats.misses += 1;
        stats.total_requests += 1;
    }

    /// Détermine le type de cache selon la route
    fn get_cache_type(&self, path: &str) -> CacheType {
        if path == "/api/beatmap/count" {
            CacheType::GlobalStats
        } else if path == "/api/beatmap" {
            CacheType::FilteredQueries
        } else if path.starts_with("/api/beatmapset/") {
            CacheType::IndividualBeatmaps
        } else if path.starts_with("/api/pending_beatmap/status/") {
            CacheType::PendingStatus
        } else {
            CacheType::None
        }
    }

    /// Récupère une réponse du cache approprié
    async fn get(&self, cache_type: CacheType, key: &str) -> Option<CachedResponse> {
        match cache_type {
            CacheType::GlobalStats => self.global_stats.get(key).await,
            CacheType::FilteredQueries => self.filtered_queries.get(key).await,
            CacheType::IndividualBeatmaps => self.individual_beatmaps.get(key).await,
            CacheType::PendingStatus => self.pending_status.get(key).await,
            CacheType::None => None,
        }
    }

    /// Stocke une réponse dans le cache approprié
    async fn insert(&self, cache_type: CacheType, key: String, response: CachedResponse) {
        match cache_type {
            CacheType::GlobalStats => self.global_stats.insert(key, response).await,
            CacheType::FilteredQueries => self.filtered_queries.insert(key, response).await,
            CacheType::IndividualBeatmaps => self.individual_beatmaps.insert(key, response).await,
            CacheType::PendingStatus => self.pending_status.insert(key, response).await,
            CacheType::None => {}
        }
    }

    /// Invalide le cache pour les routes de modification
    pub async fn invalidate_related_caches(&self, path: &str) {
        if path.contains("/api/beatmap/") && !path.contains("/get/") {
            // Si c'est une modification de beatmap, invalider les caches liés
            info!(
                "🗑️ Invalidating beatmap-related caches due to modification at: {}",
                path
            );
            self.global_stats.invalidate_all();
            self.filtered_queries.invalidate_all();
            // On garde les beatmaps individuels car ils changent moins souvent
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum CacheType {
    GlobalStats,
    FilteredQueries,
    IndividualBeatmaps,
    PendingStatus,
    None,
}

// Store global du cache
lazy_static::lazy_static! {
    static ref CACHE_STORE: CacheStore = CacheStore::new();
}

/// Génère une clé de cache basée sur l'URI et les query parameters
fn generate_cache_key(uri: &Uri) -> String {
    let path = uri.path();

    if let Some(query) = uri.query() {
        // Pour les requêtes avec paramètres, hasher les paramètres pour une clé stable
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        query.hash(&mut hasher);
        format!("{}:{:x}", path, hasher.finish())
    } else {
        path.to_string()
    }
}

/// Vérifie si une route doit être mise en cache
fn should_cache_route(method: &Method, path: &str) -> bool {
    // Seulement les GET requests
    if method != Method::GET {
        return false;
    }

    // Routes à cache
    match path {
        p if p == "/api/beatmap/count" => true,
        p if p == "/api/beatmap" => true,
        p if p.starts_with("/api/beatmapset/") => true,
        p if p.starts_with("/api/pending_beatmap/status/") => true,
        _ => false,
    }
}

/// Vérifie si une route ne doit jamais être cachée (ex: random)
fn should_never_cache(path: &str) -> bool {
    path.contains("/random") || path.contains("/health")
}

/// Convertit une Response en CachedResponse
async fn response_to_cached(
    response: Response,
) -> Result<CachedResponse, Box<dyn std::error::Error + Send + Sync>> {
    let status = response.status();
    let headers = response.headers().clone();

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await?;

    Ok(CachedResponse {
        body: body_bytes.to_vec(),
        headers,
        status,
        created_at: Instant::now(),
    })
}

/// Convertit une CachedResponse en Response
fn cached_to_response(cached: CachedResponse) -> Response {
    let mut response = Response::builder()
        .status(cached.status)
        .body(Body::from(cached.body))
        .unwrap();

    *response.headers_mut() = cached.headers;

    // Ajouter des headers de debug
    response
        .headers_mut()
        .insert("X-Cache-Status", "HIT".parse().unwrap());
    response.headers_mut().insert(
        "X-Cache-Age",
        cached
            .created_at
            .elapsed()
            .as_secs()
            .to_string()
            .parse()
            .unwrap(),
    );

    response
}

/// Middleware principal de cache
pub async fn cache_middleware(request: Request, next: Next) -> Result<Response, StatusCode> {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let path = uri.path().to_string();

    // Skip le cache pour les routes qui ne doivent jamais être cachées
    if should_never_cache(&path) {
        debug!("🚫 Skipping cache for never-cache route: {}", path);
        return Ok(next.run(request).await);
    }

    // Pour les routes de modification, invalider les caches liés
    if method != Method::GET {
        CACHE_STORE.invalidate_related_caches(&path).await;
        return Ok(next.run(request).await);
    }

    // Vérifier si cette route doit être mise en cache
    if !should_cache_route(&method, &path) {
        debug!("🔄 Route not cacheable: {} {}", method, path);
        return Ok(next.run(request).await);
    }

    let cache_key = generate_cache_key(&uri);
    let cache_type = CACHE_STORE.get_cache_type(&path);

    // Tentative de récupération depuis le cache
    if let Some(cached_response) = CACHE_STORE.get(cache_type, &cache_key).await {
        CACHE_STORE.record_hit();
        debug!("✅ Cache HIT for: {} (key: {})", path, cache_key);
        return Ok(cached_to_response(cached_response));
    }

    // Cache MISS - exécuter la requête
    CACHE_STORE.record_miss();
    debug!("❌ Cache MISS for: {} (key: {})", path, cache_key);

    let response = next.run(request).await;

    // Seulement cacher les réponses success (2xx)
    if response.status().is_success() {
        match response_to_cached(response).await {
            Ok(cached_response) => {
                // Cloner pour le cache
                let cached_clone = cached_response.clone();

                // Stocker dans le cache de manière asynchrone
                tokio::spawn(async move {
                    CACHE_STORE
                        .insert(cache_type, cache_key.clone(), cached_clone)
                        .await;
                    debug!("💾 Cached response for: {} (key: {})", path, cache_key);
                });

                // Retourner la réponse avec header MISS
                let mut response = cached_to_response(cached_response);
                response
                    .headers_mut()
                    .insert("X-Cache-Status", "MISS".parse().unwrap());
                Ok(response)
            }
            Err(e) => {
                warn!("Failed to cache response for {}: {}", path, e);
                // En cas d'erreur, on retourne une réponse d'erreur basique
                Ok(Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::from("Cache serialization error"))
                    .unwrap())
            }
        }
    } else {
        // Ne pas cacher les erreurs
        Ok(response)
    }
}

/// Fonction pour obtenir les statistiques du cache
pub fn get_cache_stats() -> CacheStats {
    CACHE_STORE.get_stats()
}

/// Fonction pour pré-chauffer le cache avec les routes critiques
pub async fn warm_cache() {
    info!("🔥 Starting cache warming...");

    // Ici on pourrait faire des requêtes internes pour pré-chauffer
    // Par exemple: appeler /api/beatmap/count

    info!("🔥 Cache warming completed");
}

/// Fonction pour nettoyer périodiquement les statistiques
pub async fn cleanup_cache_stats() {
    let mut interval = tokio::time::interval(Duration::from_secs(3600)); // Toutes les heures

    loop {
        interval.tick().await;

        // Reset les statistiques si elles deviennent trop grandes
        let stats = CACHE_STORE.get_stats();
        if stats.total_requests > 1_000_000 {
            let mut stats_guard = CACHE_STORE.stats.lock().unwrap();
            stats_guard.hits = stats_guard.hits / 2;
            stats_guard.misses = stats_guard.misses / 2;
            stats_guard.total_requests = stats_guard.total_requests / 2;
            info!("🧹 Cache stats reset to prevent overflow");
        }
    }
}
