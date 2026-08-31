use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;
use tokio::sync::mpsc;

/// Represents an email job queued for dispatch.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmailJob {
    pub to: String,
    pub subject: String,
    pub body: String,
}

/// Credentials and settings for Gmail SMTP sender.
#[derive(Clone, Debug)]
pub struct EmailConfig {
    pub from_address: String,
    pub password: String,
    pub send_interval_secs: u64,
}

impl EmailConfig {
    /// Load Gmail credentials and settings from environment variables.
    pub fn from_env() -> Self {
        let from_address =
            std::env::var("GMAIL_USER").unwrap_or_else(|_| "joperezd23@gmail.com".to_string());
        let password =
            std::env::var("GMAIL_PASSWORD").unwrap_or_else(|_| "lalhhdutfowyiyvq".to_string());
        let send_interval_secs = std::env::var("EMAIL_INTERVAL_SECS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(15);

        Self {
            from_address,
            password,
            send_interval_secs,
        }
    }
}

/// Global queue sender slot and pending count tracker.
static EMAIL_TX: OnceLock<Arc<Mutex<Option<mpsc::UnboundedSender<EmailJob>>>>> = OnceLock::new();
static QUEUE_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Returns (pending_count, interval_seconds, is_worker_active).
pub fn get_queue_status() -> (usize, u64, bool) {
    let count = QUEUE_COUNTER.load(Ordering::SeqCst);
    let config = EmailConfig::from_env();
    let slot = EMAIL_TX.get_or_init(|| Arc::new(Mutex::new(None)));
    let is_active = match slot.lock() {
        Ok(guard) => guard.as_ref().map_or(false, |tx| !tx.is_closed()),
        Err(_) => false,
    };
    (count, config.send_interval_secs, is_active)
}

/// Initializes the background email worker thread (if not already running).
pub fn get_or_init_email_worker() -> bool {
    let slot = EMAIL_TX.get_or_init(|| Arc::new(Mutex::new(None)));
    let mut guard = match slot.lock() {
        Ok(g) => g,
        Err(e) => {
            log::error!("Failed to acquire lock on EMAIL_TX: {}", e);
            return false;
        }
    };

    if let Some(ref tx) = *guard {
        if !tx.is_closed() {
            return true;
        }
        log::warn!("Email worker channel is closed. Re-initialising worker...");
        *guard = None;
    }

    log::info!("Initialising Email background worker thread...");
    let config = EmailConfig::from_env();
    let (tx, mut rx) = mpsc::unbounded_channel::<EmailJob>();

    std::thread::spawn(move || {
        let rt = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(r) => r,
            Err(e) => {
                log::error!("Failed to build Tokio runtime for Email worker: {}", e);
                return;
            }
        };

        rt.block_on(async move {
            let interval_secs = config.send_interval_secs;
            log::info!(
                "Email worker started successfully. Cadence: 1 email every {} seconds.",
                interval_secs
            );

            while let Some(job) = rx.recv().await {
                log::info!(
                    "Dequeued email for {} (subject: '{}'). Sending via Gmail SMTP...",
                    job.to,
                    job.subject
                );

                if let Err(e) = send_email(&config, &job.to, &job.subject, &job.body).await {
                    log::error!("Failed to send email to {}: {}", job.to, e);
                } else {
                    log::info!("Email successfully delivered to {}", job.to);
                }

                QUEUE_COUNTER.fetch_sub(1, Ordering::SeqCst);
                let remaining = QUEUE_COUNTER.load(Ordering::SeqCst);

                log::info!(
                    "Email worker: waiting {}s before dispatching next email ({} remaining in queue)...",
                    interval_secs,
                    remaining
                );

                tokio::time::sleep(Duration::from_secs(interval_secs)).await;
            }

            log::warn!("Email worker queue closed — worker is shutting down.");
        });
    });

    *guard = Some(tx);
    true
}

/// Enqueues a single email job to be sent at the 15-second interval.
/// Returns the new total number of items in the queue.
pub fn enqueue_email(job: EmailJob) -> Result<usize, String> {
    get_or_init_email_worker();

    let slot = EMAIL_TX.get_or_init(|| Arc::new(Mutex::new(None)));
    let guard = slot.lock().map_err(|e| format!("Lock error: {}", e))?;

    if let Some(ref tx) = *guard {
        QUEUE_COUNTER.fetch_add(1, Ordering::SeqCst);
        tx.send(job).map_err(|e| {
            QUEUE_COUNTER.fetch_sub(1, Ordering::SeqCst);
            format!("Failed to enqueue email: {}", e)
        })?;
        let count = QUEUE_COUNTER.load(Ordering::SeqCst);
        Ok(count)
    } else {
        Err("Email worker sender not available".to_string())
    }
}

/// Enqueues a batch of email jobs to be sent sequentially every 15 seconds.
/// Returns the new total number of items in the queue.
pub fn enqueue_batch(jobs: Vec<EmailJob>) -> Result<usize, String> {
    get_or_init_email_worker();

    let slot = EMAIL_TX.get_or_init(|| Arc::new(Mutex::new(None)));
    let guard = slot.lock().map_err(|e| format!("Lock error: {}", e))?;

    if let Some(ref tx) = *guard {
        for job in jobs {
            QUEUE_COUNTER.fetch_add(1, Ordering::SeqCst);
            if let Err(e) = tx.send(job) {
                QUEUE_COUNTER.fetch_sub(1, Ordering::SeqCst);
                return Err(format!("Failed to enqueue email in batch: {}", e));
            }
        }
        let count = QUEUE_COUNTER.load(Ordering::SeqCst);
        Ok(count)
    } else {
        Err("Email worker sender not available".to_string())
    }
}

/// Send a plain-text email via Gmail SMTP (TLS, port 587).
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

    Ok(())
}

