use crate::error::AppError;
use bigdecimal::BigDecimal;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::str::FromStr;

#[derive(Clone)]
pub struct BanrepClient {
    client: Client,
    url: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BanrepPayload {
    id_indicador: u32,
    datos_indicador: serde_json::Map<String, Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BanrepResponseItem {
    datos_indicador: BanrepDatosIndicador,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BanrepDatosIndicador {
    valor: String,
    tipo: String,
}

impl BanrepClient {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            url: "https://suameca.banrep.gov.co/indicadores-economicos-del-dia-back/rest/indicadoresEconomicosDiaService/consultarIndicador".to_string(),
        }
    }

    pub async fn fetch_eur_cop(&self) -> Result<BigDecimal, AppError> {
        let payload = BanrepPayload {
            id_indicador: 1321,
            datos_indicador: serde_json::Map::new(),
        };

        let items: Vec<BanrepResponseItem> = self
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
            .json()
            .await?;

        // 1. Buscamos en el array el objeto cuyo `tipo` sea "Media"
        let media_item = items
            .iter()
            .find(|item| item.datos_indicador.tipo.eq_ignore_ascii_case("Media"))
            .ok_or_else(|| {
                AppError::Parse("No item with tipo 'Media' found in BanRep response".to_string())
            })?;

        // 2. Limpieza de formato numérico:
        // "3.602,33899" -> quitamos puntos de miles -> "3602,33899" -> cambiamos coma por punto -> "3602.33899"
        let raw_valor = &media_item.datos_indicador.valor;
        let cleaned = raw_valor.replace('.', "").replace(',', ".");

        // 3. Convertimos a BigDecimal
        BigDecimal::from_str(&cleaned).map_err(|e| {
            AppError::Parse(format!(
                "Failed to parse EURCOP BigDecimal from '{}' (cleaned: '{}'): {}",
                raw_valor, cleaned, e
            ))
        })
    }
}
