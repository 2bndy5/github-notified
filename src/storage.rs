//! Chrome `storage.local` helpers — replaces `oxichrome::storage`.

use serde::de::DeserializeOwned;
use serde::Serialize;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;

use crate::js_bridge;

#[derive(Debug)]
pub enum StorageError {
    Js(JsValue),
    Serde(serde_wasm_bindgen::Error),
}

impl From<JsValue> for StorageError {
    fn from(v: JsValue) -> Self {
        StorageError::Js(v)
    }
}

impl From<serde_wasm_bindgen::Error> for StorageError {
    fn from(e: serde_wasm_bindgen::Error) -> Self {
        StorageError::Serde(e)
    }
}

pub type Result<T> = std::result::Result<T, StorageError>;

pub async fn get<T: DeserializeOwned>(key: &str) -> Result<Option<T>> {
    let keys = JsValue::from_str(key);
    let promise = js_bridge::chrome_storage_local_get(&keys);
    let result = JsFuture::from(promise).await?;

    let val = js_sys::Reflect::get(&result, &JsValue::from_str(key))
        .map_err(StorageError::Js)?;

    if val.is_undefined() || val.is_null() {
        return Ok(None);
    }

    let deserialized: T = serde_wasm_bindgen::from_value(val)?;
    Ok(Some(deserialized))
}

pub async fn set<T: Serialize>(key: &str, value: &T) -> Result<()> {
    let obj = js_sys::Object::new();
    let js_val = serde_wasm_bindgen::to_value(value)?;
    js_sys::Reflect::set(&obj, &JsValue::from_str(key), &js_val)
        .map_err(StorageError::Js)?;

    let promise = js_bridge::chrome_storage_local_set(&obj.into());
    JsFuture::from(promise).await?;
    Ok(())
}

pub async fn remove(key: &str) -> Result<()> {
    let keys = JsValue::from_str(key);
    let promise = js_bridge::chrome_storage_local_remove(&keys);
    JsFuture::from(promise).await?;
    Ok(())
}
