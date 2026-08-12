use bigdecimal::BigDecimal;
use chrono::Utc;
use reqwest::Client;
use sqlx::PgPool;

use crate::clients::banrep::BanrepClient;
use crate::clients::bcv::BcvClient;
use crate::clients::binance::BinanceClient;
use crate::clients::datos_gov::DatosGovClient;
use crate::error::AppError;
use crate::models::rate::{ExchangeRate, RateQuery};

#[derive(Clone)]
pub struct RateService {
    pool: PgPool,
    bcv_client: BcvClient,
    binance_client: BinanceClient,
    datos_gov_client: DatosGovClient,
    banrep_client: BanrepClient,
}

impl RateService {
    pub fn new(pool: PgPool, http_client: Client) -> Self {
        Self {
            pool,
            bcv_client: BcvClient::new(http_client.clone()),
            binance_client: BinanceClient::new(http_client.clone()),
            datos_gov_client: DatosGovClient::new(http_client.clone()),
            banrep_client: BanrepClient::new(http_client),
        }
    }

    pub async fn upsert_rate(
        &self,
        pair: &str,
        price: BigDecimal,
        source: &str,
    ) -> Result<ExchangeRate, AppError> {
        let today = Utc::now().date_naive();

        let record = sqlx::query_as::<_, ExchangeRate>(
            r#"
            -- SELECT id, pair, price, source, date, created_at, updated_at FROM finance.exchange_rates LIMIT 1;
            INSERT INTO finance.exchange_rates (pair, price, source, date, created_at)
            VALUES ($1, $2, $3, $4, NOW())
            ON CONFLICT (pair, date) 
            DO UPDATE SET 
                price = EXCLUDED.price,
                source = EXCLUDED.source,
                updated_at = NOW()
            RETURNING id, pair, price, source, date, created_at, updated_at
            "#,
        )
        .bind(pair)
        .bind(price)
        .bind(source)
        .bind(today)
        .fetch_one(&self.pool)
        .await?;

        Ok(record)
    }

    pub async fn sync_usd_ves(&self) -> Result<ExchangeRate, AppError> {
        let (usd, _eur) = self.bcv_client.fetch_rates().await?;
        self.upsert_rate("USDVES", usd, "BCV").await
    }

    pub async fn sync_eur_ves(&self) -> Result<ExchangeRate, AppError> {
        let (_usd, eur) = self.bcv_client.fetch_rates().await?;
        self.upsert_rate("EURVES", eur, "BCV").await
    }

    pub async fn sync_usdt_ves(&self) -> Result<ExchangeRate, AppError> {
        let price = self.binance_client.fetch_usdt_ves().await?;
        self.upsert_rate("USDTVES", price, "BINANCE_P2P").await
    }

    pub async fn sync_usd_cop(&self) -> Result<ExchangeRate, AppError> {
        let price = self.datos_gov_client.fetch_usd_cop().await?;
        self.upsert_rate("USDCOP", price, "DATOS_GOV_CO").await
    }

    pub async fn sync_eur_cop(&self) -> Result<ExchangeRate, AppError> {
        let price = self.banrep_client.fetch_eur_cop().await?;
        self.upsert_rate("EURCOP", price, "BANREP").await
    }

    pub async fn sync_all(&self) -> Result<Vec<ExchangeRate>, AppError> {
        let mut results = Vec::new();

        if let Ok(rate) = self.sync_usd_ves().await {
            results.push(rate);
        }
        if let Ok(rate) = self.sync_eur_ves().await {
            results.push(rate);
        }
        if let Ok(rate) = self.sync_usdt_ves().await {
            results.push(rate);
        }
        if let Ok(rate) = self.sync_usd_cop().await {
            results.push(rate);
        }
        if let Ok(rate) = self.sync_eur_cop().await {
            results.push(rate);
        }

        Ok(results)
    }

    pub async fn get_latest(
        &self,
        pair_filter: Option<&str>,
    ) -> Result<Vec<ExchangeRate>, AppError> {
        let rates = if let Some(pair) = pair_filter {
            sqlx::query_as::<_, ExchangeRate>(
                r#"
                SELECT DISTINCT ON (pair) id, pair, price, source, date, created_at, updated_at
                FROM finance.exchange_rates
                WHERE pair = $1
                ORDER BY pair, date DESC
                "#,
            )
            .bind(pair)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, ExchangeRate>(
                r#"
                SELECT DISTINCT ON (pair) id, pair, price, source, date, created_at, updated_at
                FROM finance.exchange_rates
                ORDER BY pair, date DESC
                "#,
            )
            .fetch_all(&self.pool)
            .await?
        };

        Ok(rates)
    }

    pub async fn get_history(&self, query: &RateQuery) -> Result<Vec<ExchangeRate>, AppError> {
        let limit = query.limit.unwrap_or(50).min(500);
        let offset = query.offset.unwrap_or(0);

        let rates = if let Some(ref pair) = query.pair {
            sqlx::query_as::<_, ExchangeRate>(
                r#"
                SELECT id, pair, price, source, date, created_at, updated_at
                FROM finance.exchange_rates
                WHERE pair = $1
                ORDER BY date DESC
                LIMIT $2 OFFSET $3
                "#,
            )
            .bind(pair)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, ExchangeRate>(
                r#"
                SELECT id, pair, price, source, date, created_at, updated_at
                FROM finance.exchange_rates
                ORDER BY date DESC, pair ASC
                LIMIT $1 OFFSET $2
                "#,
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?
        };

        Ok(rates)
    }
}
