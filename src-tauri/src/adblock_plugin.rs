use adblock::{lists::ParseOptions, request::Request, Engine, FilterSet};
use std::collections::HashMap;
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

#[derive(Clone)]
struct CacheEntry {
    blocked: bool,
    timestamp: std::time::Instant,
    access_count: u32,
}

struct AdBlockEngine {
    inner: Engine,
    cache: HashMap<String, CacheEntry>,
    cache_hits: u64,
    cache_misses: u64,
}

unsafe impl Send for AdBlockEngine {}
unsafe impl Sync for AdBlockEngine {}

#[derive(Clone)]
pub struct AdBlockState {
    engine: Arc<Mutex<Option<AdBlockEngine>>>,
    cosmetic_filters: Arc<Mutex<Vec<String>>>,
}

impl AdBlockState {
    fn new() -> Self {
        Self {
            engine: Arc::new(Mutex::new(None)),
            cosmetic_filters: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn set_engine(&self, engine: Engine, cosmetic: Vec<String>) {
        let mut lock = self.engine.lock().unwrap();
        *lock = Some(AdBlockEngine {
            inner: engine,
            cache: HashMap::new(),
            cache_hits: 0,
            cache_misses: 0,
        });

        let mut cosmetic_lock = self.cosmetic_filters.lock().unwrap();
        *cosmetic_lock = cosmetic;
    }

    fn check_url(&self, url: &str, source_url: &str, request_type: &str) -> bool {
        // Validaciones rápidas primero
        if Self::is_whitelisted(url) {
            return false;
        }

        if request_type == "preflight"
            || url.starts_with("data:")
            || url.starts_with("blob:")
            || url.starts_with("tauri://")
        {
            return false;
        }

        let mut lock = self.engine.lock().unwrap();
        if let Some(wrapper) = lock.as_mut() {
            let cache_key = format!("{}|{}", url, request_type);

            // Verificar caché con LRU mejorado
            if let Some(entry) = wrapper.cache.get_mut(&cache_key) {
                if entry.timestamp.elapsed().as_secs() < 120 {
                    // Aumentado a 2 minutos
                    entry.access_count += 1;
                    wrapper.cache_hits += 1;
                    return entry.blocked;
                } else {
                    // Entrada expirada, remover
                    wrapper.cache.remove(&cache_key);
                }
            }

            wrapper.cache_misses += 1;

            // Normalización de tipo de solicitud más eficiente
            let normalized_type = match request_type {
                "fetch" | "xhr" => "xmlhttprequest",
                "link" => "stylesheet",
                "img" => "image",
                "video" | "audio" => "media",
                _ => request_type, // Mantener tipos conocidos
            };

            if let Ok(request) = Request::new(url, source_url, normalized_type) {
                let result = wrapper.inner.check_network_request(&request);

                // Insertar en caché con contador de acceso
                wrapper.cache.insert(
                    cache_key,
                    CacheEntry {
                        blocked: result.matched,
                        timestamp: std::time::Instant::now(),
                        access_count: 1,
                    },
                );

                // Limpieza de caché más inteligente
                if wrapper.cache.len() > 5000 {
                    // Reducido para mejor rendimiento
                    let now = std::time::Instant::now();

                    // Primero remover entradas expiradas
                    wrapper
                        .cache
                        .retain(|_, v| now.duration_since(v.timestamp).as_secs() < 120);

                    // Si todavía hay muchas, remover las menos usadas
                    if wrapper.cache.len() > 3000 {
                        let mut entries: Vec<_> = wrapper
                            .cache
                            .iter()
                            .map(|(k, v)| (k.clone(), v.clone()))
                            .collect();
                        entries.sort_by_key(|(_, v)| (v.access_count, v.timestamp));

                        // Recolectar claves a eliminar
                        let to_remove = entries.len() - 3000;
                        let keys_to_remove: Vec<String> = entries
                            .iter()
                            .take(to_remove)
                            .map(|(k, _)| k.clone())
                            .collect();

                        // Eliminar claves recolectadas
                        for key in keys_to_remove {
                            wrapper.cache.remove(&key);
                        }
                    }
                }

                return result.matched;
            }
        }
        false
    }

    fn is_whitelisted(url: &str) -> bool {
        WHITELIST_DOMAINS.iter().any(|domain| url.contains(domain))
    }

    fn get_cosmetic_filters(&self, hostname: &str) -> Vec<String> {
        let lock = self.cosmetic_filters.lock().unwrap();

        lock.iter()
            .filter(|f| {
                if f.contains("##") {
                    let domain_part = f.split("##").next().unwrap_or("");
                    domain_part.is_empty() || hostname.contains(domain_part)
                } else {
                    false
                }
            })
            .take(500)
            .cloned()
            .collect()
    }

    fn is_ready(&self) -> bool {
        self.engine.lock().unwrap().is_some()
    }

    fn get_cache_stats(&self) -> (u64, u64, usize) {
        let lock = self.engine.lock().unwrap();
        if let Some(wrapper) = lock.as_ref() {
            (
                wrapper.cache_hits,
                wrapper.cache_misses,
                wrapper.cache.len(),
            )
        } else {
            (0, 0, 0)
        }
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
            get_cosmetic_filters,
            check_batch_urls,
            get_cache_stats
        ])
        .build()
}
async fn setup_filters(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    println!("🛡️ AdBlock: Checking filter cache...");

    // Create cache directory if it doesn't exist
    let cache_dir = app.path().app_data_dir()?.join("adblock_cache");
    fs::create_dir_all(&cache_dir)?;

    let mut all_content = String::new();
    let mut cosmetic_rules = Vec::new();
    let cache_duration = 24 * 60 * 60; // 24 hours in seconds

    for (name, url) in FILTER_LISTS {
        let filename = format!("{}.txt", name.replace(&[' ', '-', '\''][..], "_"));
        let cache_path = cache_dir.join(&filename);

        print!("   {} ... ", name);

        // Check if valid cache exists
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
            // Use cache
            match fs::read_to_string(&cache_path) {
                Ok(content) => {
                    println!("📁 cache");
                    content
                }
                Err(_) => {
                    // If cache read fails, download
                    download_and_cache(url, &cache_path).await?
                }
            }
        } else {
            // Download and save to cache
            download_and_cache(url, &cache_path).await?
        };

        // Extract cosmetic rules
        for line in content.lines() {
            let line = line.trim();
            if line.contains("##") && !line.contains("#@#") {
                cosmetic_rules.push(line.to_string());
            }
        }

        all_content.push_str(&content);
        all_content.push('\n');
    }

    // Process filters
    let mut filter_set = FilterSet::new(false);
    let parse_options = ParseOptions::default();

    for line in all_content.lines() {
        let line = line.trim();
        if !line.is_empty() && !line.starts_with('!') {
            let _ = filter_set.add_filter(line, parse_options);
        }
    }

    let engine = Engine::from_filter_set(filter_set, true);
    let state: tauri::State<AdBlockState> = app.state();
    state.set_engine(engine, cosmetic_rules);

    println!("🛡️ AdBlock ready");
    Ok(())
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
                // Save to cache
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

#[tauri::command]
pub async fn get_cosmetic_filters(
    hostname: String,
    state: tauri::State<'_, AdBlockState>,
) -> Result<Vec<String>, String> {
    Ok(state.get_cosmetic_filters(&hostname))
}

#[tauri::command]
pub async fn get_cache_stats(
    state: tauri::State<'_, AdBlockState>,
) -> Result<(u64, u64, usize), String> {
    Ok(state.get_cache_stats())
}
