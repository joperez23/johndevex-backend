use serde::{Deserialize, Serialize};

/// Payload for queuing a single email.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailRequest {
    /// Recipient email address
    pub to: String,
    /// Email subject line
    pub subject: String,
    /// Plain-text email body
    pub body: String,
}

/// Payload for queuing a batch list of emails.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchEmailRequest {
    /// List of email items to be queued
    pub emails: Vec<EmailRequest>,
}

/// Status response for the email queue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailQueueStatus {
    pub queued_count: usize,
    pub send_interval_seconds: u64,
    pub is_worker_active: bool,
}

