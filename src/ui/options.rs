use crate::browser::alarms::schedule_poll;
use crate::config::{Config, RepoFilterMode, DEFAULT_API_URL, DEFAULT_ROOT_URL};
#[cfg(target_arch = "wasm32")]
use crate::github::client::GitHubClient;
use crate::notifications::service::poll_and_update;
use leptos::prelude::*;

#[component]
pub fn OptionsView() -> impl IntoView {
    let token = RwSignal::new(String::new());
    let api_url = RwSignal::new(DEFAULT_API_URL.to_string());
    let root_url = RwSignal::new(DEFAULT_ROOT_URL.to_string());
    let only_participating = RwSignal::new(false);
    let poll_interval_mins = RwSignal::new(1u32);
    let show_desktop_notifications = RwSignal::new(true);
    let reuse_existing_tab = RwSignal::new(true);
    let repo_filter_mode = RwSignal::new(RepoFilterMode::None);
    let repo_filter_text = RwSignal::new(String::new());

    let test_status = RwSignal::new(Option::<Result<String, String>>::None);
    let is_testing = RwSignal::new(false);
    let save_message = RwSignal::new(Option::<String>::None);

    // Load saved settings
    Effect::new(move || {
        wasm_bindgen_futures::spawn_local(async move {
            let cfg = Config::load().await;
            token.set(cfg.token);
            api_url.set(cfg.api_url);
            root_url.set(cfg.root_url);
            only_participating.set(cfg.only_participating);
            poll_interval_mins.set(cfg.poll_interval_mins);
            show_desktop_notifications.set(cfg.show_desktop_notifications);
            reuse_existing_tab.set(cfg.reuse_existing_tab);
            repo_filter_mode.set(cfg.repo_filter_mode);
            repo_filter_text.set(cfg.repo_filter_list.join("\n"));
        });
    });

    let on_test_connection = move |_| {
        let tok = token.get_untracked();
        let api = api_url.get_untracked();
        is_testing.set(true);
        test_status.set(None);

        wasm_bindgen_futures::spawn_local(async move {
            #[cfg(target_arch = "wasm32")]
            let res = {
                let client = GitHubClient::new(tok, api);
                client.validate_token().await
            };
            #[cfg(not(target_arch = "wasm32"))]
            let res = {
                let _ = (tok, api);
                Ok("Test mode".to_string())
            };

            test_status.set(Some(res));
            is_testing.set(false);
        });
    };

    let on_save = move |_| {
        let list: Vec<String> = repo_filter_text
            .get_untracked()
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let cfg = Config {
            token: token.get_untracked().trim().to_string(),
            api_url: api_url.get_untracked().trim().to_string(),
            root_url: root_url.get_untracked().trim().to_string(),
            only_participating: only_participating.get_untracked(),
            poll_interval_mins: poll_interval_mins.get_untracked().max(1),
            show_desktop_notifications: show_desktop_notifications.get_untracked(),
            play_sound: false,
            reuse_existing_tab: reuse_existing_tab.get_untracked(),
            repo_filter_mode: repo_filter_mode.get_untracked(),
            repo_filter_list: list,
        };

        wasm_bindgen_futures::spawn_local(async move {
            let _ = cfg.save().await;
            schedule_poll(cfg.poll_interval_mins as f64);
            let _ = poll_and_update(&cfg).await;
            save_message.set(Some("Settings saved successfully!".to_string()));
        });
    };

    view! {
        <div style="max-width: 680px; margin: 40px auto; padding: 24px; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif; color: #1f2328; line-height: 1.5;">
            // Header
            <div style="display: flex; align-items: center; gap: 12px; margin-bottom: 24px; border-bottom: 1px solid #d0d7de; padding-bottom: 16px;">
                <svg height="32" width="32" viewBox="0 0 16 16" fill="currentColor">
                    <path d="M8 0c4.42 0 8 3.58 8 8a8.013 8.013 0 0 1-5.45 7.59c-.4.08-.55-.17-.55-.38 0-.27.01-1.13.01-2.2 0-.75-.25-1.23-.54-1.48 1.78-.2 3.65-.88 3.65-3.95 0-.88-.31-1.59-.82-2.15.08-.2.36-1.02-.08-2.12 0 0-.67-.22-2.2.82-.64-.18-1.32-.27-2-.27-.68 0-1.36.09-2 .27-1.53-1.03-2.2-.82-2.2-.82-.44 1.1-.16 1.92-.08 2.12-.51.56-.82 1.28-.82 2.15 0 3.06 1.86 3.75 3.64 3.95-.23.2-.44.55-.51 1.07-.46.21-1.61.55-2.33-.66-.15-.24-.6-.83-1.23-.82-.67.01-.27.38.01.53.34.19.73.9.82 1.13.16.45.68 1.31 2.69.94 0 .67.01 1.3.01 1.49 0 .21-.15.45-.55.38A7.995 7.995 0 0 1 0 8c0-4.42 3.58-8 8-8Z"></path>
                </svg>
                <div>
                    <h1 style="font-size: 24px; font-weight: 600; margin: 0;">"GitHub Notified Settings"</h1>
                    <p style="font-size: 13px; color: #57606a; margin: 0;">"Ported to Rust with oxichrome & Manifest V3"</p>
                </div>
            </div>

            // Save feedback banner
            {move || save_message.get().map(|msg| view! {
                <div style="background: #dafbe1; border: 1px solid #4ac26b; color: #1a7f37; padding: 12px 16px; border-radius: 6px; margin-bottom: 20px; font-size: 14px; font-weight: 500;">
                    {msg}
                </div>
            })}

            // Section 1: Authentication & Token
            <div style="background: #ffffff; border: 1px solid #d0d7de; border-radius: 6px; padding: 20px; margin-bottom: 24px;">
                <h2 style="font-size: 16px; font-weight: 600; margin-top: 0; margin-bottom: 12px;">"GitHub Personal Access Token (Classic)"</h2>

                <div style="background: #f6f8fa; border: 1px solid #d0d7de; border-radius: 6px; padding: 14px; margin-bottom: 16px; font-size: 13px;">
                    <div style="font-weight: 600; margin-bottom: 6px;">"How to create a Classic PAT:"</div>
                    <ol style="margin: 0; padding-left: 20px; line-height: 1.6;">
                        <li>"Visit " <a href="https://github.com/settings/tokens" target="_blank" style="color: #0969da;">"GitHub Settings → Developer Settings → Personal access tokens → Tokens (classic)"</a></li>
                        <li>"Click " <b>"Generate new token (classic)"</b> " and set an expiration."</li>
                        <li>"Under " <b>"Select scopes"</b> ", check " <span style="background: #ddf4ff; color: #0969da; padding: 2px 6px; border-radius: 4px; font-weight: 600;">"notifications"</span>"."</li>
                        <li>"Click Generate and paste your token below."</li>
                    </ol>
                    <div style="margin-top: 10px; font-size: 12px; color: #cf222e; font-weight: 500;">"⚠ The GitHub REST API does not support fine-grained PATs for notifications. Use a classic PAT (ghp_...)."</div>
                </div>

                <label style="display: block; font-weight: 600; font-size: 13px; margin-bottom: 6px;">
                    "Personal Access Token (ghp_...):"
                </label>
                <div style="display: flex; gap: 8px; margin-bottom: 12px;">
                    <input
                        type="password"
                        placeholder="ghp_..."
                        prop:value=move || token.get()
                        on:input=move |ev| token.set(event_target_value(&ev))
                        style="flex: 1; padding: 7px 12px; border: 1px solid #d0d7de; border-radius: 6px; font-size: 13px; font-family: monospace;"
                    />
                    <button
                        on:click=on_test_connection
                        style="background: #f6f8fa; border: 1px solid #d0d7de; border-radius: 6px; padding: 7px 16px; font-weight: 600; font-size: 13px; cursor: pointer;"
                    >
                        {move || if is_testing.get() { "Testing..." } else { "Test Connection" }}
                    </button>
                </div>

                {move || test_status.get().map(|res| match res {
                    Ok(msg) => view! {
                        <div style="color: #1a7f37; font-size: 13px; font-weight: 500; display: flex; align-items: center; gap: 6px;">
                            <span>"✓"</span> {msg}
                        </div>
                    }.into_any(),
                    Err(err) => view! {
                        <div style="color: #cf222e; font-size: 13px; font-weight: 500; display: flex; align-items: center; gap: 6px;">
                            <span>"✗"</span> {err}
                        </div>
                    }.into_any(),
                })}
            </div>

            // Section 2: Endpoints & URLs (GitHub Enterprise)
            <div style="background: #ffffff; border: 1px solid #d0d7de; border-radius: 6px; padding: 20px; margin-bottom: 24px;">
                <h2 style="font-size: 16px; font-weight: 600; margin-top: 0; margin-bottom: 12px;">"GitHub Endpoints (Enterprise Support)"</h2>

                <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 16px; margin-bottom: 8px;">
                    <div>
                        <label style="display: block; font-weight: 600; font-size: 13px; margin-bottom: 6px;">
                            "GitHub API URL"
                        </label>
                        <input
                            type="text"
                            prop:value=move || api_url.get()
                            on:input=move |ev| api_url.set(event_target_value(&ev))
                            style="width: 100%; box-sizing: border-box; padding: 7px 12px; border: 1px solid #d0d7de; border-radius: 6px; font-size: 13px;"
                        />
                        <span style="font-size: 11px; color: #57606a;">"Default: https://api.github.com"</span>
                    </div>

                    <div>
                        <label style="display: block; font-weight: 600; font-size: 13px; margin-bottom: 6px;">
                            "GitHub Web Root URL"
                        </label>
                        <input
                            type="text"
                            prop:value=move || root_url.get()
                            on:input=move |ev| root_url.set(event_target_value(&ev))
                            style="width: 100%; box-sizing: border-box; padding: 7px 12px; border: 1px solid #d0d7de; border-radius: 6px; font-size: 13px;"
                        />
                        <span style="font-size: 11px; color: #57606a;">"Default: https://github.com"</span>
                    </div>
                </div>
            </div>

            // Section 3: Notification Filters
            <div style="background: #ffffff; border: 1px solid #d0d7de; border-radius: 6px; padding: 20px; margin-bottom: 24px;">
                <h2 style="font-size: 16px; font-weight: 600; margin-top: 0; margin-bottom: 12px;">"Notification Filters"</h2>

                <label style="display: flex; align-items: center; gap: 8px; margin-bottom: 16px; cursor: pointer; font-size: 13px;">
                    <input
                        type="checkbox"
                        prop:checked=move || only_participating.get()
                        on:change=move |ev| only_participating.set(event_target_checked(&ev))
                    />
                    <span>"Only show notifications for issues and pull requests I am participating in"</span>
                </label>

                <div style="margin-bottom: 12px;">
                    <label style="display: block; font-weight: 600; font-size: 13px; margin-bottom: 6px;">
                        "Repository Filter Mode"
                    </label>
                    <select
                        on:change=move |ev| {
                            let val = event_target_value(&ev);
                            let mode = match val.as_str() {
                                "whitelist" => RepoFilterMode::Whitelist,
                                "blacklist" => RepoFilterMode::Blacklist,
                                _ => RepoFilterMode::None,
                            };
                            repo_filter_mode.set(mode);
                        }
                        style="padding: 7px 12px; border: 1px solid #d0d7de; border-radius: 6px; font-size: 13px; background: white;"
                    >
                        <option value="none" selected=move || repo_filter_mode.get() == RepoFilterMode::None>"No repository filtering"</option>
                        <option value="whitelist" selected=move || repo_filter_mode.get() == RepoFilterMode::Whitelist>"Whitelist (Only show selected repos)"</option>
                        <option value="blacklist" selected=move || repo_filter_mode.get() == RepoFilterMode::Blacklist>"Blacklist (Hide selected repos)"</option>
                    </select>
                </div>

                {move || if repo_filter_mode.get() != RepoFilterMode::None {
                    view! {
                        <div>
                            <label style="display: block; font-size: 12px; color: #57606a; margin-bottom: 6px;">
                                "Enter repository names, one per line (e.g. owner/repo):"
                            </label>
                            <textarea
                                rows="4"
                                prop:value=move || repo_filter_text.get()
                                on:input=move |ev| repo_filter_text.set(event_target_value(&ev))
                                style="width: 100%; box-sizing: border-box; padding: 8px; border: 1px solid #d0d7de; border-radius: 6px; font-family: monospace; font-size: 12px;"
                            ></textarea>
                        </div>
                    }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }}
            </div>

            // Section 4: Behavior Preferences
            <div style="background: #ffffff; border: 1px solid #d0d7de; border-radius: 6px; padding: 20px; margin-bottom: 24px;">
                <h2 style="font-size: 16px; font-weight: 600; margin-top: 0; margin-bottom: 12px;">"Preferences"</h2>

                <div style="margin-bottom: 16px;">
                    <label style="display: block; font-weight: 600; font-size: 13px; margin-bottom: 6px;">
                        "Polling Interval"
                    </label>
                    <select
                        on:change=move |ev| {
                            if let Ok(v) = event_target_value(&ev).parse::<u32>() {
                                poll_interval_mins.set(v);
                            }
                        }
                        style="padding: 7px 12px; border: 1px solid #d0d7de; border-radius: 6px; font-size: 13px; background: white;"
                    >
                        <option value="1" selected=move || poll_interval_mins.get() == 1>"Every 1 minute (Recommended)"</option>
                        <option value="2" selected=move || poll_interval_mins.get() == 2>"Every 2 minutes"</option>
                        <option value="5" selected=move || poll_interval_mins.get() == 5>"Every 5 minutes"</option>
                        <option value="10" selected=move || poll_interval_mins.get() == 10>"Every 10 minutes"</option>
                        <option value="15" selected=move || poll_interval_mins.get() == 15>"Every 15 minutes"</option>
                    </select>
                </div>

                <div style="display: flex; flex-direction: column; gap: 10px;">
                    <label style="display: flex; align-items: center; gap: 8px; cursor: pointer; font-size: 13px;">
                        <input
                            type="checkbox"
                            prop:checked=move || show_desktop_notifications.get()
                            on:change=move |ev| show_desktop_notifications.set(event_target_checked(&ev))
                        />
                        <span>"Show desktop notifications for newly received notifications"</span>
                    </label>

                    <label style="display: flex; align-items: center; gap: 8px; cursor: pointer; font-size: 13px;">
                        <input
                            type="checkbox"
                            prop:checked=move || reuse_existing_tab.get()
                            on:change=move |ev| reuse_existing_tab.set(event_target_checked(&ev))
                        />
                        <span>"Reuse existing GitHub notification tab if already open"</span>
                    </label>
                </div>
            </div>

            // Save Actions
            <div style="display: flex; justify-content: flex-end; gap: 12px;">
                <button
                    on:click=on_save
                    style="background: #1f883d; color: white; border: none; padding: 10px 24px; border-radius: 6px; font-weight: 600; font-size: 14px; cursor: pointer;"
                >
                    "Save Settings"
                </button>
            </div>
        </div>
    }
}
