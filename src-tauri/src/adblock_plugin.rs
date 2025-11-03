use adblock::{lists::ParseOptions, request::Request, Engine};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{
    plugin::{Builder, TauriPlugin},
    AppHandle, Manager,
};

const FILTER_LISTS: &[(&str, &str)] = &[
    ("uBlock filters", "https://raw.githubusercontent.com/uBlockOrigin/uAssets/master/filters/filters.txt"),
    ("uBlock filters - Privacy", "https://raw.githubusercontent.com/uBlockOrigin/uAssets/master/filters/privacy.txt"),
    ("uBlock filters - Badware risks", "https://raw.githubusercontent.com/uBlockOrigin/uAssets/master/filters/badware.txt"),
    ("uBlock filters - Resource abuse", "https://raw.githubusercontent.com/uBlockOrigin/uAssets/master/filters/resource-abuse.txt"),
    ("uBlock filters - Unbreak", "https://raw.githubusercontent.com/uBlockOrigin/uAssets/master/filters/unbreak.txt"),
    ("EasyList", "https://easylist.to/easylist/easylist.txt"),
    ("EasyPrivacy", "https://easylist.to/easylist/easyprivacy.txt"),
    ("AdGuard URL Tracking Protection", "https://filters.adtidy.org/extension/chromium/filters/17.txt"),
    ("Peter Lowe's Ad and tracking server list", "https://pgl.yoyo.org/adservers/serverlist.php?hostformat=hosts&showintro=0&mimetype=plaintext"),
];

const WHITELIST_DOMAINS: &[&str] = &[];

// ✅ Eliminado el campo cosmetic_filters innecesario
#[derive(Clone)]
pub struct AdBlockState {
    engine: Arc<Mutex<Option<Engine>>>,
}

impl AdBlockState {
    fn new() -> Self {
        Self {
            engine: Arc::new(Mutex::new(None)),
        }
    }

    fn set_engine(&self, engine: Engine) {
        let mut lock = self.engine.lock().unwrap();
        *lock = Some(engine);
    }

    fn check_url(&self, url: &str, source_url: &str, request_type: &str) -> bool {
        if Self::is_whitelisted(url)
            || url.starts_with("data:")
            || url.starts_with("blob:")
            || url.starts_with("tauri://")
        {
            return false;
        }

        let lock = self.engine.lock().unwrap();
        if let Some(engine) = lock.as_ref() {
            let normalized_type = match request_type {
                "fetch" | "xhr" => "xmlhttprequest",
                "link" => "stylesheet",
                "img" => "image",
                "video" | "audio" => "media",
                "script" => "script",
                "document" => "document",
                "sub_frame" | "subdocument" => "subdocument",
                "main_frame" => "document",
                _ => "other",
            };

            if let Ok(request) = Request::new(url, source_url, normalized_type) {
                return engine.check_network_request(&request).matched;
            }
        }
        false
    }

    fn is_whitelisted(url: &str) -> bool {
        WHITELIST_DOMAINS.iter().any(|domain| url.contains(domain))
    }

    // ✅ Ahora acepta URL completa en lugar de hostname
    fn get_cosmetic_resources(&self, url: &str) -> serde_json::Value {
        use adblock::cosmetic_filter_cache::UrlSpecificResources;

        let lock = self.engine.lock().unwrap();

        let resources = if let Some(engine) = lock.as_ref() {
            engine.url_cosmetic_resources(url)
        } else {
            UrlSpecificResources::empty()
        };

        serde_json::json!({
            "hide_selectors": resources.hide_selectors.into_iter().collect::<Vec<_>>(),
            "exceptions": resources.exceptions.into_iter().collect::<Vec<_>>(),
            "injected_script": resources.injected_script,
            "procedural_actions": resources.procedural_actions.into_iter().collect::<Vec<_>>(),
            "generichide": resources.generichide,
        })
    }

    fn is_ready(&self) -> bool {
        self.engine.lock().unwrap().is_some()
    }
}

