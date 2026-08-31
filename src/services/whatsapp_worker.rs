use headless_chrome::{Browser, LaunchOptions};
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::mpsc;
use urlencoding::encode;

use crate::models::whatsapp::WhatsAppMessage;
use crate::services::email_service::{EmailJob, enqueue_email};

/// Sends an email alert to joperez@gmail.com when a WhatsApp message fails.
/// Enqueues the notification into the background email queue.
pub fn notify_failure_by_email(recipient: &str, whatsapp_text: &str, error_detail: &str) {
    let to = "joperezd23@gmail.com".to_string();
    let subject = format!("⚠️ WhatsApp send failed → {}", recipient);
    let body = format!(
        "A WhatsApp message could not be delivered.\n\n\
         Recipient  : {}\n\
         Error      : {}\n\n\
         Original message\n\
         ─────────────────\n\
         {}\n",
        recipient, error_detail, whatsapp_text
    );

    let job = EmailJob {
        to: to.clone(),
        subject,
        body,
    };

    if let Err(e) = enqueue_email(job) {
        log::error!("Failed to enqueue WhatsApp-failure email: {}", e);
    } else {
        log::info!("WhatsApp-failure notification email queued for {}", to);
    }
}

/// Resolves the Chrome/Chromium executable path.
/// Priority order:
///   1. `CHROME_PATH` environment variable
///   2. Common system locations (Rocky/RHEL, Debian/Ubuntu, macOS)
///   3. Fallback: let `headless_chrome` auto-detect (may fail)
fn resolve_chrome_path() -> Option<PathBuf> {
    // 1. Env-var override (recommended for servers)
    if let Ok(p) = std::env::var("CHROME_PATH") {
        let path = PathBuf::from(p);
        if path.exists() {
            return Some(path);
        }
    }

    // 2. Well-known paths across distros
    let candidates = [
        "/usr/bin/chromium-browser",
        "/usr/bin/chromium",
        "/usr/bin/google-chrome",
        "/usr/bin/google-chrome-stable",
        "/usr/lib64/chromium-browser/chromium-browser", // Rocky/RHEL
        "/usr/lib/chromium-browser/chromium-browser",   // Debian
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
    ];

    for path in &candidates {
        let p = PathBuf::from(path);
        if p.exists() {
            return Some(p);
        }
    }

    None
}

/// Sleep for `total_secs` seconds while keeping the Chrome WebSocket alive.
///
/// `headless_chrome` (and Chrome itself) will drop the debugging connection if
/// no messages are exchanged for the `idle_browser_timeout` period. Even with a
/// long timeout set, doing a tiny JS `evaluate` every few seconds is a cheap,
/// reliable way to guarantee the connection is never considered idle.
fn keep_alive_sleep(tab: &std::sync::Arc<headless_chrome::Tab>, total_secs: u64) {
    let chunk = Duration::from_secs(5);
    let total = Duration::from_secs(total_secs);
    let start = std::time::Instant::now();
    while start.elapsed() < total {
        std::thread::sleep(chunk.min(total.saturating_sub(start.elapsed())));
        // No-op JS ping — just enough to reset the idle timer.
        let _ = tab.evaluate("1", false);
    }
}

