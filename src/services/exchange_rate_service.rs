//! Orquesta el scraping del BCV y su persistencia en base de datos, y
//! expone las operaciones de lectura que usan los handlers HTTP.

use chrono::{DateTime, Local};

use crate::error::AppError;
use crate::models::{ExchangeRate, Pair};
use crate::repositories::ExchangeRateRepository;
use crate::services::bcv_scraper::BcvScraper;

#[derive(Clone)]
pub struct ExchangeRateService {
    scraper: BcvScraper,
    repository: ExchangeRateRepository,
}

/// Resultado de un ciclo de scraping + guardado exitoso.
#[derive(Debug)]
pub struct ScrapeAndSaveResult {
    pub usd: ExchangeRate,
    pub eur: ExchangeRate,
}

impl ExchangeRateService {
    pub fn new(scraper: BcvScraper, repository: ExchangeRateRepository) -> Self {
        Self {
            scraper,
            repository,
        }
    }

    /// Descarga las tasas actuales del BCV y las guarda en base de datos
    /// como dos filas nuevas (una por moneda), ambas con el mismo
    /// `scraped_at`.
    pub async fn scrape_and_save(&self) -> Result<ScrapeAndSaveResult, AppError> {
        let prices = self.scraper.fetch_rates().await?;

        let created_at: DateTime<Local> = Local::now();
        println!("{}", created_at);

        let usd = self
            .repository
            .insert(Pair::UsdVes, &prices.usd, created_at)
            .await?;
        let eur = self
            .repository
            .insert(Pair::EurVes, &prices.eur, created_at)
            .await?;

        log::info!(
            "tasas BCV guardadas -> USD: {} VES, EUR: {} VES ({})",
            prices.usd,
            prices.eur,
            created_at.to_rfc3339()
        );

        Ok(ScrapeAndSaveResult { usd, eur })
    }

    /// Última tasa guardada para una moneda específica. Devuelve
    /// `AppError::NotFound` si todavía no se ha scrapeado ninguna.
    pub async fn latest(&self, pair: Pair) -> Result<ExchangeRate, AppError> {
        self.repository.latest(pair).await?.ok_or_else(|| {
            AppError::NotFound(format!(
                "no hay tasas registradas para {pair}. Ejecuta primero POST /api/v1/exchange-rates/scrape"
            ))
        })
    }

    /// Últimas tasas de USD y EUR (cualquiera de las dos puede ser `None`
    /// si aún no se ha scrapeado esa moneda).
    pub async fn latest_all(
        &self,
    ) -> Result<(Option<ExchangeRate>, Option<ExchangeRate>), AppError> {
        let usd = self.repository.latest(Pair::UsdVes).await?;
        let eur = self.repository.latest(Pair::EurVes).await?;
        Ok((usd, eur))
    }

    /// Histórico de una moneda, más reciente primero, limitado a `limit`
    /// filas.
    pub async fn history(&self, pair: Pair, limit: i64) -> Result<Vec<ExchangeRate>, AppError> {
        self.repository.history(pair, limit).await
    }

    /// Comprueba que la base de datos responde (para el health-check).
    pub async fn ping_database(&self) -> Result<(), AppError> {
        sqlx::query("SELECT 1")
            .execute(self.repository.pool())
            .await?;
        Ok(())
    }
}
