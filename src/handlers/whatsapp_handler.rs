use ntex::web;
use std::sync::Arc;

use crate::AppState;
use crate::error::AppError;
use crate::models::whatsapp::WhatsAppMessage;
use crate::services::whatsapp_worker::get_or_init_worker;

pub async fn send_whatsapp_message(
    _state: web::types::State<Arc<AppState>>,
    msg: web::types::Json<WhatsAppMessage>,
) -> Result<web::HttpResponse, AppError> {
    let payload = msg.into_inner();
    let to = payload.to.clone();

    // Get (or lazily create) the WhatsApp worker channel.
    let tx = get_or_init_worker().ok_or_else(|| {
        AppError::Queue(
            "WhatsApp worker could not start: Chrome/Chromium binary not found.".to_string(),
        )
    })?;

    // Try to send to the channel (non-blocking).
    tx.try_send(payload).map_err(|_| {
        AppError::Queue("WhatsApp message queue is full. Try again later.".to_string())
    })?;

    Ok(web::HttpResponse::Ok().json(&serde_json::json!({
        "status": "queued",
        "message": format!("Message for {} added to the queue to be sent.", to)
    })))
}

/// Serves the latest QR code screenshot captured by the headless Chrome worker.
/// Refresh this URL every few seconds until you see a valid QR code, then scan it.
pub async fn get_qr_screenshot(
    _state: web::types::State<Arc<AppState>>,
) -> web::HttpResponse {
    let qr_path = std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join("whatsapp_qr.png");

    match std::fs::read(&qr_path) {
        Ok(bytes) => web::HttpResponse::Ok()
            .content_type("image/png")
            // Instruct the browser not to cache so each refresh shows the latest QR.
            .header("Cache-Control", "no-store")
            .body(bytes),
        Err(_) => web::HttpResponse::NotFound().json(&serde_json::json!({
            "error": "QR code not available. Either the worker has not started yet, \
                      the session is already active, or Chrome is not running in headless mode."
        })),
    }
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/whatsapp")
            .route("/send", web::post().to(send_whatsapp_message))
            .route("/qr", web::get().to(get_qr_screenshot))
    );
}

