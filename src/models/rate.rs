use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ExchangeRate {
    pub id: i64,
    pub pair: String,
    pub price: BigDecimal,
    pub source: String,
    pub date: NaiveDate,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct RateQuery {
    pub pair: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
