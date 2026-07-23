//! Tarea en background opcional que repite el scraping cada N segundos.
//!
//! Se activa solo si la variable de entorno `BCV_SCRAPE_INTERVAL_SECS` está
//! definida. Usa `ntex::rt::spawn` (el spawner propio del runtime de ntex)
//! para que la tarea viva dentro del mismo runtime que el servidor HTTP.

use std::time::Duration;

use crate::services::exchange_rate_service::ExchangeRateService;

pub fn spawn_periodic_scrape(service: ExchangeRateService, interval_secs: u64) {
    log::info!("scraping automático del BCV activado cada {interval_secs}s");

    ntex::rt::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(interval_secs)).await;

            match service.scrape_and_save().await {
                Ok(_) => log::info!("scraping periódico del BCV completado"),
                Err(err) => log::warn!("scraping periódico del BCV falló: {err}"),
            }
        }
    });
}