pub fn init() -> TauriPlugin<tauri::Wry> {
    Builder::new("adblock")
        .setup(|app, _api| {
            println!("🛡️ AdBlock: Initializing plugin...");

            let state = AdBlockState::new();
            app.manage(state.clone());

            let app_handle = app.clone();

            tauri::async_runtime::spawn(async move {
                if let Err(e) = setup_filters(&app_handle).await {
                    eprintln!("❌ AdBlock: Error setting up filters: {}", e);
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
    println!("🛡️ AdBlock: Checking filter cache...");

    let cache_dir = app.path().app_data_dir()?.join("adblock_cache");
    fs::create_dir_all(&cache_dir)?;

    let engine_cache_path = cache_dir.join("engine.dat");
    let cache_duration = 24 * 60 * 60;

    if let Ok(cached_engine) = load_engine_from_cache(&engine_cache_path, cache_duration) {
        println!("🚀 AdBlock: Loaded engine from cache");
        let state: tauri::State<AdBlockState> = app.state();
        state.set_engine(cached_engine);
        println!("🛡️ AdBlock ready (from cache)");
        return Ok(());
    }

    let mut all_content = String::new();

    for (name, url) in FILTER_LISTS {
        let filename = format!("{}.txt", name.replace(&[' ', '-', '\''][..], "_"));
        let cache_path = cache_dir.join(&filename);

        print!("   {} ... ", name);

        let use_cache = if cache_path.exists() {
            if let Ok(metadata) = fs::metadata(&cache_path) {
                if let Ok(modified) = metadata.modified() {
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    let cache_time = modified.duration_since(UNIX_EPOCH).unwrap().as_secs();

                    (now - cache_time) < cache_duration
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        };

        let content = if use_cache {
            match fs::read_to_string(&cache_path) {
                Ok(content) => {
                    println!("📁 cache");
                    content
                }
                Err(_) => download_and_cache(url, &cache_path).await?,
            }
        } else {
            download_and_cache(url, &cache_path).await?
        };

        all_content.push_str(&content);
        all_content.push('\n');
    }

    let engine = Engine::from_rules(
        all_content
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty() && !line.starts_with('!')),
        ParseOptions::default(),
    );

    // ✅ Manejo correcto de errores en serialización
    if let Err(e) = save_engine_to_cache(&engine, &engine_cache_path) {
        eprintln!("⚠️ Could not save engine cache: {}", e);
    } else {
        println!("💾 Engine saved to cache");
    }

    let state: tauri::State<AdBlockState> = app.state();
    state.set_engine(engine);

    println!("🛡️ AdBlock ready");
    Ok(())
}

fn save_engine_to_cache(
    engine: &Engine,
    cache_path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let serialized = engine.serialize();
    fs::write(cache_path, serialized)?;
    Ok(())
}

fn load_engine_from_cache(
    cache_path: &PathBuf,
    cache_duration: u64,
) -> Result<Engine, Box<dyn std::error::Error>> {
    if !cache_path.exists() {
        return Err("Cache file does not exist".into());
    }

    if let Ok(metadata) = fs::metadata(cache_path) {
        if let Ok(modified) = metadata.modified() {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let cache_time = modified.duration_since(UNIX_EPOCH).unwrap().as_secs();

            if (now - cache_time) >= cache_duration {
                return Err("Cache expired".into());
            }
        } else {
            return Err("Could not read cache modification time".into());
        }
    } else {
        return Err("Could not read cache metadata".into());
    }

    let serialized_data = fs::read(cache_path)?;

    let mut engine = Engine::default();
    engine
        .deserialize(&serialized_data)
        .map_err(|e| format!("Deserialization error: {:?}", e))?;

    Ok(engine)
}

async fn download_and_cache(
    url: &str,
    cache_path: &PathBuf,
) -> Result<String, Box<dyn std::error::Error>> {
    match ureq::get(url)
        .timeout(std::time::Duration::from_secs(30))
        .call()
    {
        Ok(response) => {
            if let Ok(content) = response.into_string() {
                if let Err(e) = fs::write(cache_path, &content) {
                    eprintln!("⚠️ Could not save cache: {}", e);
                }
                println!("✓ downloaded");
                Ok(content)
            } else {
                Err("Error reading response".into())
            }
        }
        Err(e) => {
            println!("✗ {}", e);
            Err(e.into())
        }
    }
}

#[tauri::command]
pub async fn check_batch_urls(
    urls: Vec<(String, String, String)>,
    state: tauri::State<'_, AdBlockState>,
) -> Result<Vec<bool>, String> {
    Ok(urls
        .iter()
        .map(|(url, source, req_type)| state.check_url(url, source, req_type))
        .collect())
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

// ✅ Ahora acepta URL completa
#[tauri::command]
pub async fn get_cosmetic_resources(
    url: String,
    state: tauri::State<'_, AdBlockState>,
) -> Result<serde_json::Value, String> {
    Ok(state.get_cosmetic_resources(&url))
}

// ✅ Ahora recibe exceptions desde el frontend
#[tauri::command]
pub async fn get_hidden_class_id_selectors(
    classes: Vec<String>,
    ids: Vec<String>,
    exceptions: HashSet<String>,
    state: tauri::State<'_, AdBlockState>,
) -> Result<Vec<String>, String> {
    let lock = state.engine.lock().unwrap();

    if let Some(engine) = lock.as_ref() {
        Ok(engine.hidden_class_id_selectors(classes, ids, &exceptions))
    } else {
        Ok(Vec::new())
    }
}
