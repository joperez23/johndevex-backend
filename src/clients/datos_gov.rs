use crate::error::AppError;
use bigdecimal::BigDecimal;
use reqwest::Client;
use serde_json::Value;
use std::str::FromStr;

#[derive(Clone)]
pub struct DatosGovClient {
    client: Client,
    url: String,
}

impl DatosGovClient {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            url: "https://www.datos.gov.co/api/v3/views/32sa-8pi3/query.json?query=SELECT valor,vigenciadesde order by vigenciadesde desc limit 1".to_string(),
        }
    }

    pub async fn fetch_usd_cop(&self) -> Result<BigDecimal, AppError> {
        let json_resp: Value = self
            .client
            .get(&self.url)
            .header("User-Agent", "Mozilla/5.0")
            .send()
            .await?
            .json()
            .await?;

        let valor_opt = if let Some(arr) = json_resp.as_array() {
            arr.first()
                .and_then(|item| item.get("valor"))
                .and_then(|v| {
                    v.as_str()
                        .map(|s| s.to_string())
                        .or_else(|| v.as_f64().map(|f| f.to_string()))
                })
        } else if let Some(rows) = json_resp.get("rows").and_then(|r| r.as_array()) {
            rows.first()
                .and_then(|r| r.get(0))
                .and_then(|v| v.as_str().map(|s| s.to_string()))
        } else {
            None
        };

        let raw_valor = valor_opt.ok_or_else(|| {
            AppError::Parse("Could not extract 'valor' from datos.gov.co response".to_string())
        })?;

        BigDecimal::from_str(&raw_valor).map_err(|e| {
            AppError::Parse(format!(
                "Failed to parse USDCOP BigDecimal from '{}': {}",
                raw_valor, e
            ))
        })
    }
}
