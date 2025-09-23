use crate::handlers::weekly::post::setup_week;
use axum::{Router, routing::post};
use db::db::DatabaseManager;

pub fn router(db: DatabaseManager) -> Router<DatabaseManager> {
    Router::new()
        .route("/weekly/setup", post(setup_week::handler))
        .with_state(db)
}
