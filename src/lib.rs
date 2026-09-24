use wasm_bindgen::prelude::*;

pub mod browser;
pub mod config;
pub mod github;
pub mod js_bridge;
pub mod notifications;
pub mod storage;

use config::Config;
use notifications::service::poll_and_update;

/// Initialise the background service worker.
/// Called from `background.js` as `__bg_start()` after `init()`.
#[wasm_bindgen]
pub async fn __bg_start() {
    web_sys::console::log_1(&"[github-notified] Background service worker initializing...".into());

    // Register alarm handler for periodic background check
    browser::on_alarm(|name| {
        if name == browser::alarms::ALARM_POLL_NOTIFICATIONS {
            web_sys::console::log_1(
                &"[github-notified] Alarm triggered, polling notifications...".into(),
            );
            wasm_bindgen_futures::spawn_local(async {
                let cfg = Config::load().await;
                let _ = poll_and_update(&cfg).await;
            });
        }
    });

    // Register notification click handler (opens the clicked thread URL)
    browser::on_notification_clicked(|target_url| {
        web_sys::console::log_1(
            &format!("[github-notified] Desktop notification clicked: {target_url}").into(),
        );
        wasm_bindgen_futures::spawn_local(async move {
            let cfg = Config::load().await;
            browser::open_or_focus_tab(&target_url, cfg.reuse_existing_tab).await;
        });
    });

    // Initial check and alarm registration
    let cfg = Config::load().await;
    browser::schedule_poll(cfg.poll_interval_mins as f64);
    let _ = poll_and_update(&cfg).await;

    web_sys::console::log_1(
        &"[github-notified] Background service worker initialization complete.".into(),
    );
}

/// Called from `background.js` to register the `runtime.onInstalled` listener.
/// Must be called synchronously (before any await), so it is a separate export.
#[wasm_bindgen]
pub fn __register_on_installed() {
    use wasm_bindgen::closure::Closure;
    let closure = Closure::wrap(Box::new(move |_details: JsValue| {
        web_sys::console::log_1(&"[github-notified] Extension installed/updated.".into());
        wasm_bindgen_futures::spawn_local(async {
            let cfg = Config::load().await;
            browser::schedule_poll(cfg.poll_interval_mins as f64);
            let _ = poll_and_update(&cfg).await;
        });
    }) as Box<dyn FnMut(JsValue)>);

    js_bridge::chrome_runtime_on_installed_add_listener(&closure);
    closure.forget();
}
