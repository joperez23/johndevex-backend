//! Estado compartido de la aplicación.
//!
//! Se registra una sola vez con `App::state()` y se inyecta en cada handler
//! mediante el extractor `web::types::State<AppState>`. Internamente solo
//! contiene handles "baratos" de clonar (el pool de sqlx y el cliente de
//! reqwest son ambos `Arc`-friendly), así que clonarlo por cada worker HTTP
//! es económico y seguro (ver https://ntex.rs/docs/application#shared-mutable-state).

use crate::services::ExchangeRateService;

#[derive(Clone)]
pub struct AppState {
    pub exchange_rate_service: ExchangeRateService,
}
