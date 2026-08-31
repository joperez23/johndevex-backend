mod clients;
mod config;
mod db;
mod error;
mod handlers;
mod models;
mod services;

use ntex::web;
use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;

use config::Config;
use services::rate_service::RateService;

pub struct AppState {
    pub rate_service: RateService,
}

#[ntex::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let config = Config::from_env();
    log::info!("Starting Finance Rates Backend server...");

    let pool = db::init_pool(&config.database_url)
        .await
        .expect("Failed to create PostgreSQL connection pool");

    log::info!("Successfully connected to PostgreSQL database.");

    // reqwest client with TLS & custom timeout
    let http_client = Client::builder()
        .timeout(Duration::from_secs(config.request_timeout_secs))
        .danger_accept_invalid_certs(true)
        .build()
        .expect("Failed to build HTTP client");

    let rate_service = RateService::new(pool, http_client);

    let app_state = Arc::new(AppState { rate_service });

    // Start the Email queue worker eagerly
    services::email_service::get_or_init_email_worker();
    log::info!("Email background worker initialized (15s dispatch cadence).");

    // Start the WhatsApp worker eagerly so Chrome opens and the QR code
    // is shown immediately at boot, rather than on the first API call.
    // If Chrome is not installed the server still starts (get_or_init_worker
    // returns None and logs an error, which is handled gracefully).
    match services::whatsapp_worker::get_or_init_worker() {
        Some(_) => log::info!("WhatsApp worker started — open the Chrome window and scan the QR code."),
        None => log::warn!("WhatsApp worker could not start (Chrome not found). /api/whatsapp/send will be unavailable."),
    }


    let bind_addr = format!("{}:{}", config.server_host, config.server_port);
    log::info!("Server listening on http://{}", bind_addr);

    web::HttpServer::new(move || {
        // 1. Clone the Arc for this specific worker thread
        let app_state = app_state.clone();

        // 2. Return an `async move` future that owns the cloned state
        async move {
            web::App::new()
                .state(app_state)
                .middleware(web::middleware::Logger::default())
                .configure(handlers::rate_handler::configure_routes)
                .configure(handlers::whatsapp_handler::configure_routes)
                .configure(handlers::email_handler::configure_routes)
        }
    })
    .bind(&bind_addr)?
    .run()
    .await
}

