use crate::browser::action;
#[cfg(target_arch = "wasm32")]
use crate::browser::notifications;
use crate::config::Config;
#[cfg(target_arch = "wasm32")]
use crate::config::{STORAGE_KEY_LAST_ERROR, STORAGE_KEY_NOTIFICATIONS_CACHE};
use crate::github::client::GitHubClient;
#[cfg(target_arch = "wasm32")]
use crate::github::models::{NotificationSummary, NotificationsCache};
#[cfg(target_arch = "wasm32")]
use crate::github::urls::notification_to_web_url;
#[cfg(target_arch = "wasm32")]
use crate::notifications::filter::{compute_new_notifications, filter_notifications};

/// Primary routine to check notifications from GitHub and update browser badge and state
pub async fn poll_and_update(config: &Config) -> Result<usize, String> {
    if !config.has_token() {
        action::set_badge_text("!");
        action::set_badge_color("#cf222e");
        action::set_title("GitHub Notified: Please set your Personal Access Token in Options.");
        return Err("No token configured".to_string());
    }

    let client = GitHubClient::new(&config.token, &config.api_url);

    #[cfg(target_arch = "wasm32")]
    {
        // 1. Load previous cache to compute newly arrived notifications
        let prev_cache =
            oxichrome::storage::get::<NotificationsCache>(STORAGE_KEY_NOTIFICATIONS_CACHE)
                .await
                .unwrap_or(None)
                .unwrap_or_default();

        let prev_ids: Vec<String> = prev_cache
            .notifications
            .iter()
            .map(|n| n.id.clone())
            .collect();

        // 2. Fetch notifications from GitHub API
        let threads = match client
            .fetch_notifications(config.only_participating, None)
            .await
        {
            Ok(t) => t,
            Err(err) => {
                oxichrome::log!("[github-notified] Polling error: {}", err);
                action::set_badge_text("!");
                action::set_badge_color("#cf222e");
                action::set_title(&format!("GitHub Notified Error: {err}"));
                let _ = oxichrome::storage::set(STORAGE_KEY_LAST_ERROR, &err).await;
                return Err(err);
            }
        };

        // Clear any previous error
        let _ = oxichrome::storage::remove(STORAGE_KEY_LAST_ERROR).await;

        // 3. Apply repository and participation filters
        let filtered = filter_notifications(&threads, config);
        let count = filtered.len();

        // 4. Update Toolbar Badge
        if count == 0 {
            action::clear_badge();
            action::set_title("GitHub Notified: No unread notifications");
        } else {
            let count_display = if count > 99 {
                "99+".to_string()
            } else {
                count.to_string()
            };
            action::set_badge_text(&count_display);
            action::set_badge_color("#0969da"); // GitHub accent blue
            action::set_title(&format!("GitHub Notified: {count} unread notifications"));
        }

        // 5. Desktop Notifications for new items
        if config.show_desktop_notifications && !prev_ids.is_empty() {
            let new_items = compute_new_notifications(&filtered, &prev_ids);
            for item in new_items {
                let notif_title = format!("{}: {}", item.repository.full_name, item.subject.title);
                let notif_body = format!("Reason: {} ({})", item.reason, item.subject.type_name);
                let web_url = notification_to_web_url(item, &config.root_url);
                notifications::show_desktop_notification(
                    &web_url,
                    &notif_title,
                    &notif_body,
                    "icon-48.png",
                );
            }
        }

        // 6. Cache summaries for quick popup display
        let summaries: Vec<NotificationSummary> = filtered
            .iter()
            .map(|t| NotificationSummary {
                id: t.id.clone(),
                repo_full_name: t.repository.full_name.clone(),
                title: t.subject.title.clone(),
                type_name: t.subject.type_name.clone(),
                reason: t.reason.clone(),
                updated_at: t.updated_at.clone(),
                web_url: notification_to_web_url(t, &config.root_url),
            })
            .collect();

        let new_cache = NotificationsCache {
            count,
            last_checked: js_sys::Date::new_0().to_iso_string().into(),
            notifications: summaries,
        };

        let _ = oxichrome::storage::set(STORAGE_KEY_NOTIFICATIONS_CACHE, &new_cache).await;

        Ok(count)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (client, config);
        Ok(0)
    }
}
