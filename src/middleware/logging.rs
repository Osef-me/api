use axum::{
    body::Body,
    http::Request,
    middleware::{self, Next},
    response::Response,
};
use std::time::Instant;
use tracing::info;

pub async fn track_execution_time(req: Request<Body>, next: Next) -> Response {
    let path = req.uri().path().to_owned();
    let method = req.method().clone();

    let start = Instant::now();
    let response = next.run(req).await;
    let duration = start.elapsed();

    info!(
        "Request {} {} completed in {:.2?} with status {}",
        method,
        path,
        duration,
        response.status()
    );

    response
}

// Option 1: Utiliser uniquement le middleware personnalisé
pub fn setup_middleware<S>(app: axum::Router<S>) -> axum::Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    app.layer(middleware::from_fn(track_execution_time))
}
