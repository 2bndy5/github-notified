//! Raw `#[wasm_bindgen]` extern declarations for Chrome Extension APIs.
//! Replaces the equivalent bindings previously provided by oxichrome-core.

use wasm_bindgen::prelude::*;

// chrome.runtime

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["chrome", "runtime"], js_name = getURL)]
    pub fn chrome_runtime_get_url(path: &str) -> String;

    #[wasm_bindgen(js_namespace = ["chrome", "runtime", "onInstalled"], js_name = addListener)]
    pub fn chrome_runtime_on_installed_add_listener(callback: &Closure<dyn FnMut(JsValue)>);
}

// chrome.storage.local

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["chrome", "storage", "local"], js_name = get)]
    pub fn chrome_storage_local_get(keys: &JsValue) -> js_sys::Promise;

    #[wasm_bindgen(js_namespace = ["chrome", "storage", "local"], js_name = set)]
    pub fn chrome_storage_local_set(items: &JsValue) -> js_sys::Promise;

    #[wasm_bindgen(js_namespace = ["chrome", "storage", "local"], js_name = remove)]
    pub fn chrome_storage_local_remove(keys: &JsValue) -> js_sys::Promise;
}

// chrome.tabs

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["chrome", "tabs"], js_name = query)]
    pub fn chrome_tabs_query(query_info: &JsValue) -> js_sys::Promise;

    #[wasm_bindgen(js_namespace = ["chrome", "tabs"], js_name = create)]
    pub fn chrome_tabs_create(create_properties: &JsValue) -> js_sys::Promise;

    #[wasm_bindgen(js_namespace = ["chrome", "tabs"], js_name = update)]
    pub fn chrome_tabs_update(tab_id: i32, update_properties: &JsValue) -> js_sys::Promise;
}
