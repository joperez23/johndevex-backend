//! Capa de acceso a datos para la tabla `exchange_rates`.
//!
//! Usa la API "dinámica" de sqlx (`sqlx::query_as`) en vez de las macros
//! `query!`/`query_as!` verificadas en tiempo de compilación, para que el
//! proyecto compile sin necesitar una base de datos disponible durante
//! `cargo build` (más simple para empezar; se puede migrar a las macros más
//! adelante con `cargo sqlx prepare` si se desea).

use bigdecimal::BigDecimal;
use chrono::{DateTime, Local};
use sqlx::PgPool;

use crate::error::AppError;
use crate::models::{ExchangeRate, Pair};

const SOURCE_BCV: &str = "BCV";

#[derive(Clone)]
pub struct ExchangeRateRepository {
    pool: PgPool,
}

impl ExchangeRateRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Referencia al pool subyacente (útil para health-checks).
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Inserta una nueva lectura de tasa de cambio y devuelve la fila creada.
    pub async fn insert(
        &self,
        pair: Pair,
        price: &BigDecimal,
        created_at: DateTime<Local>,
    ) -> Result<ExchangeRate, AppError> {
        let record = sqlx::query_as::<_, ExchangeRate>(
            r#"
            INSERT INTO finance.exchange_rates (pair, price, source, created_at)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (pair, date)
            DO UPDATE SET price = EXCLUDED.price, updated_at = now()
            RETURNING id, pair, price, source, date, created_at, updated_at
            "#,
        )
        .bind(pair.as_str())
        .bind(price.clone())
        .bind(SOURCE_BCV)
        .bind(created_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(record)
    }

    /// Devuelve la última lectura registrada para una moneda, si existe.
    pub async fn latest(&self, pair: Pair) -> Result<Option<ExchangeRate>, AppError> {
        let record = sqlx::query_as::<_, ExchangeRate>(
            r#"
            SELECT id, pair, price, source, date, created_at, updated_at
            FROM finance.exchange_rates
            WHERE pair = $1
            ORDER BY date DESC
            LIMIT 1
            "#,
        )
        .bind(pair.as_str())
        .fetch_optional(&self.pool)
        .await?;

        Ok(record)
    }

    /// Devuelve el histórico de una moneda, más reciente primero.
    pub async fn history(&self, pair: Pair, limit: i64) -> Result<Vec<ExchangeRate>, AppError> {
        let records = sqlx::query_as::<_, ExchangeRate>(
            r#"
            SELECT id, pair, price, source, date, created_at, updated_at
            FROM finance.exchange_rates
            WHERE pair = $1
            ORDER BY date DESC
            LIMIT $2
            "#,
        )
        .bind(pair.as_str())
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }
}
