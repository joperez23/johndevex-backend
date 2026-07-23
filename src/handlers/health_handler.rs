use ntex::web::{self, types::State, HttpResponse};
use serde::Serialize;

use crate::state::AppState;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    database: &'static str,
}

/// GET /health
///
/// Health-check simple: responde 200 si el proceso está vivo y puede
/// alcanzar la base de datos; 503 si la base de datos no responde.
#[web::get("/health")]
pub async fn health(state: State<AppState>) -> HttpResponse {
    match state.exchange_rate_service.ping_database().await {
        Ok(()) => HttpResponse::Ok().json(&HealthResponse {
            status: "ok",
            database: "up",
        }),
        Err(err) => {
            log::warn!("health-check: la base de datos no responde: {err}");
            HttpResponse::ServiceUnavailable().json(&HealthResponse {
                status: "degraded",
                database: "down",
            })
        }
    }
}
