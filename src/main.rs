#![recursion_limit = "256"]
mod config;
mod db;
mod error;
mod handlers;
mod models;
mod repositories;
mod routes;
mod services;
mod state;

use ntex::web::{self, middleware, App, HttpServer};
use ntex_cors::Cors;

use crate::config::Config;
use crate::repositories::ExchangeRateRepository;
use crate::services::scheduler;
use crate::services::{BcvScraper, ExchangeRateService};
use crate::state::AppState;

#[ntex::main]
async fn main() -> std::io::Result<()> {
    // Carga variables desde un archivo `.env` si existe (no falla si no lo
    // encuentra: en producción normalmente se inyectan las variables
    // directamente en el entorno del contenedor).
    dotenvy::dotenv().ok();

    let config = Config::from_env().unwrap_or_else(|err| {
        eprintln!("[config] {err}");
        std::process::exit(1);
    });

    if std::env::var_os("RUST_LOG").is_none() {
        // SAFETY: se ejecuta una sola vez, antes de spawnear ningún hilo.
        unsafe { std::env::set_var("RUST_LOG", &config.log_level) };
    }
    env_logger::init();

    log::info!("conectando a PostgreSQL...");
    let pool = db::create_pool(&config).await.unwrap_or_else(|err| {
        log::error!("no se pudo conectar a la base de datos: {err}");
        std::process::exit(1);
    });

    log::info!("ejecutando migraciones pendientes...");
    db::run_migrations(&pool).await.unwrap_or_else(|err| {
        log::error!("no se pudieron ejecutar las migraciones: {err}");
        std::process::exit(1);
    });

    let scraper = BcvScraper::new(
        config.bcv_url.clone(),
        config.bcv_request_timeout_secs,
        config.bcv_insecure_tls,
    )
    .unwrap_or_else(|err| {
        log::error!("no se pudo inicializar el scraper del BCV: {err}");
        std::process::exit(1);
    });

    let repository = ExchangeRateRepository::new(pool);
    let exchange_rate_service = ExchangeRateService::new(scraper, repository);

    // Scraping inicial "best effort": si el BCV no responde al arrancar, la
    // API igual queda operativa (los endpoints de lectura simplemente
    // devolverán 404 hasta el próximo scraping exitoso).
    match exchange_rate_service.scrape_and_save().await {
        Ok(_) => log::info!("scraping inicial del BCV completado"),
        Err(err) => {
            log::warn!("scraping inicial del BCV falló (se reintentará más adelante): {err}")
        }
    }

    if let Some(interval_secs) = config.bcv_scrape_interval_secs {
        scheduler::spawn_periodic_scrape(exchange_rate_service.clone(), interval_secs);
    }

    let app_state = AppState {
        exchange_rate_service,
    };
    let cors_origins = config.cors_allowed_origins.clone();
    let bind_addr = format!("{}:{}", config.server_host, config.server_port);
    let workers = config.server_workers;

    log::info!("iniciando servidor HTTP en http://{bind_addr}");

    let mut server = HttpServer::new(move || {
        let app_state = app_state.clone();
        let cors_origins = cors_origins.clone();

        async move {
            let mut cors = Cors::new();
            if cors_origins.iter().any(|origin| origin == "*") {
                cors = cors.send_wildcard();
            } else {
                for origin in &cors_origins {
                    cors = cors.allowed_origin(origin);
                }
            }

            App::new()
                .state(app_state)
                .middleware(middleware::Logger::default())
                .middleware(middleware::Compress::default())
                .middleware(cors.finish::<ntex::web::error::DefaultError>())
                .configure(routes::configure)
                .default_service(web::route().to(routes::not_found))
        }
    })
    .bind(bind_addr)?;

    if let Some(workers) = workers {
        server = server.workers(workers);
    }

    server.run().await
}
