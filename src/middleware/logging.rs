use axum::{
    body::Body,
    http::Request,
    middleware::{self, Next},
    response::Response,
};
use axum::extract::MatchedPath;
use axum::http::header;
use std::time::Instant;
use tracing::{error, info, warn};

pub async fn track_execution_time(req: Request<Body>, next: Next) -> Response {
    let path = req.uri().path().to_owned();
    let method = req.method().clone();

    // Enrichissement: collecter informations utiles sans exposer de données sensibles
    let version = format!("{:?}", req.version());
    let matched_endpoint = req
        .extensions()
        .get::<MatchedPath>()
        .map(|p| p.as_str().to_owned())
        .unwrap_or_else(|| path.clone());

    let headers = req.headers();
    let user_agent: String = headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_owned())
        .unwrap_or_else(|| "-".to_string());
    let x_real_ip: String = headers
        .get("x-real-ip")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_owned())
        .unwrap_or_default();
    let x_forwarded_for: String = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_owned())
        .unwrap_or_default();
    let client_ip: String = if !x_real_ip.is_empty() {
        x_real_ip.clone()
    } else if !x_forwarded_for.is_empty() {
        x_forwarded_for.clone()
    } else {
        "-".to_string()
    };

    // Redaction: n'expose pas les valeurs sensibles, seulement leur présence
    let has_auth = headers.get(header::AUTHORIZATION).is_some();
    let has_cookie = headers.get(header::COOKIE).is_some();

    let start = Instant::now();
    let response = next.run(req).await;
    let duration = start.elapsed();
    let duration_ms = duration.as_millis();
    let status = response.status();

    // Niveau: warn si requête lente (>=100ms), error si 5xx, sinon info
    if status.is_server_error() {
        error!(
            method = %method,
            path = %path,
            endpoint = %matched_endpoint,
            status = %status,
            duration_ms = duration_ms,
            http_version = %version,
            user_agent = %user_agent,
            client_ip = %client_ip,
            auth_present = has_auth,
            cookie_present = has_cookie,
            "request completed with server error"
        );
    } else if duration_ms >= 100 {
        warn!(
            method = %method,
            path = %path,
            endpoint = %matched_endpoint,
            status = %status,
            duration_ms = duration_ms,
            http_version = %version,
            user_agent = %user_agent,
            client_ip = %client_ip,
            auth_present = has_auth,
            cookie_present = has_cookie,
            "slow request"
        );
    } else {
        info!(
            method = %method,
            path = %path,
            endpoint = %matched_endpoint,
            status = %status,
            duration_ms = duration_ms,
            http_version = %version,
            user_agent = %user_agent,
            client_ip = %client_ip,
            auth_present = has_auth,
            cookie_present = has_cookie,
            "request completed"
        );
    }

    response
}

// Option 1: Utiliser uniquement le middleware personnalisé
pub fn setup_middleware<S>(app: axum::Router<S>) -> axum::Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    app.layer(middleware::from_fn(track_execution_time))
}
