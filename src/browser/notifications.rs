#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["chrome", "notifications"], js_name = create)]
    fn chrome_notifications_create(id: &str, options: &JsValue);

    #[wasm_bindgen(js_namespace = ["chrome", "notifications"], js_name = clear)]
    fn chrome_notifications_clear(id: &str);

    #[wasm_bindgen(js_namespace = ["chrome", "notifications", "onClicked"], js_name = addListener)]
    pub fn chrome_notifications_on_clicked_add_listener(callback: &Closure<dyn FnMut(String)>);
}

/// Create a desktop notification
pub fn show_desktop_notification(id: &str, title: &str, message: &str, icon_url: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        let obj = js_sys::Object::new();
        let _ = js_sys::Reflect::set(
            &obj,
            &JsValue::from_str("type"),
            &JsValue::from_str("basic"),
        );
        let _ = js_sys::Reflect::set(
            &obj,
            &JsValue::from_str("iconUrl"),
            &JsValue::from_str(icon_url),
        );
        let _ = js_sys::Reflect::set(&obj, &JsValue::from_str("title"), &JsValue::from_str(title));
        let _ = js_sys::Reflect::set(
            &obj,
            &JsValue::from_str("message"),
            &JsValue::from_str(message),
        );
        chrome_notifications_create(id, &obj.into());
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (id, title, message, icon_url);
    }
}

/// Listen to notification clicks
#[cfg(target_arch = "wasm32")]
pub fn on_notification_clicked<F>(mut callback: F)
where
    F: FnMut(String) + 'static,
{
    let closure = Closure::wrap(Box::new(move |notification_id: String| {
        callback(notification_id);
    }) as Box<dyn FnMut(String)>);

    chrome_notifications_on_clicked_add_listener(&closure);
    closure.forget();
}

#[cfg(not(target_arch = "wasm32"))]
pub fn on_notification_clicked<F>(_callback: F)
where
    F: FnMut(String) + 'static,
{
}
