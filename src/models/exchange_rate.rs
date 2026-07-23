use std::fmt;
use std::str::FromStr;

use bigdecimal::BigDecimal;
use chrono::{DateTime, Local, NaiveDate};
use serde::Serialize;

use crate::error::AppError;

/// Fila persistida en la tabla `exchange_rates`.
///
/// `rate` se maneja como `BigDecimal` (precisión exacta, sin errores de
/// redondeo de punto flotante), tal como se pidió, y se mapea 1:1 a la
/// columna `NUMERIC(20, 8)` de PostgreSQL gracias al feature `bigdecimal`
/// de sqlx.
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct ExchangeRate {
    pub id: i64,
    pub pair: String,
    pub price: BigDecimal,
    pub source: String,
    pub date: NaiveDate,
    pub created_at: DateTime<Local>,
    pub updated_at: Option<DateTime<Local>>,
}

/// Monedas soportadas actualmente por el scraper del BCV.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Pair {
    UsdVes,
    EurVes,
}

impl Pair {
    pub fn as_str(&self) -> &'static str {
        match self {
            Pair::UsdVes => "USDVES",
            Pair::EurVes => "EURVES",
        }
    }
}

impl fmt::Display for Pair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Pair {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_uppercase().as_str() {
            "USDVES" => Ok(Pair::UsdVes),
            "EURVES" => Ok(Pair::EurVes),
            other => Err(AppError::BadRequest(format!(
                "moneda no soportada: '{other}' (valores válidos: USD, EUR)"
            ))),
        }
    }
}