/// Lazily starts the WhatsApp worker the first time a message needs to be sent.
///
/// Returns a `Sender` connected to the worker's channel, creating the worker
/// thread (and launching Chrome) only on the very first call. Subsequent calls
/// return a clone of the already-existing sender without touching Chrome.
///
/// This avoids crashing the server on startup in environments where Chrome is
/// not installed or a display is unavailable.
pub fn get_or_init_worker() -> Option<mpsc::Sender<WhatsAppMessage>> {
    use std::sync::{Arc, Mutex, OnceLock};

    // Global slot: holds the sender while the worker is alive, None otherwise.
    static WORKER_TX: OnceLock<Arc<Mutex<Option<mpsc::Sender<WhatsAppMessage>>>>> = OnceLock::new();

    let slot = WORKER_TX.get_or_init(|| Arc::new(Mutex::new(None)));
    let mut guard = slot.lock().unwrap();

    // Check if the sender is still live (worker running).
    if let Some(ref tx) = *guard {
        if !tx.is_closed() {
            return Some(tx.clone());
        }
        // Worker died — clear the slot so we can re-initialise.
        log::warn!("WhatsApp worker channel is closed. Re-initialising…");
        *guard = None;
    }

    // ── Bootstrap the worker ─────────────────────────────────────────────────
    log::info!("WhatsApp worker: initialising worker thread…");

    let chrome_path = match resolve_chrome_path() {
        Some(p) => {
            log::info!("Using Chrome binary: {}", p.display());
            p
        }
        None => {
            log::error!(
                "Could not find a Chrome/Chromium binary. \
                 Install chromium (`sudo dnf install chromium`) or set the \
                 CHROME_PATH environment variable. WhatsApp sending is disabled."
            );
            return None;
        }
    };

    let (tx, rx) = mpsc::channel::<WhatsAppMessage>(100);

    std::thread::spawn(move || {
        log::info!("WhatsApp worker thread started.");

        // Persist the Chrome user-data directory so the session survives restarts.
        let user_data_dir = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("whatsapp_profile");

        // ── Clean up stale Chrome singleton lock files ───────────────────────
        // A previous Chrome crash leaves behind SingletonLock / SingletonCookie /
        // SingletonSocket symlinks that prevent a new instance from using the
        // profile directory, which causes the QR code to never appear.
        for lock_file in &["SingletonLock", "SingletonCookie", "SingletonSocket"] {
            let lock_path = user_data_dir.join(lock_file);
            if lock_path.exists() || lock_path.is_symlink() {
                match std::fs::remove_file(&lock_path) {
                    Ok(_) => log::info!("Removed stale Chrome lock file: {}", lock_path.display()),
                    Err(e) => log::warn!("Could not remove {}: {}", lock_path.display(), e),
                }
            }
        }

        // ── Decide headless mode ─────────────────────────────────────────────
        // If WHATSAPP_HEADLESS=true is set explicitly, respect it.
        // Otherwise, fall back to headless when no X display is available
        // (common on servers), because Chrome crashes instantly without one.
        let env_headless = std::env::var("WHATSAPP_HEADLESS")
            .map(|v| v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);

        let display_available = std::env::var("DISPLAY")
            .map(|v| !v.is_empty())
            .unwrap_or(false);

        let headless = if env_headless {
            true
        } else if !display_available {
            log::warn!(
                "No DISPLAY found — running Chrome in headless mode. \
                 The QR code will NOT appear in a window. \
                 To scan it, set WHATSAPP_HEADLESS=false and run the server \
                 inside a desktop session (or forward the display with DISPLAY=:0)."
            );
            true
        } else {
            false
        };

        // ── Extra flags required on Linux servers ────────────────────────────
        // --disable-gpu            → avoids GPU process crashes in headless/server envs
        // --disable-dev-shm-usage  → /dev/shm is often too small in containers/VMs
        // --no-first-run           → skip first-run chrome setup dialogs
        // --no-default-browser-check → skip "make Chrome your default" dialog
        // --user-agent             → WhatsApp Web rejects "HeadlessChrome" in the UA;
        //                           we spoof a normal desktop Chrome UA so it loads properly.
        let ua = format!(
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 \
             (KHTML, like Gecko) Chrome/{ver} Safari/537.36",
            ver = "151.0.7922.173"
        );
        let ua_flag = format!("--user-agent={}", ua);
        let extra_args: Vec<std::ffi::OsString> = vec![
            "--user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36".into(),
            "--headless=new".into(),
            "--disable-gpu".into(),
            "--disable-dev-shm-usage".into(),
            "--no-sandbox".into(),
            "--disable-extensions".into(),
            "--disable-component-update".into(),
            "--disable-background-networking".into(),
            "--disable-sync".into(),
            "--disable-translate".into(),
            "--mute-audio".into(),
            "--no-first-run".into(),
            "--no-default-browser-check".into(),
            ua_flag.into(),
        ];
        let extra_args_refs: Vec<&std::ffi::OsStr> =
            extra_args.iter().map(|s| s.as_os_str()).collect();

        let launch_options = match LaunchOptions::default_builder()
            .path(Some(chrome_path))
            .headless(headless)
            .user_data_dir(Some(user_data_dir))
            .sandbox(false) // Required in Linux server environments
            .args(extra_args_refs)
            // Default is 30 s — Chrome disconnects during our 30 s rate-limit sleep.
            // Set to 10 minutes so the browser stays alive between messages.
            .idle_browser_timeout(Duration::from_secs(600))
            .build()
        {
            Ok(opts) => opts,
            Err(e) => {
                log::error!("Failed to build Chrome launch options: {:?}", e);
                return;
            }
        };

        let browser = match Browser::new(launch_options) {
            Ok(b) => {
                log::info!("Chrome browser launched successfully.");
                b
            }
            Err(e) => {
                log::error!(
                    "Failed to launch Chrome: {:?}. WhatsApp sending is disabled.",
                    e
                );
                return;
            }
        };

        let tab = match browser.new_tab() {
            Ok(t) => t,
            Err(e) => {
                log::error!("Failed to open a new Chrome tab: {:?}", e);
                return;
            }
        };

        log::info!("Navigating to WhatsApp Web…");
        if let Err(e) = tab.navigate_to("https://web.whatsapp.com") {
            log::error!("Failed to navigate to WhatsApp Web: {:?}", e);
            return;
        }

        if headless {
            log::info!("────────────────────────────────────────────────────────");
            log::info!("  ACTION REQUIRED (headless mode — no display detected)");
            log::info!(
                "  Open http://127.0.0.1:{}/api/whatsapp/qr in your",
                std::env::var("SERVER_PORT").unwrap_or_else(|_| "8000".into())
            );
            log::info!("  browser. The QR code image refreshes every ~3 s.");
            log::info!("  Scan it with WhatsApp → Linked Devices → Link a Device.");
            log::info!("  You have 120 seconds.");
            log::info!("────────────────────────────────────────────────────────");
        } else {
            log::info!("────────────────────────────────────────────────────────");
            log::info!("  ACTION REQUIRED: Open the Chrome window that just");
            log::info!("  appeared and scan the WhatsApp QR code with your");
            log::info!("  phone → WhatsApp → Linked Devices → Link a Device.");
            log::info!("  You have 120 seconds.");
            log::info!("────────────────────────────────────────────────────────");
        }

        let qr_timeout = Duration::from_secs(120);
        let poll_interval = Duration::from_secs(3);
        let start = std::time::Instant::now();
        let mut logged_in = false;

        // ── Determine screenshot path for headless QR display ───────────────
        let qr_screenshot_path = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("whatsapp_qr.png");

        // Give WhatsApp Web a moment to render before we start polling.
        std::thread::sleep(Duration::from_secs(5));

        while start.elapsed() < qr_timeout {
            // The side-panel / chat list is rendered as `#pane-side` once logged in.
            if tab
                .find_element("#pane-side")
                .or_else(|_| tab.find_element("[data-testid='chat-list']"))
                .is_ok()
            {
                logged_in = true;
                // Remove the QR screenshot now that we're logged in.
                let _ = std::fs::remove_file(&qr_screenshot_path);
                break;
            }

            // In headless mode the user can't see the browser window, so we
            // capture a screenshot of the page every poll cycle and write it to
            // `whatsapp_qr.png` in the working directory.
            // The GET /api/whatsapp/qr endpoint serves this file so the user can
            // open it in any browser to see the live QR code.
            if headless {
                match tab.capture_screenshot(
                    headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption::Png,
                    None,
                    None,
                    true,
                ) {
                    Ok(png_bytes) => {
                        if let Err(e) = std::fs::write(&qr_screenshot_path, &png_bytes) {
                            log::warn!("Could not save QR screenshot: {}", e);
                        } else {
                            log::info!(
                                "QR code screenshot saved → open http://127.0.0.1:{}/api/whatsapp/qr in your browser to scan it.",
                                std::env::var("SERVER_PORT").unwrap_or_else(|_| "8000".into())
                            );
                        }
                    }
                    Err(e) => log::warn!("Screenshot failed: {:?}", e),
                }
            }

            std::thread::sleep(poll_interval);
        }

        if !logged_in {
            log::error!(
                "WhatsApp QR code was not scanned within 120 s. \
                 Worker is shutting down. Restart the server and try again."
            );
            let _ = std::fs::remove_file(&qr_screenshot_path);
            return;
        }

        log::info!("WhatsApp session confirmed ✓ — worker is ready.");
        log::info!(
            "WhatsApp worker ready. Rate limit enforced: 30 s between messages (≤ 2 msg/min)."
        );

        let mut tab = tab; // make mutable for reconnection
        let mut receiver = rx;
        while let Some(msg) = receiver.blocking_recv() {
            log::info!("Dequeued WhatsApp message → to={}", msg.to);

            // Build the WhatsApp Web deep-link with pre-filled text.
            let url = format!(
                "https://web.whatsapp.com/send?phone={}&text={}",
                msg.to,
                encode(&msg.text)
            );

            if let Err(e) = tab.navigate_to(&url) {
                log::error!("Navigation error for {}: {:?}", msg.to, e);

                // The Chrome WebSocket connection dropped (e.g. the browser
                // was idle-killed or crashed). Try to recover by opening a
                // fresh tab — no re-login needed since the session is stored
                // in `whatsapp_profile/`.
                log::warn!("Connection closed — attempting to open a new tab and retry…");
                match browser.new_tab() {
                    Ok(new_tab) => {
                        tab = new_tab;
                        log::info!("New tab opened. Retrying navigation…");
                        if let Err(e2) = tab.navigate_to(&url) {
                            let err_str = format!("{:?}", e2);
                            log::error!(
                                "Retry navigation also failed for {}: {}. Skipping message.",
                                msg.to,
                                err_str
                            );
                            notify_failure_by_email(&msg.to, &msg.text, &err_str);
                            keep_alive_sleep(&tab, 30);
                            continue;
                        }
                    }
                    Err(e2) => {
                        let err_str = format!(
                            "Could not open a new tab (browser may have crashed): {:?}",
                            e2
                        );
                        log::error!("{}. Worker is shutting down — restart the server.", err_str);
                        notify_failure_by_email(&msg.to, &msg.text, &err_str);
                        break;
                    }
                }
            }

            // ── Step 1: dismiss "Continue to Chat" popup ─────────────────────
            // When using the /send?phone=... deep-link, WhatsApp Web shows a
            // confirmation dialog before opening the chat. We must click it or
            // the send button will never appear.
            log::info!("Checking for 'Continue to Chat' popup…");
            let popup_selectors = [
                "[data-testid='popup-confirm-button']",
                "div[role='button'][class*='confirm']",
                // Fallback: any button whose visible text contains "Continue"
                // (evaluated via JS since headless_chrome has no :contains support)
            ];
            let mut popup_dismissed = false;
            for sel in &popup_selectors {
                if let Ok(btn) =
                    tab.wait_for_element_with_custom_timeout(sel, Duration::from_secs(5))
                    && btn.click().is_ok()
                {
                    log::info!("'Continue to Chat' popup dismissed.");
                    popup_dismissed = true;
                    std::thread::sleep(Duration::from_secs(2));
                    break;
                }
            }
            // JS fallback: find a button containing the word "Continue"
            if !popup_dismissed {
                let _ = tab.evaluate(
                    r#"(function() {
                        var buttons = document.querySelectorAll('div[role="button"], button');
                        for (var i = 0; i < buttons.length; i++) {
                            if (buttons[i].innerText && buttons[i].innerText.indexOf('Continue') !== -1) {
                                buttons[i].click();
                                return true;
                            }
                        }
                        return false;
                    })()"#,
                    false,
                );
                std::thread::sleep(Duration::from_secs(2));
            }

            // ── Step 2: find and click the send button ───────────────────────
            // WhatsApp Web has updated its selectors over time; try several.
            log::info!("Waiting for send button (up to 30 s)…");
            let send_selectors = [
                "[data-testid='send']",
                "span[data-icon='send']",
                "button[aria-label='Send']",
                "[data-icon='send']",
                "[data-testid='enviar']",
                "span[data-icon='enviar']",
                "button[aria-label='Enviar']",
                "[data-icon='enviar']",
            ];

            let mut sent = false;
            'send: for sel in &send_selectors {
                if let Ok(btn) =
                    tab.wait_for_element_with_custom_timeout(sel, Duration::from_secs(30))
                {
                    match btn.click() {
                        Ok(_) => {
                            std::thread::sleep(Duration::from_secs(3));
                            log::info!("Message sent successfully to {}", msg.to);
                            sent = true;
                            break 'send;
                        }
                        Err(e) => {
                            log::warn!("Selector '{}' found but click failed: {:?}", sel, e);
                        }
                    }
                }
            }
            if !sent {
                let err_str = "Send button not found after trying all selectors. \
                    The number may be invalid or WhatsApp Web changed its UI."
                    .to_string();
                log::error!(
                    "Send button not found for {} after trying all selectors. \
                     The number may be invalid or WhatsApp Web changed its UI.",
                    msg.to
                );
                notify_failure_by_email(&msg.to, &msg.text, &err_str);
            }

            // ─── Rate-limit delay ───────────────────────────────────────────
            // 30 seconds between messages guarantees ≤ 2 messages per minute,
            // and mimics human cadence to reduce ban risk.
            // We ping the tab with a no-op JS call every 5 s to keep the
            // Chrome WebSocket alive instead of a single blocking sleep.
            log::info!("Rate-limit pause: 30 s before next message…");
            keep_alive_sleep(&tab, 30);
        }

        log::warn!("WhatsApp worker channel closed — worker is shutting down.");
    });

    *guard = Some(tx.clone());
    Some(tx)
}
