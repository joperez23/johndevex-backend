use serde::{Deserialize, Serialize};

/// Payload for the POST /api/email/send endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailRequest {
    /// Recipient email address
    pub to: String,
    /// Email subject line
    pub subject: String,
    /// Plain-text email body
    pub body: String,
}
