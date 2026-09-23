#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["chrome", "alarms"], js_name = create)]
    fn chrome_alarms_create(name: &str, alarm_info: &JsValue);

    #[wasm_bindgen(js_namespace = ["chrome", "alarms"], js_name = clear)]
    fn chrome_alarms_clear(name: &str);

    #[wasm_bindgen(js_namespace = ["chrome", "alarms", "onAlarm"], js_name = addListener)]
    pub fn chrome_alarms_on_alarm_add_listener(callback: &Closure<dyn FnMut(JsValue)>);
}

pub const ALARM_POLL_NOTIFICATIONS: &str = "poll-github-notifications";

/// Set up recurring background alarm
pub fn schedule_poll(period_in_minutes: f64) {
    #[cfg(target_arch = "wasm32")]
    {
        let obj = js_sys::Object::new();
        let _ = js_sys::Reflect::set(
            &obj,
            &JsValue::from_str("delayInMinutes"),
            &JsValue::from_f64(0.1), // Check shortly after launch
        );
        let _ = js_sys::Reflect::set(
            &obj,
            &JsValue::from_str("periodInMinutes"),
            &JsValue::from_f64(period_in_minutes.max(1.0)),
        );
        chrome_alarms_create(ALARM_POLL_NOTIFICATIONS, &obj.into());
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = period_in_minutes;
}

/// Clear recurring alarm
pub fn clear_poll_alarm() {
    #[cfg(target_arch = "wasm32")]
    chrome_alarms_clear(ALARM_POLL_NOTIFICATIONS);
}

/// Register a listener callback for chrome.alarms.onAlarm
#[cfg(target_arch = "wasm32")]
pub fn on_alarm<F>(mut callback: F)
where
    F: FnMut(String) + 'static,
{
    let closure = Closure::wrap(Box::new(move |alarm: JsValue| {
        if let Ok(name_val) = js_sys::Reflect::get(&alarm, &JsValue::from_str("name")) {
            if let Some(name) = name_val.as_string() {
                callback(name);
            }
        }
    }) as Box<dyn FnMut(JsValue)>);

    chrome_alarms_on_alarm_add_listener(&closure);
    closure.forget();
}

#[cfg(not(target_arch = "wasm32"))]
pub fn on_alarm<F>(_callback: F)
where
    F: FnMut(String) + 'static,
{
}
