// src-tauri/src/adblock_plugin.rs (corregido)
use adblock::{lists::ParseOptions, request::Request, Engine};
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{plugin::{Builder, TauriPlugin}, AppHandle, Manager};

const FILTER_LISTS: &[(&str, &str)] = &[
    ("uBlock filters", "https://raw.githubusercontent.com/uBlockOrigin/uAssets/master/filters/filters.txt"),
    ("uBlock filters - Privacy", "https://raw.githubusercontent.com/uBlockOrigin/uAssets/master/filters/privacy.txt"),
    ("uBlock filters - Badware risks", "https://raw.githubusercontent.com/uBlockOrigin/uAssets/master/filters/badware.txt"),
    ("uBlock filters - Resource abuse", "https://raw.githubusercontent.com/uBlockOrigin/uAssets/master/filters/resource-abuse.txt"),
    ("uBlock filters - Unbreak", "https://raw.githubusercontent.com/uBlockOrigin/uAssets/master/filters/unbreak.txt"),
    ("EasyList", "https://easylist.to/easylist/easylist.txt"),
    ("EasyPrivacy", "https://easylist.to/easylist/easyprivacy.txt"),
    ("AdGuard URL Tracking Protection", "https://filters.adtidy.org/extension/chromium/filters/17.txt"),
    ("Peter Lowes Ad and tracking server list", "https://pgl.yoyo.org/adservers/serverlist.php?hostformat=hosts&showintro=0&mimetype=plaintext"),
];

const WHITELIST_DOMAINS: &[&str] = &[];
const CACHE_DURATION_SECS: u64 = 24 * 60 * 60;
const MAX_DOWNLOAD_SIZE: usize = 50 * 1024 * 1024;
const ENGINE_CACHE_FILE: &str = "engine.dat";

#[derive(Clone)]
pub struct AdBlockState {
    engine: Arc<RwLock<Option<Engine>>>,
}

impl AdBlockState {
    fn new() -> Self {
        Self {
            engine: Arc::new(RwLock::new(None)),
        }
    }

    fn set_engine(&self, engine: Engine) {
        *self.engine.write().unwrap() = Some(engine);
    }

    fn check_url(&self, url: &str, source_url: &str, request_type: &str) -> bool {
        if Self::is_whitelisted(url) || matches!(url.get(..6), Some("data:" | "blob:" | "tauri:")) {
            return false;
        }

        let engine_guard = self.engine.read().unwrap();
        let Some(engine) = engine_guard.as_ref() else {
            return false;
        };

        let normalized_type = match request_type {
            "fetch" | "xhr" => "xmlhttprequest",
            "link" => "stylesheet",
            "img" => "image",
            "video" | "audio" => "media",
            _ => request_type,
        };

        Request::new(url, source_url, normalized_type)
            .map(|req| engine.check_network_request(&req).matched)
            .unwrap_or(false)
    }

    fn is_whitelisted(url: &str) -> bool {
        WHITELIST_DOMAINS.iter().any(|domain| url.contains(domain))
    }

    fn get_cosmetic_resources(&self, url: &str) -> serde_json::Value {
        use adblock::cosmetic_filter_cache::UrlSpecificResources;
        
        let engine_guard = self.engine.read().unwrap();
        let resources = engine_guard
            .as_ref()
            .map(|e| e.url_cosmetic_resources(url))
            .unwrap_or_else(UrlSpecificResources::empty);

        serde_json::json!({
            "hide_selectors": resources.hide_selectors,
            "exceptions": resources.exceptions,
            "injected_script": resources.injected_script,
            "procedural_actions": resources.procedural_actions,
            "generichide": resources.generichide,
        })
    }

    fn is_ready(&self) -> bool {
        self.engine.read().unwrap().is_some()
    }
}

