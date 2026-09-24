#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["chrome", "action"], js_name = setBadgeText)]
    fn chrome_action_set_badge_text(details: &JsValue);

    #[wasm_bindgen(js_namespace = ["chrome", "action"], js_name = setBadgeBackgroundColor)]
    fn chrome_action_set_badge_background_color(details: &JsValue);

    #[wasm_bindgen(js_namespace = ["chrome", "action"], js_name = setTitle)]
    fn chrome_action_set_title(details: &JsValue);
}

pub fn set_badge_text(text: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        let obj = js_sys::Object::new();
        let _ = js_sys::Reflect::set(&obj, &JsValue::from_str("text"), &JsValue::from_str(text));
        chrome_action_set_badge_text(&obj.into());
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = text;
}

pub fn set_badge_color(color_hex: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        let obj = js_sys::Object::new();
        let _ = js_sys::Reflect::set(
            &obj,
            &JsValue::from_str("color"),
            &JsValue::from_str(color_hex),
        );
        chrome_action_set_badge_background_color(&obj.into());
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = color_hex;
}

pub fn set_title(title: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        let obj = js_sys::Object::new();
        let _ = js_sys::Reflect::set(&obj, &JsValue::from_str("title"), &JsValue::from_str(title));
        chrome_action_set_title(&obj.into());
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = title;
}

/// Reset or clear the badge text
pub fn clear_badge() {
    set_badge_text("");
}

/// Register a listener for toolbar icon clicks (fires only when there is no popup)
#[cfg(target_arch = "wasm32")]
pub fn on_clicked<F>(mut callback: F)
where
    F: FnMut() + 'static,
{
    let closure = Closure::wrap(Box::new(move |_tab: JsValue| {
        callback();
    }) as Box<dyn FnMut(JsValue)>);

    crate::js_bridge::chrome_action_on_clicked_add_listener(&closure);
    closure.forget();
}

#[cfg(not(target_arch = "wasm32"))]
pub fn on_clicked<F>(_callback: F)
where
    F: FnMut() + 'static,
{
}
