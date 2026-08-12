use crate::error::AppError;
use bigdecimal::BigDecimal;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Clone)]
pub struct BinanceClient {
    client: Client,
    url: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BinanceSearchRequest<'a> {
    fiat: &'a str,
    page: u32,
    rows: u32,
    trade_type: &'a str,
    asset: &'a str,
    countries: Vec<String>,
    pro_merchant_ads: bool,
    shield_merchant_ads: bool,
    filter_type: &'a str,
    periods: Vec<String>,
    additional_kyc_verify_filter: u32,
    publisher_type: &'a str,
    pay_types: Vec<String>,
    classifies: Vec<&'a str>,
    traded_with: bool,
    followed: bool,
    privilege_desc: Option<String>,
}

#[derive(Deserialize)]
struct BinanceSearchResponse {
    data: Option<Vec<BinanceAdData>>,
}

#[derive(Deserialize)]
struct BinanceAdData {
    adv: BinanceAdv,
    #[serde(rename = "privilegeDesc")]
    privilege_desc: Option<String>,
}

#[derive(Deserialize)]
struct BinanceAdv {
    price: String,
}

impl BinanceClient {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            url: "https://p2p.binance.com/bapi/c2c/v2/friendly/c2c/adv/search".to_string(),
        }
    }

    pub async fn fetch_usdt_ves(&self) -> Result<BigDecimal, AppError> {
        let payload = BinanceSearchRequest {
            fiat: "VES",
            page: 1,
            rows: 1,
            trade_type: "SELL",
            asset: "USDT",
            countries: vec![],
            pro_merchant_ads: false,
            shield_merchant_ads: false,
            filter_type: "tradable",
            periods: vec![],
            additional_kyc_verify_filter: 0,
            publisher_type: "merchant",
            pay_types: vec![],
            classifies: vec!["mass", "profession", "fiat_trade"],
            traded_with: false,
            followed: false,
            privilege_desc: None,
        };

        let response = self
            .client
            .post(&self.url)
            .header("Content-Type", "application/json")
            .header(
                "User-Agent",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/120.0",
            )
            .json(&payload)
            .send()
            .await?
            .json::<BinanceSearchResponse>()
            .await?;

        let price_str = response
            .data
            .and_then(|ads| {
                ads.into_iter()
                    .find(|ad| ad.privilege_desc.as_deref() != Some("Promoted Ad"))
            })
            .map(|ad| ad.adv.price)
            .ok_or_else(|| {
                AppError::Parse(
                    "No valid non-promoted ad returned from Binance P2P API".to_string(),
                )
            })?;

        BigDecimal::from_str(&price_str).map_err(|e| {
            AppError::Parse(format!(
                "Failed to parse Binance price '{}': {}",
                price_str, e
            ))
        })
    }
}
