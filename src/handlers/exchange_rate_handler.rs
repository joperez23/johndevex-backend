use ntex::web::{
    self,
    types::{Path, Query, State},
    HttpResponse,
};
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::models::{ExchangeRate, Pair};
use crate::state::AppState;

#[derive(Debug, Serialize)]
struct LatestRatesResponse {
    usd: Option<ExchangeRate>,
    eur: Option<ExchangeRate>,
}

/// GET /api/v1/exchange-rates/latest
///
/// Devuelve la última tasa guardada de USD y de EUR (cualquiera de las dos
/// puede venir en `null` si todavía no se ha scrapeado esa moneda).
#[web::get("/exchange-rates/latest")]
pub async fn latest(state: State<AppState>) -> Result<HttpResponse, AppError> {
    let (usd, eur) = state.exchange_rate_service.latest_all().await?;
    Ok(HttpResponse::Ok().json(&LatestRatesResponse { usd, eur }))
}

#[derive(Debug, Deserialize)]
pub struct PairPath {
    pair: String,
}

/// GET /api/v1/exchange-rates/{pair}/latest
///
/// `{pair}` acepta "USD" o "EUR" (sin distinguir mayúsculas/minúsculas).
#[web::get("/exchange-rates/{pair}/latest")]
pub async fn latest_by_pair(
    state: State<AppState>,
    path: Path<PairPath>,
) -> Result<HttpResponse, AppError> {
    let pair: Pair = path.pair.parse()?;
    let price = state.exchange_rate_service.latest(pair).await?;
    Ok(HttpResponse::Ok().json(&price))
}

#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    limit: Option<i64>,
}

/// GET /api/v1/exchange-rates/{pair}/history?limit=30
///
/// Histórico de una moneda, más reciente primero. `limit` es opcional
/// (por defecto 30, máximo 365).
#[web::get("/exchange-rates/{pair}/history")]
pub async fn history(
    state: State<AppState>,
    path: Path<PairPath>,
    query: Query<HistoryQuery>,
) -> Result<HttpResponse, AppError> {
    let pair: Pair = path.pair.parse()?;
    let limit = query.limit.unwrap_or(30).clamp(1, 365);

    let prices = state.exchange_rate_service.history(pair, limit).await?;
    Ok(HttpResponse::Ok().json(&prices))
}

/// POST /api/v1/exchange-rates/scrape
///
/// Dispara un scraping inmediato de https://www.bcv.org.ve/glosario/cambio-oficial,
/// guarda el resultado (USD y EUR) en base de datos como `BigDecimal` y
/// devuelve las dos filas recién insertadas.
#[web::post("/exchange-rates/scrape")]
pub async fn scrape_now(state: State<AppState>) -> Result<HttpResponse, AppError> {
    let result = state.exchange_rate_service.scrape_and_save().await?;

    Ok(HttpResponse::Created().json(&serde_json::json!({
        "usd": result.usd,
        "eur": result.eur,
    })))
}
