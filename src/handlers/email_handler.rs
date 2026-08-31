use ntex::web;
use std::sync::Arc;

use crate::AppState;
use crate::error::AppError;
use crate::models::email::{BatchEmailRequest, EmailQueueStatus, EmailRequest};
use crate::services::email_service::{
    EmailConfig, EmailJob, enqueue_batch, enqueue_email, get_queue_status,
};

/// POST /api/email/send
///
/// Queues a single email to be sent at the configured 15-second cadence.
pub async fn send_email_handler(
    _state: web::types::State<Arc<AppState>>,
    req: web::types::Json<EmailRequest>,
) -> Result<web::HttpResponse, AppError> {
    let payload = req.into_inner();
    let to = payload.to.clone();

    let job = EmailJob {
        to: payload.to,
        subject: payload.subject,
        body: payload.body,
    };

    let total_in_queue = enqueue_email(job).map_err(AppError::Queue)?;
    let config = EmailConfig::from_env();

    Ok(web::HttpResponse::Ok().json(&serde_json::json!({
        "status": "queued",
        "message": format!("Email for {} added to the queue.", to),
        "queue_position": total_in_queue,
        "send_interval_seconds": config.send_interval_secs
    })))
}

/// POST /api/email/send-batch
///
/// Queues a list of emails to be dispatched sequentially every 15 seconds.
pub async fn send_batch_email_handler(
    _state: web::types::State<Arc<AppState>>,
    req: web::types::Json<BatchEmailRequest>,
) -> Result<web::HttpResponse, AppError> {
    let payload = req.into_inner();
    let count = payload.emails.len();

    let jobs: Vec<EmailJob> = payload
        .emails
        .into_iter()
        .map(|e| EmailJob {
            to: e.to,
            subject: e.subject,
            body: e.body,
        })
        .collect();

    let total_in_queue = enqueue_batch(jobs).map_err(AppError::Queue)?;
    let config = EmailConfig::from_env();

    Ok(web::HttpResponse::Ok().json(&serde_json::json!({
        "status": "queued",
        "added_count": count,
        "total_in_queue": total_in_queue,
        "message": format!("{} emails added to the queue to be dispatched every {} seconds.", count, config.send_interval_secs),
        "send_interval_seconds": config.send_interval_secs
    })))
}

/// GET /api/email/queue-status
///
/// Returns current queue metrics and worker status.
pub async fn get_queue_status_handler(
    _state: web::types::State<Arc<AppState>>,
) -> Result<web::HttpResponse, AppError> {
    let (queued_count, interval_secs, is_active) = get_queue_status();

    let status = EmailQueueStatus {
        queued_count,
        send_interval_seconds: interval_secs,
        is_worker_active: is_active,
    };

    Ok(web::HttpResponse::Ok().json(&status))
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/email")
            .route("/send", web::post().to(send_email_handler))
            .route("/send-batch", web::post().to(send_batch_email_handler))
            .route("/queue-status", web::get().to(get_queue_status_handler))
            .route("/status", web::get().to(get_queue_status_handler)),
    );
}
