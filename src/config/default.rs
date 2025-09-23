use super::types::*;
use db::config::DatabaseConfig;

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3000,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "json".to_string(),
        }
    }
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allowed_origins: vec![
                "http://localhost:3000".to_string(),
                "http://127.0.0.1:3000".to_string(),
            ],
            allowed_methods: vec![
                "GET".to_string(),
                "POST".to_string(),
                "PUT".to_string(),
                "DELETE".to_string(),
                "OPTIONS".to_string(),
            ],
            allowed_headers: vec!["content-type".to_string(), "authorization".to_string()],
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        use tracing::warn;
        warn!("Using default configuration as no .env file was found");

        Self {
            server: ServerConfig::default(),
            database: DatabaseConfig::default(),
            logging: LoggingConfig::default(),
            cors: CorsConfig::default(),
        }
    }
}
