//! # Routes Module
//!
//! Ce module gère la configuration des routes de l'API.
//! Il permet d'organiser les routes par domaine fonctionnel et de les combiner
//! dans un routeur Axum unique.
//!
//! ## Utilisation
//!
//! Pour ajouter de nouvelles routes :
//! 1. Créez un nouveau module dans le dossier `routes/`
//! 2. Implémentez une fonction `router()` qui retourne un `Router`
//! 3. Ajoutez le module dans ce fichier
//! 4. Utilisez `merge()` pour combiner les routes

use axum::Router;
use db::db::DatabaseManager;

// Re-export all route modules here
pub mod beatmap;
pub mod help;
pub mod docs;
//pub mod pending_beatmap;
//pub mod scores;
//pub mod weekly;


pub fn create_router(db: DatabaseManager) -> Router {
    Router::new()
        // Routes API
        .nest("/api", beatmap::router(db.clone()))
        .nest("/api", help::router())   
        .merge(docs::router(db.clone()))
        //.nest("/api", pending_beatmap::router(db.clone()))
        //.nest("/api", scores::router(db.clone()))
        //.nest("/api", weekly::router(db.clone()))
        // Add your other route modules here
        // Example:
        // .nest("/api", user::router())
        // .nest("/api", product::router())
        .with_state(db)
}
