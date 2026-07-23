//! Composición de rutas de la API.
//!
//! Se usa `App::configure()` (ver https://ntex.rs/docs/application#configure)
//! para mantener `main.rs` limpio y poder testear/registrar las rutas desde
//! un único punto.

use ntex::web::{self, HttpResponse};

use crate::handlers::{exchange_rate_handler, health_handler, trm_cop_handler};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(health_handler::health);

    cfg.service(
        web::scope("/api/v1")
            .service(exchange_rate_handler::latest)
            .service(exchange_rate_handler::latest_by_pair)
            .service(exchange_rate_handler::history)
            .service(exchange_rate_handler::scrape_now)
            .service(trm_cop_handler::sync_colombian_trm),
    );
}

/// Handler reutilizado como `default_service` de la `App` en `main.rs`:
/// cualquier ruta no encontrada devuelve un 404 en JSON (en vez del HTML
/// por defecto de ntex), consistente con el resto de la API.
pub async fn not_found() -> HttpResponse {
    HttpResponse::NotFound().json(&serde_json::json!({
        "error": "not_found",
        "message": "el recurso solicitado no existe",
    }))
}
