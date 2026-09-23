#[cfg(target_arch = "wasm32")]
use serde::{Deserialize, Serialize};

#[cfg(target_arch = "wasm32")]
#[derive(Serialize)]
struct QueryInfo<'a> {
    url: &'a str,
}

#[cfg(target_arch = "wasm32")]
#[derive(Serialize)]
struct CreateTabProps<'a> {
    url: &'a str,
    active: bool,
}

#[cfg(target_arch = "wasm32")]
#[allow(dead_code)]
#[derive(Serialize)]
struct UpdateTabProps {
    active: bool,
}

#[cfg(target_arch = "wasm32")]
#[allow(dead_code)]
#[derive(Deserialize)]
struct Tab {
    id: Option<i32>,
    #[serde(default)]
    url: Option<String>,
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["chrome", "tabs"], js_name = update)]
    fn chrome_tabs_update(tab_id: i32, update_properties: &JsValue) -> js_sys::Promise;
}

/// Open a URL, optionally reusing an existing tab matching the URL pattern
pub async fn open_or_focus_tab(target_url: &str, reuse_existing: bool) {
    #[cfg(target_arch = "wasm32")]
    {
        if reuse_existing {
            // Strip queries/hashes for pattern matching
            let pattern = if target_url.contains("github.com/notifications") {
                "*://github.com/notifications*"
            } else {
                target_url
            };

            let query = QueryInfo { url: pattern };
            if let Ok(tabs) = oxichrome::tabs::query::<_, Tab>(&query).await {
                if let Some(tab) = tabs.first() {
                    if let Some(tab_id) = tab.id {
                        let obj = js_sys::Object::new();
                        let _ = js_sys::Reflect::set(
                            &obj,
                            &JsValue::from_str("active"),
                            &JsValue::from_bool(true),
                        );
                        let _ = chrome_tabs_update(tab_id, &obj.into());
                        return;
                    }
                }
            }
        }

        let props = CreateTabProps {
            url: target_url,
            active: true,
        };
        let _ = oxichrome::tabs::create::<_, Tab>(&props).await;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (target_url, reuse_existing);
    }
}
