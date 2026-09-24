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
#[derive(Deserialize)]
struct Tab {
    id: Option<i32>,
    #[serde(default)]
    url: Option<String>,
}

/// Open a URL, optionally reusing an existing tab matching the URL pattern
pub async fn open_or_focus_tab(target_url: &str, reuse_existing: bool) {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsValue;
        use wasm_bindgen_futures::JsFuture;

        if reuse_existing {
            let pattern = if target_url.contains("github.com/notifications") {
                "*://github.com/notifications*"
            } else {
                target_url
            };

            let query = QueryInfo { url: pattern };
            if let Ok(js_query) = serde_wasm_bindgen::to_value(&query) {
                let promise = crate::js_bridge::chrome_tabs_query(&js_query);
                if let Ok(result) = JsFuture::from(promise).await {
                    if let Ok(tabs) = serde_wasm_bindgen::from_value::<Vec<Tab>>(result) {
                        if let Some(tab) = tabs.first() {
                            if let Some(tab_id) = tab.id {
                                let obj = js_sys::Object::new();
                                let _ = js_sys::Reflect::set(
                                    &obj,
                                    &JsValue::from_str("active"),
                                    &JsValue::from_bool(true),
                                );
                                let _ = JsFuture::from(crate::js_bridge::chrome_tabs_update(
                                    tab_id,
                                    &obj.into(),
                                ))
                                .await;
                                return;
                            }
                        }
                    }
                }
            }
        }

        let props = CreateTabProps {
            url: target_url,
            active: true,
        };
        if let Ok(js_props) = serde_wasm_bindgen::to_value(&props) {
            let promise = crate::js_bridge::chrome_tabs_create(&js_props);
            let _ = JsFuture::from(promise).await;
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (target_url, reuse_existing);
    }
}
