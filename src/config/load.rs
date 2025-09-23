use db::config::DatabaseConfig;
use dotenvy::var;
use tracing::info;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use super::types::*;

impl ServerConfig {
    pub fn load() -> Self {
        ServerConfig {
            host: var("SERVER_HOST").unwrap_or_else(|_| Self::default().host),
            port: var("SERVER_PORT")
                .unwrap_or_else(|_| Self::default().port.to_string())
                .parse()
                .unwrap_or(Self::default().port),
        }
    }
}

impl LoggingConfig {
    pub fn load() -> Self {
        LoggingConfig {
            level: var("LOG_LEVEL").unwrap_or_else(|_| Self::default().level),
            format: var("LOG_FORMAT").unwrap_or_else(|_| Self::default().format),
        }
    }
}

impl CorsConfig {
    pub fn load() -> Self {
        CorsConfig {
            allowed_origins: var("CORS_ALLOWED_ORIGINS")
                .unwrap_or_else(|_| Self::default().allowed_origins.join(","))
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            allowed_methods: var("CORS_ALLOWED_METHODS")
                .unwrap_or_else(|_| Self::default().allowed_methods.join(","))
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            allowed_headers: var("CORS_ALLOWED_HEADERS")
                .unwrap_or_else(|_| Self::default().allowed_headers.join(","))
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
        }
    }
}

impl Config {
    /// Initialise le système de logging
    fn init_logging(level: &str, _format: &str) {
        let env_filter = EnvFilter::try_from_default_env()
            .or_else(|_| EnvFilter::try_new(level))
            .unwrap_or_else(|_| EnvFilter::new("info"));

        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer())
            .init();

        info!("Logging initialized with level: {}", level);
    }

    /// Charge toute la config
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        dotenvy::dotenv().ok();

        let config = Config {
            server: ServerConfig::load(),
            database: DatabaseConfig::load(),
            logging: LoggingConfig::load(),
            cors: CorsConfig::load(),
        };

        Self::init_logging(&config.logging.level, &config.logging.format);

        info!(
            "Configuration loaded successfully. Server will bind to: {}",
            config.server_address()
        );
        Ok(config)
    }

    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }
}