pub fn init() -> TauriPlugin<tauri::Wry> {
    Builder::new("adblock")
        .setup(|app, _| {
            let state = AdBlockState::new();
            app.manage(state.clone());

            let app_handle = app.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = setup_filters(&app_handle).await {
                    eprintln!("AdBlock error: {}", e);
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            is_url_blocked,
            is_adblock_ready,
            get_cosmetic_resources,
            check_batch_urls,
            get_hidden_class_id_selectors,
        ])
        .build()
}

async fn setup_filters(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let cache_dir = app.path().app_data_dir()?.join("adblock_cache");
    fs::create_dir_all(&cache_dir)?;
    
    let engine_cache_path = cache_dir.join(ENGINE_CACHE_FILE);

    if let Ok(engine) = load_engine_from_cache(&engine_cache_path, CACHE_DURATION_SECS) {
        let state: tauri::State<AdBlockState> = app.state();
        state.set_engine(engine);
        return Ok(());
    }

    let filters = fetch_all_filters(&cache_dir).await?;
    let engine = Engine::from_rules(
        filters.lines().filter(|l| !l.is_empty() && !l.starts_with('!')),
        ParseOptions::default(),
    );

    if let Err(e) = save_engine_to_cache(&engine, &engine_cache_path) {
        eprintln!("Cache save error: {}", e);
    }

    let state: tauri::State<AdBlockState> = app.state();
    state.set_engine(engine);

    Ok(())
}

async fn fetch_all_filters(cache_dir: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let mut all_content = String::new();
    
    for (name, url) in FILTER_LISTS {
        let filename = sanitize_filename(name);
        let cache_path = cache_dir.join(&filename);
        
        let content = if is_cache_valid(&cache_path, CACHE_DURATION_SECS) {
            fs::read_to_string(&cache_path).unwrap_or_default()
        } else {
            match download_list(url, &cache_path).await {
                Ok(text) => text,
                Err(e) => {
                    eprintln!("Failed {}: {}", name, e);
                    continue;
                }
            }
        };
        
        all_content.push_str(&content);
        all_content.push('\n');
    }
    
    Ok(all_content)
}

async fn download_list(url: &str, cache_path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let response = ureq::get(url)
        .timeout(Duration::from_secs(30))
        .call()?;

    let content = response.into_string()?;
    
    if content.len() > MAX_DOWNLOAD_SIZE {
        return Err("Content too large".into());
    }

    fs::write(cache_path, &content)?;
    Ok(content)
}

fn is_cache_valid(path: &Path, max_age_secs: u64) -> bool {
    fs::metadata(path)
        .and_then(|m| m.modified())
        .map(|modified| {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            let cache_time = modified.duration_since(UNIX_EPOCH).unwrap().as_secs();
            now.saturating_sub(cache_time) < max_age_secs
        })
        .unwrap_or(false)
}

fn save_engine_to_cache(engine: &Engine, cache_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::write(cache_path, engine.serialize())?;
    Ok(())
}

fn load_engine_from_cache(path: &Path, max_age_secs: u64) -> Result<Engine, Box<dyn std::error::Error>> {
    if !is_cache_valid(path, max_age_secs) {
        return Err("Cache invalid".into());
    }

    let data = fs::read(path)?;
    let mut engine = Engine::default();
    engine.deserialize(&data).map_err(|e| format!("Deserialize: {:?}", e))?;
    Ok(engine)
}

fn sanitize_filename(name: &str) -> String {
    name.replace(&[' ', '-', '\''][..], "_").replace(|c: char| !c.is_alphanumeric() && c != '_', "") + ".txt"
}

#[tauri::command]
pub async fn check_batch_urls(
    urls: Vec<(String, String, String)>,
    state: tauri::State<'_, AdBlockState>,
) -> Result<Vec<bool>, String> {
    Ok(urls.into_iter().map(|(url, src, typ)| state.check_url(&url, &src, &typ)).collect())
}

#[tauri::command]
pub async fn is_url_blocked(
    url: String,
    source_url: String,
    request_type: String,
    state: tauri::State<'_, AdBlockState>,
) -> Result<bool, String> {
    Ok(state.check_url(&url, &source_url, &request_type))
}

#[tauri::command]
pub async fn is_adblock_ready(state: tauri::State<'_, AdBlockState>) -> Result<bool, String> {
    Ok(state.is_ready())
}

#[tauri::command]
pub async fn get_cosmetic_resources(
    url: String,
    state: tauri::State<'_, AdBlockState>,
) -> Result<serde_json::Value, String> {
    Ok(state.get_cosmetic_resources(&url))
}

#[tauri::command]
pub async fn get_hidden_class_id_selectors(
    classes: Vec<String>,
    ids: Vec<String>,
    exceptions: HashSet<String>,
    state: tauri::State<'_, AdBlockState>,
) -> Result<Vec<String>, String> {
    let engine_guard = state.engine.read().unwrap();
    Ok(engine_guard
        .as_ref()
        .map(|e| e.hidden_class_id_selectors(classes, ids, &exceptions))
        .unwrap_or_default())
}