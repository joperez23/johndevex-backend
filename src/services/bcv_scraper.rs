//! Scraper de https://www.bcv.org.ve/glosario/cambio-oficial
//!
//! El BCV no ofrece una API oficial, así que se obtiene el HTML público de
//! esa página y se extrae el valor de USD y EUR desde los contenedores
//! `div#dolar` y `div#euro` (patrón usado consistentemente por la comunidad
//! para este sitio). Los valores en el HTML vienen en formato venezolano
//! (coma como separador decimal, ej. "168,66830000"), por lo que se
//! normalizan antes de convertirlos a `BigDecimal`.
//!
//! Nota sobre TLS: el certificado del sitio del BCV ha dado problemas de
//! validación de forma recurrente a lo largo de los años (es un problema
//! conocido y documentado por quienes hacen scraping de este sitio en
//! particular). Por eso este cliente permite desactivar la verificación
//! estricta del certificado únicamente para esta URL fija y conocida,
//! controlado por la variable de entorno `BCV_INSECURE_TLS`.

use std::str::FromStr;
use std::time::Duration;

use bigdecimal::BigDecimal;
use scraper::{Html, Selector};

use crate::error::AppError;

pub const DEFAULT_BCV_URL: &str = "https://www.bcv.org.ve/glosario/cambio-oficial";

/// Resultado de un scraping exitoso: tasas VES por 1 USD y VES por 1 EUR.
#[derive(Debug, Clone)]
pub struct BcvRates {
    pub usd: BigDecimal,
    pub eur: BigDecimal,
}

#[derive(Clone)]
pub struct BcvScraper {
    client: reqwest::Client,
    url: String,
}

impl BcvScraper {
    pub fn new(url: String, timeout_secs: u64, insecure_tls: bool) -> Result<Self, AppError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            // Ver nota de módulo: solo afecta a las peticiones hechas con
            // este cliente, que únicamente llama a `url` (el BCV).
            .danger_accept_invalid_certs(insecure_tls)
            .user_agent(
                "Mozilla/5.0 (compatible; bcv-rates-api/1.0; +https://ntex.rs) reqwest",
            )
            .build()
            .map_err(|e| AppError::Internal(format!("no se pudo construir el cliente HTTP: {e}")))?;

        Ok(Self { client, url })
    }

    /// Descarga la página del BCV y extrae las tasas de USD y EUR.
    pub async fn fetch_rates(&self) -> Result<BcvRates, AppError> {
        log::debug!("descargando {}", self.url);

        let response = self
            .client
            .get(&self.url)
            .send()
            .await
            .map_err(|e| AppError::Scraping(format!("no se pudo conectar con el BCV: {e}")))?;

        let response = response
            .error_for_status()
            .map_err(|e| AppError::Scraping(format!("el BCV respondió con error HTTP: {e}")))?;

        let body = response
            .text()
            .await
            .map_err(|e| AppError::Scraping(format!("no se pudo leer la respuesta del BCV: {e}")))?;

        Self::parse_rates(&body)
    }

    /// Lógica de parseo, separada de la petición HTTP para poder testearla
    /// fácilmente con HTML de ejemplo (ver tests al final del archivo).
    fn parse_rates(html: &str) -> Result<BcvRates, AppError> {
        let document = Html::parse_document(html);

        let usd = extract_rate(&document, "USD", &["#dolar strong", "#dolar"])?;
        let eur = extract_rate(&document, "EUR", &["#euro strong", "#euro"])?;

        Ok(BcvRates { usd, eur })
    }
}

/// Busca el primer selector (de una lista de alternativas, por si el sitio
/// cambia ligeramente su marcado) que contenga un número parseable.
fn extract_rate(document: &Html, label: &str, selectors: &[&str]) -> Result<BigDecimal, AppError> {
    for raw_selector in selectors {
        let selector = Selector::parse(raw_selector)
            .map_err(|e| AppError::Internal(format!("selector CSS inválido '{raw_selector}': {e:?}")))?;

        if let Some(element) = document.select(&selector).next() {
            let text: String = element.text().collect::<Vec<_>>().join("");
            if let Ok(value) = parse_ves_decimal(&text) {
                return Ok(value);
            }
        }
    }

    Err(AppError::Scraping(format!(
        "no se pudo encontrar la tasa de {label} en la página del BCV \
         (es posible que la estructura del sitio haya cambiado)"
    )))
}

/// Convierte un número en formato venezolano (coma decimal, punto como
/// separador de miles) a `BigDecimal`. Ej: "168,66830000" -> 168.66830000
pub fn parse_ves_decimal(raw: &str) -> Result<BigDecimal, AppError> {
    let cleaned: String = raw.chars().filter(|c| !c.is_whitespace()).collect();

    if cleaned.is_empty() {
        return Err(AppError::Scraping("valor numérico vacío".to_string()));
    }

    let normalized = if cleaned.contains(',') {
        // "1.234,56" -> quitar puntos (miles) y cambiar coma por punto
        cleaned.replace('.', "").replace(',', ".")
    } else {
        cleaned
    };

    BigDecimal::from_str(&normalized)
        .map_err(|e| AppError::Scraping(format!("valor numérico inválido '{raw}': {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_venezuelan_decimal_format() {
        let value = parse_ves_decimal("168,66830000").unwrap();
        assert_eq!(value, BigDecimal::from_str("168.66830000").unwrap());
    }

    #[test]
    fn parses_plain_dot_format() {
        let value = parse_ves_decimal("36.52").unwrap();
        assert_eq!(value, BigDecimal::from_str("36.52").unwrap());
    }

    #[test]
    fn rejects_empty_value() {
        assert!(parse_ves_decimal("   ").is_err());
    }

    #[test]
    fn extracts_usd_and_eur_from_sample_html() {
        // HTML mínimo que reproduce la estructura real del BCV: contenedores
        // con id="dolar" / id="euro" que envuelven un <strong> con el valor.
        let html = r#"
            <html>
                <body>
                    <div id="dolar">
                        <span>USD</span>
                        <strong>168,66830000</strong>
                    </div>
                    <div id="euro">
                        <span>EUR</span>
                        <strong>182,14330000</strong>
                    </div>
                </body>
            </html>
        "#;

        let rates = BcvScraper::parse_rates(html).expect("debería parsear correctamente");

        assert_eq!(rates.usd, BigDecimal::from_str("168.66830000").unwrap());
        assert_eq!(rates.eur, BigDecimal::from_str("182.14330000").unwrap());
    }
}
