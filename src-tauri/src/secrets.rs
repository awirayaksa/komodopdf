use keyring::Entry;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

static FALLBACK_KEYS: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
static FILE_PATH: OnceLock<Mutex<Option<PathBuf>>> = OnceLock::new();

fn fallback_keys() -> &'static Mutex<HashMap<String, String>> {
    FALLBACK_KEYS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Must be called once during app startup (e.g. in `setup`) so the
/// file-based fallback knows where to read/write.
pub fn init_file_path(path: PathBuf) {
    let _ = FILE_PATH.set(Mutex::new(Some(path)));
}

fn file_path() -> Option<PathBuf> {
    FILE_PATH.get()?.lock().unwrap().clone()
}

fn read_file_fallback(username: &str) -> Option<String> {
    let path = file_path()?;
    if !path.exists() {
        return None;
    }
    let content = std::fs::read_to_string(&path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    json.get(username)?.as_str().map(|s| s.to_string())
}

fn write_file_fallback(username: &str, key: &str) {
    let path = match file_path() {
        Some(p) => p,
        None => return,
    };
    let mut json = if path.exists() {
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(|| serde_json::Value::Object(Default::default()))
    } else {
        serde_json::Value::Object(Default::default())
    };
    if let serde_json::Value::Object(ref mut map) = json {
        map.insert(username.to_string(), serde_json::Value::String(key.to_string()));
    }
    let _ = std::fs::create_dir_all(path.parent().unwrap_or(path.as_path()));
    let _ = std::fs::write(&path, json.to_string());
}

fn delete_file_fallback(username: &str) {
    let path = match file_path() {
        Some(p) => p,
        None => return,
    };
    if !path.exists() {
        return;
    }
    let mut json = std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| serde_json::Value::Object(Default::default()));
    if let serde_json::Value::Object(ref mut map) = json {
        map.remove(username);
    }
    let _ = std::fs::write(&path, json.to_string());
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum KeySlot {
    Chat,
    Embed,
}

impl KeySlot {
    fn service(&self) -> &'static str {
        "komodo-pdf"
    }
    fn username(&self) -> &'static str {
        match self {
            KeySlot::Chat => "api-key-chat",
            KeySlot::Embed => "api-key-embed",
        }
    }
}

pub fn set_api_key(slot: KeySlot, key: String) -> Result<(), String> {
    // Try OS keyring first (best effort)
    if let Ok(entry) = Entry::new(slot.service(), slot.username()) {
        let _ = entry.set_password(&key);
    }
    // Always cache in memory so the session works even if keyring is flaky
    let mut map = fallback_keys().lock().unwrap();
    map.insert(slot.username().to_string(), key.clone());
    // Also persist to file so the key survives app restarts when the
    // OS keyring is unavailable or flaky.
    write_file_fallback(slot.username(), &key);
    Ok(())
}

pub fn has_api_key(slot: KeySlot) -> bool {
    // Try keyring
    if let Ok(entry) = Entry::new(slot.service(), slot.username()) {
        if entry.get_password().is_ok() {
            return true;
        }
    }
    // File fallback
    if read_file_fallback(slot.username()).is_some() {
        return true;
    }
    // In-memory fallback
    let map = fallback_keys().lock().unwrap();
    map.contains_key(slot.username())
}

pub fn delete_api_key(slot: KeySlot) -> Result<(), String> {
    if let Ok(entry) = Entry::new(slot.service(), slot.username()) {
        let _ = entry.delete_credential();
    }
    let mut map = fallback_keys().lock().unwrap();
    map.remove(slot.username());
    delete_file_fallback(slot.username());
    Ok(())
}

pub fn get_api_key(slot: KeySlot) -> Result<Option<String>, String> {
    // Try OS keyring first
    if let Ok(entry) = Entry::new(slot.service(), slot.username()) {
        match entry.get_password() {
            Ok(p) => return Ok(Some(p)),
            Err(keyring::Error::NoEntry) => {}
            Err(e) => return Err(e.to_string()),
        }
    }
    // Fall back to file store (survives restarts)
    if let Some(key) = read_file_fallback(slot.username()) {
        return Ok(Some(key));
    }
    // Fall back to in-memory cache
    let map = fallback_keys().lock().unwrap();
    Ok(map.get(slot.username()).cloned())
}
