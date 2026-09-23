use crate::browser::tabs::open_or_focus_tab;
use crate::config::Config;
#[cfg(target_arch = "wasm32")]
use crate::config::STORAGE_KEY_NOTIFICATIONS_CACHE;
use crate::github::models::NotificationSummary;
#[cfg(target_arch = "wasm32")]
use crate::github::models::NotificationsCache;
use crate::notifications::service::poll_and_update;
use leptos::prelude::*;

#[component]
pub fn PopupView() -> impl IntoView {
    let unread_count = RwSignal::new(0usize);
    let notifications = RwSignal::new(Vec::<NotificationSummary>::new());
    let has_token = RwSignal::new(true);
    let is_refreshing = RwSignal::new(false);
    let error_msg = RwSignal::new(Option::<String>::None);
    let notifications_url = RwSignal::new("https://github.com/notifications".to_string());

    // Load initial cache and config
    Effect::new(move || {
        wasm_bindgen_futures::spawn_local(async move {
            let cfg = Config::load().await;
            has_token.set(cfg.has_token());
            notifications_url.set(cfg.notifications_url());

            #[cfg(target_arch = "wasm32")]
            if let Ok(Some(cache)) =
                oxichrome::storage::get::<NotificationsCache>(STORAGE_KEY_NOTIFICATIONS_CACHE).await
            {
                unread_count.set(cache.count);
                notifications.set(cache.notifications);
            }
        });
    });

    let on_refresh = move |_| {
        is_refreshing.set(true);
        error_msg.set(None);
        wasm_bindgen_futures::spawn_local(async move {
            let cfg = Config::load().await;
            match poll_and_update(&cfg).await {
                Ok(count) => {
                    unread_count.set(count);
                    #[cfg(target_arch = "wasm32")]
                    if let Ok(Some(cache)) = oxichrome::storage::get::<NotificationsCache>(
                        STORAGE_KEY_NOTIFICATIONS_CACHE,
                    )
                    .await
                    {
                        notifications.set(cache.notifications);
                    }
                }
                Err(e) => {
                    error_msg.set(Some(e));
                }
            }
            is_refreshing.set(false);
        });
    };

    let on_open_notifications = move |_| {
        let url = notifications_url.get_untracked();
        wasm_bindgen_futures::spawn_local(async move {
            let cfg = Config::load().await;
            open_or_focus_tab(&url, cfg.reuse_existing_tab).await;
        });
    };

    let on_open_options = move |_| {
        #[cfg(target_arch = "wasm32")]
        {
            let options_url = oxichrome::runtime::get_url("options.html");
            wasm_bindgen_futures::spawn_local(async move {
                let _ = oxichrome::tabs::create::<_, serde_json::Value>(&serde_json::json!({
                    "url": options_url,
                    "active": true
                }))
                .await;
            });
        }
    };

    let on_item_click = move |web_url: String| {
        wasm_bindgen_futures::spawn_local(async move {
            let cfg = Config::load().await;
            open_or_focus_tab(&web_url, cfg.reuse_existing_tab).await;
        });
    };

    view! {
        <div style="width: 360px; max-height: 480px; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif; background: #ffffff; color: #1f2328; display: flex; flex-direction: column; overflow: hidden; border-radius: 6px; box-shadow: 0 4px 12px rgba(0,0,0,0.15);">
            // Top Bar
            <div style="display: flex; align-items: center; justify-content: space-between; padding: 12px 16px; border-bottom: 1px solid #d0d7de; background: #f6f8fa;">
                <div style="display: flex; align-items: center; gap: 8px;">
                    <svg height="20" width="20" viewBox="0 0 16 16" fill="currentColor">
                        <path d="M8 0c4.42 0 8 3.58 8 8a8.013 8.013 0 0 1-5.45 7.59c-.4.08-.55-.17-.55-.38 0-.27.01-1.13.01-2.2 0-.75-.25-1.23-.54-1.48 1.78-.2 3.65-.88 3.65-3.95 0-.88-.31-1.59-.82-2.15.08-.2.36-1.02-.08-2.12 0 0-.67-.22-2.2.82-.64-.18-1.32-.27-2-.27-.68 0-1.36.09-2 .27-1.53-1.03-2.2-.82-2.2-.82-.44 1.1-.16 1.92-.08 2.12-.51.56-.82 1.28-.82 2.15 0 3.06 1.86 3.75 3.64 3.95-.23.2-.44.55-.51 1.07-.46.21-1.61.55-2.33-.66-.15-.24-.6-.83-1.23-.82-.67.01-.27.38.01.53.34.19.73.9.82 1.13.16.45.68 1.31 2.69.94 0 .67.01 1.3.01 1.49 0 .21-.15.45-.55.38A7.995 7.995 0 0 1 0 8c0-4.42 3.58-8 8-8Z"></path>
                    </svg>
                    <span style="font-weight: 600; font-size: 14px;">"Notifications"</span>
                    <span style="background: #afb8c133; color: #1f2328; font-size: 12px; font-weight: 600; padding: 2px 7px; border-radius: 10px;">
                        {move || unread_count.get()}
                    </span>
                </div>

                <div style="display: flex; gap: 8px;">
                    <button
                        on:click=on_refresh
                        title="Refresh notifications"
                        style="background: none; border: 1px solid #d0d7de; border-radius: 4px; padding: 4px 8px; cursor: pointer; font-size: 12px; display: flex; align-items: center; color: #24292f;"
                    >
                        {move || if is_refreshing.get() { "..." } else { "↻" }}
                    </button>
                    <button
                        on:click=on_open_options
                        title="Options / Settings"
                        style="background: none; border: 1px solid #d0d7de; border-radius: 4px; padding: 4px 8px; cursor: pointer; font-size: 12px; color: #24292f;"
                    >
                        "⚙"
                    </button>
                </div>
            </div>

            // Error banner if any
            {move || error_msg.get().map(|err| view! {
                <div style="background: #ffebe9; color: #cf222e; padding: 8px 12px; font-size: 12px; border-bottom: 1px solid #ffcecb;">
                    {err}
                </div>
            })}

            // Body
            <div style="flex: 1; overflow-y: auto; max-height: 360px;">
                {move || {
                    if !has_token.get() {
                        view! {
                            <div style="padding: 24px; text-align: center; color: #57606a;">
                                <p style="font-weight: 600; margin-bottom: 6px; color: #1f2328;">"Token Required"</p>
                                <p style="font-size: 12px; margin-bottom: 16px;">
                                    "Please provide a GitHub fine-grained Personal Access Token to view your notifications."
                                </p>
                                <button
                                    on:click=on_open_options
                                    style="background: #1f883d; color: white; border: none; padding: 6px 14px; border-radius: 6px; font-weight: 600; font-size: 12px; cursor: pointer;"
                                >
                                    "Configure in Options"
                                </button>
                            </div>
                        }.into_any()
                    } else if notifications.get().is_empty() {
                        view! {
                            <div style="padding: 32px 16px; text-align: center; color: #57606a;">
                                <div style="font-size: 24px; margin-bottom: 8px;">"✓"</div>
                                <p style="font-weight: 600; color: #1f2328; margin-bottom: 4px;">"All caught up!"</p>
                                <p style="font-size: 12px;">"You have no unread notifications."</p>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div style="display: flex; flex-direction: column;">
                                {notifications.get().into_iter().map(|item| {
                                    let url = item.web_url.clone();
                                    view! {
                                        <div
                                            on:click=move |_| on_item_click(url.clone())
                                            style="padding: 10px 14px; border-bottom: 1px solid #d0d7de22; cursor: pointer; transition: background 0.15s ease;"
                                        >
                                            <div style="display: flex; justify-content: space-between; align-items: baseline; margin-bottom: 2px;">
                                                <span style="font-size: 11px; color: #57606a; font-weight: 500;">
                                                    {item.repo_full_name}
                                                </span>
                                                <span style="font-size: 10px; background: #ddf4ff; color: #0969da; padding: 1px 5px; border-radius: 4px;">
                                                    {item.type_name}
                                                </span>
                                            </div>
                                            <div style="font-size: 13px; font-weight: 600; color: #0969da; text-decoration: none; line-height: 1.3;">
                                                {item.title}
                                            </div>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any()
                    }
                }}
            </div>

            // Footer Button
            <div style="padding: 10px 14px; border-top: 1px solid #d0d7de; background: #f6f8fa;">
                <button
                    on:click=on_open_notifications
                    style="width: 100%; background: #24292f; color: white; border: none; padding: 8px 12px; border-radius: 6px; font-weight: 600; font-size: 13px; cursor: pointer; text-align: center;"
                >
                    "Open GitHub Notifications"
                </button>
            </div>
        </div>
    }
}
