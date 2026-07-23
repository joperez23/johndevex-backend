//! Creación del pool de conexiones a PostgreSQL y ejecución de migraciones.

use std::time::Duration;

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use crate::config::Config;

/// Migrador embebido en el binario en tiempo de compilación a partir de los
/// archivos `.sql` en `./migrations`. Se ejecuta una sola vez al arrancar.
static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

/// Crea el pool de conexiones usando los parámetros de `Config`.
pub async fn create_pool(config: &Config) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(config.database_max_connections)
        .min_connections(config.database_min_connections)
        .acquire_timeout(Duration::from_secs(config.database_connect_timeout_secs))
        .connect(&config.database_url)
        .await
}

/// Ejecuta las migraciones pendientes contra el pool dado.
///
/// Es seguro llamarla en cada arranque: sqlx lleva su propia tabla de
/// control (`_sqlx_migrations`) y solo aplica lo que falte.
pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    MIGRATOR.run(pool).await
}
