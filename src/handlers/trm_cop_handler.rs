use ntex::web;
use serde::{Deserialize, Serialize};

// Estructura para mapear la respuesta de la API de Datos Abiertos de Colombia
#[derive(Deserialize)]
struct GovernmentTrmResponse {
    valor: String,
    vigenciadesde: String,
}

#[derive(Serialize)]
struct TrmResponse {
    trm: String,
    fecha_vigencia: String,
}

#[web::get("/sync-trm")]
pub async fn sync_colombian_trm() -> Result<web::HttpResponse, web::Error> {
    // API oficial de Datos Abiertos de la TRM en Colombia (ordenada por la más reciente)
    // let url = "https://www.datos.gov.co/resource/ceyp-9c7c.json?$limit=1&$order=vigenciadesde DESC";
    let url = "https://www.datos.gov.co/api/v3/views/32sa-8pi3/query.json?query=SELECT valor,vigenciadesde order by vigenciadesde desc limit 1";

    let client = reqwest::Client::new();

    // Hacemos la petición directamente al JSON de la API
    let response: Vec<GovernmentTrmResponse> = client
        .get(url)
        .header("User-Agent", "Mozilla/5.0")
        .send()
        .await
        .map_err(|e| web::error::ErrorInternalServerError(format!("Error de conexión: {}", e)))?
        .json()
        .await
        .map_err(|e| {
            web::error::ErrorInternalServerError(format!("Error al parsear el JSON: {}", e))
        })?;

    // println!("{:?}");

    // Validamos que la API nos haya retornado al menos un registro
    if let Some(latest_trm) = response.first() {
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

        Ok(web::HttpResponse::Ok().json(&TrmResponse {
            trm: latest_trm.valor.clone(),
            fecha_vigencia: latest_trm.vigenciadesde.clone(),
        }))
    } else {
        Ok(web::HttpResponse::NotFound().json(&serde_json::json!({
            "error": "No se encontraron registros de TRM activos"
        })))
    }
}
