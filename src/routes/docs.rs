//! # Beatmap Routes Module
//!
//! Ce module configure les routes de beatmap.

use axum::{
    Router,
};
use db::db::DatabaseManager;
use utoipa_scalar::{Scalar, Servable as ScalarServable};
use utoipa_redoc::{Redoc, Servable as RedocServable};
use utoipa_rapidoc::{RapiDoc};
use utoipa_swagger_ui::{SwaggerUi};
use utoipa::OpenApi;
#[derive(OpenApi)]
#[openapi(paths(
    crate::handlers::beatmap::batch::checksums::handler,
    crate::handlers::pending_beatmap::get::status_by_osu_id::handler,
    crate::handlers::beatmap::get::list::handler
))]
struct ApiDoc;

pub fn router(db: DatabaseManager) -> Router<DatabaseManager> {
    Router::new()
        .merge(Scalar::with_url("/docs", ApiDoc::openapi()))                // Interface principale
        .merge(Redoc::with_url("/docs/redoc", ApiDoc::openapi()))           // Bonus
        .merge(RapiDoc::new("/api-docs/openapi.json").path("/docs/rapidoc"))           // Bonus
        .merge(SwaggerUi::new("/docs/swagger").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .with_state(db)
}
