use ntex::web;
use std::sync::Arc;

use crate::AppState;
use crate::error::AppError;
use crate::models::email::EmailRequest;
use crate::services::email_service::{send_email, EmailConfig};

/// POST /api/email/send
///
/// Sends a plain-text email via Gmail SMTP.
///
/// Request body (JSON):
/// ```json
/// {
///   "to": "recipient@example.com",
///   "subject": "Hello",
///   "body": "Message body here"
/// }
/// ```
pub async fn send_email_handler(
    _state: web::types::State<Arc<AppState>>,
    req: web::types::Json<EmailRequest>,
) -> Result<web::HttpResponse, AppError> {
    let payload = req.into_inner();
    let config = EmailConfig::from_env();

    send_email(&config, &payload.to, &payload.subject, &payload.body)
        .await
        .map_err(|e| AppError::Api(e))?;

    Ok(web::HttpResponse::Ok().json(&serde_json::json!({
        "status": "sent",
        "message": format!("Email sent successfully to {}", payload.to)
    })))
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/email")
            .route("/send", web::post().to(send_email_handler)),
    );
}
