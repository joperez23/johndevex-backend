//! Configuración de la aplicación, cargada desde variables de entorno.
//!
//! Toda variable tiene un valor por defecto razonable, salvo `DATABASE_URL`
//! que es obligatoria. Ver `.env.example` para la lista completa.

use std::env;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct Config {
    // Servidor
    pub server_host: String,
    pub server_port: u16,
    pub server_workers: Option<usize>,

    // Base de datos
    pub database_url: String,
    pub database_max_connections: u32,
    pub database_min_connections: u32,
    pub database_connect_timeout_secs: u64,

    // Scraper BCV
    pub bcv_url: String,
    pub bcv_request_timeout_secs: u64,
    pub bcv_insecure_tls: bool,
    pub bcv_scrape_interval_secs: Option<u64>,

    // CORS
    pub cors_allowed_origins: Vec<String>,

    // Logging
    pub log_level: String,
}

/// Error de configuración: variable faltante o con un valor que no se pudo
/// interpretar.
#[derive(Debug)]
pub struct ConfigError(String);

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ConfigError {}

impl Config {
    /// Construye la configuración a partir de las variables de entorno
    /// actualmente definidas (llama primero a `dotenvy::dotenv()` si quieres
    /// cargar un archivo `.env`).
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            server_host: env_or("SERVER_HOST", "0.0.0.0"),
            server_port: env_parse_or("SERVER_PORT", 8080)?,
            server_workers: env_opt_parse("SERVER_WORKERS")?,

            database_url: env_required("DATABASE_URL")?,
            database_max_connections: env_parse_or("DATABASE_MAX_CONNECTIONS", 10)?,
            database_min_connections: env_parse_or("DATABASE_MIN_CONNECTIONS", 1)?,
            database_connect_timeout_secs: env_parse_or("DATABASE_CONNECT_TIMEOUT_SECS", 10)?,

            bcv_url: env_or(
                "BCV_URL",
                crate::services::bcv_scraper::DEFAULT_BCV_URL,
            ),
            bcv_request_timeout_secs: env_parse_or("BCV_REQUEST_TIMEOUT_SECS", 20)?,
            bcv_insecure_tls: env_parse_or("BCV_INSECURE_TLS", true)?,
            bcv_scrape_interval_secs: env_opt_parse("BCV_SCRAPE_INTERVAL_SECS")?,

            cors_allowed_origins: env_or("CORS_ALLOWED_ORIGINS", "*")
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),

            log_level: env_or("RUST_LOG", "info,ntex=info,sqlx=warn"),
        })
    }
}

fn env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

fn env_required(key: &str) -> Result<String, ConfigError> {
    env::var(key).map_err(|_| ConfigError(format!("falta la variable de entorno requerida: {key}")))
}

fn env_parse_or<T>(key: &str, default: T) -> Result<T, ConfigError>
where
    T: FromStr,
    T::Err: fmt::Display,
{
    match env::var(key) {
        Ok(val) => val
            .parse::<T>()
            .map_err(|e| ConfigError(format!("valor inválido para {key}: {e}"))),
        Err(_) => Ok(default),
    }
}

fn env_opt_parse<T>(key: &str) -> Result<Option<T>, ConfigError>
where
    T: FromStr,
    T::Err: fmt::Display,
{
    match env::var(key) {
        Ok(val) if !val.trim().is_empty() => val
            .parse::<T>()
            .map(Some)
            .map_err(|e| ConfigError(format!("valor inválido para {key}: {e}"))),
        _ => Ok(None),
    }
}
