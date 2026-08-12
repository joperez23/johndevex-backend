use crate::error::AppError;
use bigdecimal::BigDecimal;
use reqwest::Client;
use scraper::{Html, Selector};
use std::str::FromStr;

#[derive(Clone)]
pub struct BcvClient {
    client: Client,
    url: String,
}

impl BcvClient {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            url: "https://www.bcv.org.ve/glosario/cambio-oficial".to_string(),
        }
    }

    pub async fn fetch_rates(&self) -> Result<(BigDecimal, BigDecimal), AppError> {
        let html_content = self
            .client
            .get(&self.url)
            .header(
                "User-Agent",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/120.0",
            )
            .send()
            .await?
            .text()
            .await?;

        let document = Html::parse_document(&html_content);

        let usd_rate = Self::extract_rate(&document, "#dolar strong")
            .or_else(|_| Self::extract_rate_fallback(&html_content, "dolar"))?;

        let eur_rate = Self::extract_rate(&document, "#euro strong")
            .or_else(|_| Self::extract_rate_fallback(&html_content, "euro"))?;

        Ok((usd_rate, eur_rate))
    }

    fn extract_rate(document: &Html, selector_str: &str) -> Result<BigDecimal, AppError> {
        let selector = Selector::parse(selector_str)
            .map_err(|e| AppError::Parse(format!("Invalid CSS selector: {:?}", e)))?;

        let element = document.select(&selector).next().ok_or_else(|| {
            AppError::Parse(format!("Selector '{}' not found in BCV page", selector_str))
        })?;

        let raw_text = element.text().collect::<String>();
        Self::clean_and_parse_decimal(&raw_text)
    }

    fn extract_rate_fallback(html: &str, currency_id: &str) -> Result<BigDecimal, AppError> {
        let re_str = format!(
            r#"(?i)id="{}"[^>]*>[\s\S]*?<strong>\s*([0-9.,]+)\s*</strong>"#,
            currency_id
        );
        let re = regex::Regex::new(&re_str)
            .map_err(|e| AppError::Parse(format!("Regex compile error: {}", e)))?;

        if let Some(caps) = re.captures(html)
            && let Some(matched) = caps.get(1)
        {
            return Self::clean_and_parse_decimal(matched.as_str());
        }
        Err(AppError::Parse(format!(
            "Failed to scrape BCV rate for {}",
            currency_id
        )))
    }

    fn clean_and_parse_decimal(raw: &str) -> Result<BigDecimal, AppError> {
        let cleaned = raw.trim().replace(',', ".");
        BigDecimal::from_str(&cleaned).map_err(|e| {
            AppError::Parse(format!("Failed to parse BigDecimal from '{}': {}", raw, e))
        })
    }
}
