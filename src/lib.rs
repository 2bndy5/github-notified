use leptos::prelude::*;

pub mod browser;
pub mod config;
pub mod github;
pub mod notifications;
pub mod ui;

use config::Config;
use notifications::service::poll_and_update;

#[oxichrome::extension(
    name = "GitHub Notified",
    version = "0.1.0",
    description = "Checks GitHub notifications with fine-grained PAT, badge count, and desktop alerts",
    permissions = ["storage", "alarms", "notifications", "tabs"]
)]
pub struct GitHubNotifiedExtension;

#[oxichrome::background]
async fn start() {
    oxichrome::log!("[github-notified] Background service worker initializing...");

    // Register alarm handler for periodic background check
    browser::on_alarm(|name| {
        if name == browser::alarms::ALARM_POLL_NOTIFICATIONS {
            oxichrome::log!("[github-notified] Alarm triggered, polling notifications...");
            wasm_bindgen_futures::spawn_local(async {
                let cfg = Config::load().await;
                let _ = poll_and_update(&cfg).await;
            });
        }
    });

    // Register notification click handler (opens the clicked thread or notifications page)
    browser::on_notification_clicked(|target_url| {
        oxichrome::log!(
            "[github-notified] Desktop notification clicked: {}",
            target_url
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

    oxichrome::log!("[github-notified] Background service worker initialization complete.");
}

#[oxichrome::on(runtime::on_installed)]
async fn handle_install(details: oxichrome::__private::wasm_bindgen::JsValue) {
    oxichrome::log!(
        "[github-notified] Extension installed/updated: {:?}",
        details
    );
    let cfg = Config::load().await;
    browser::schedule_poll(cfg.poll_interval_mins as f64);
    let _ = poll_and_update(&cfg).await;
}

#[oxichrome::popup]
fn Popup() -> impl IntoView {
    view! {
        <ui::PopupView />
    }
}

#[oxichrome::options_page]
fn Options() -> impl IntoView {
    view! {
        <ui::OptionsView />
    }
}
