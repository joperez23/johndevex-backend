use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

/// Credentials for the Gmail SMTP sender, loaded from environment variables
/// `GMAIL_USER` and `GMAIL_PASSWORD`.
#[derive(Clone, Debug)]
pub struct EmailConfig {
    pub from_address: String,
    pub password: String,
}

impl EmailConfig {
    /// Load Gmail credentials from environment variables.
    /// Falls back to the values baked-in at compile time when the vars are absent.
    pub fn from_env() -> Self {
        let from_address =
            std::env::var("GMAIL_USER").unwrap_or_else(|_| "joperezd23@gmail.com".to_string());
        let password =
            std::env::var("GMAIL_PASSWORD").unwrap_or_else(|_| "lalhhdutfowyiyvq".to_string());
        Self {
            from_address,
            password,
        }
    }
}

/// Send a plain-text email via Gmail SMTP (TLS, port 587).
///
/// # Arguments
/// * `config`  – Gmail credentials (from / password).
/// * `to`      – Recipient address, e.g. `"joperez@gmail.com"`.
/// * `subject` – Email subject line.
/// * `body`    – Plain-text message body.
pub async fn send_email(
    config: &EmailConfig,
    to: &str,
    subject: &str,
    body: &str,
) -> Result<(), String> {
    let email = Message::builder()
        .from(
            config
                .from_address
                .parse()
                .map_err(|e| format!("Invalid sender address: {}", e))?,
        )
        .to(to
            .parse()
            .map_err(|e| format!("Invalid recipient address: {}", e))?)
        .subject(subject)
        .header(ContentType::TEXT_PLAIN)
        .body(body.to_string())
        .map_err(|e| format!("Failed to build email: {}", e))?;

    let creds = Credentials::new(config.from_address.clone(), config.password.clone());

    let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay("smtp.gmail.com")
        .map_err(|e| format!("Failed to build SMTP transport: {}", e))?
        .credentials(creds)
        .build();

    mailer
        .send(email)
        .await
        .map_err(|e| format!("Failed to send email: {}", e))?;

    log::info!("Email sent successfully to {}", to);
    Ok(())
}
