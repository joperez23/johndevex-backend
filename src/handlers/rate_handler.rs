use ntex::web;
use std::sync::Arc;

use crate::AppState;
use crate::error::AppError;
use crate::models::rate::RateQuery;

pub async fn sync_usd_ves_handler(
    state: web::types::State<Arc<AppState>>,
) -> Result<web::HttpResponse, AppError> {
    let rate = state.rate_service.sync_usd_ves().await?;
    Ok(web::HttpResponse::Ok().json(&rate))
}

pub async fn sync_eur_ves_handler(
    state: web::types::State<Arc<AppState>>,
) -> Result<web::HttpResponse, AppError> {
    let rate = state.rate_service.sync_eur_ves().await?;
    Ok(web::HttpResponse::Ok().json(&rate))
}

pub async fn sync_usdt_ves_handler(
    state: web::types::State<Arc<AppState>>,
) -> Result<web::HttpResponse, AppError> {
    let rate = state.rate_service.sync_usdt_ves().await?;
    Ok(web::HttpResponse::Ok().json(&rate))
}

pub async fn sync_usd_cop_handler(
    state: web::types::State<Arc<AppState>>,
) -> Result<web::HttpResponse, AppError> {
    let rate = state.rate_service.sync_usd_cop().await?;
    Ok(web::HttpResponse::Ok().json(&rate))
}

pub async fn sync_eur_cop_handler(
    state: web::types::State<Arc<AppState>>,
) -> Result<web::HttpResponse, AppError> {
    let rate = state.rate_service.sync_eur_cop().await?;
    Ok(web::HttpResponse::Ok().json(&rate))
}

pub async fn sync_all_handler(
    state: web::types::State<Arc<AppState>>,
) -> Result<web::HttpResponse, AppError> {
    let rates = state.rate_service.sync_all().await?;
    Ok(web::HttpResponse::Ok().json(&rates))
}

pub async fn get_latest_handler(
    state: web::types::State<Arc<AppState>>,
    query: web::types::Query<RateQuery>,
) -> Result<web::HttpResponse, AppError> {
    let rates = state.rate_service.get_latest(query.pair.as_deref()).await?;
    Ok(web::HttpResponse::Ok().json(&rates))
}

pub async fn get_history_handler(
    state: web::types::State<Arc<AppState>>,
    query: web::types::Query<RateQuery>,
) -> Result<web::HttpResponse, AppError> {
    let rates = state.rate_service.get_history(&query).await?;
    Ok(web::HttpResponse::Ok().json(&rates))
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/finance")
            .route("/sync/usd-ves", web::post().to(sync_usd_ves_handler))
            .route("/sync/eur-ves", web::post().to(sync_eur_ves_handler))
            .route("/sync/usdt-ves", web::post().to(sync_usdt_ves_handler))
            .route("/sync/usd-cop", web::post().to(sync_usd_cop_handler))
            .route("/sync/eur-cop", web::post().to(sync_eur_cop_handler))
            .route("/sync/all", web::post().to(sync_all_handler))
            .route("/latest", web::get().to(get_latest_handler))
            .route("/history", web::get().to(get_history_handler)),
    );
}
