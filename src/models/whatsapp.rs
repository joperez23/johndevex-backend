use serde::{Deserialize, Serialize};

/// Represents a WhatsApp message to be queued and sent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppMessage {
    /// Phone number in international format, e.g. "573001234567" (no + or spaces)
    pub to: String,
    /// Plain text body of the message
    pub text: String,
}
