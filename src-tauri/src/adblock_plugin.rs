use adblock::{lists::ParseOptions, request::Request, Engine, FilterSet};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime, State, Webview,
};

const EASYLIST_URL: &str = "https://easylist.to/easylist/easylist.txt";
const EASYPRIVACY_URL: &str = "https://easylist.to/easylist/easyprivacy.txt";
const CACHE_DURATION_DAYS: u64 = 7; // Actualizar cada 7 días

// Motor de AdBlock - almacenamos el filter set y recursos
// Usamos Arc<Mutex<>> para thread safety
struct AdBlockEngine {
    cache_dir: PathBuf,
    filter_set: Arc<Mutex<FilterSet>>,
    resources: Arc<Mutex<HashMap<String, String>>>, // Simplificado a base64 strings
}

impl AdBlockEngine {
    fn new(cache_dir: PathBuf) -> Self {
        println!("🛡️ Initializing AdBlock Engine...");

        // Crear directorio de caché si no existe
        if !cache_dir.exists() {
            fs::create_dir_all(&cache_dir).ok();
        }

        let mut filter_set = FilterSet::new(false);

        // Cargar listas de filtros
        let easylist = Self::load_filter_list(&cache_dir, "easylist.txt", EASYLIST_URL);

        let easyprivacy = Self::load_filter_list(&cache_dir, "easyprivacy.txt", EASYPRIVACY_URL);

        // Agregar filtros básicos manuales
        let manual_filters = vec![
            // Google Ads (respaldo)
            "||doubleclick.net^",
            "||googlesyndication.com^",
            "||googleadservices.com^",
            "||google-analytics.com^",
            "||googletagmanager.com^",
            "||googletagservices.com^",
            // Facebook tracking
            "||facebook.com/tr^",
            "||connect.facebook.net^",
            // Patrones comunes
            "*/ads.js",
            "*/advertising.js",
            // Filtros cosméticos comunes
            "##.ad-container",
            "##.ads",
            "##.advertisement",
            "##.google-ad",
            "##.adsbygoogle",
        ];

        let mut total_filters = 0;

        // Agregar EasyList
        if let Some(content) = easylist {
            let count = Self::add_filters_from_content(&mut filter_set, &content);
            println!("✅ Loaded {} filters from EasyList", count);
            total_filters += count;
        } else {
            println!("⚠️ EasyList not available, using manual filters");
        }

        // Agregar EasyPrivacy
        if let Some(content) = easyprivacy {
            let count = Self::add_filters_from_content(&mut filter_set, &content);
            println!("✅ Loaded {} filters from EasyPrivacy", count);
            total_filters += count;
        } else {
            println!("⚠️ EasyPrivacy not available");
        }

        // Agregar filtros manuales
        for filter in manual_filters {
            if filter_set
                .add_filter(filter, ParseOptions::default())
                .is_ok()
            {
                total_filters += 1;
            }
        }

        // Inicializar recursos para reemplazo
        let mut resources = HashMap::new();

        // Agregar recursos básicos
        resources.insert(
            "1x1-transparent.gif".to_string(),
            "R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7".to_string(), // base64 de 1x1 transparent gif
        );

        println!(
            "✅ AdBlock Engine initialized with {} total filters",
            total_filters
        );

        Self {
            cache_dir,
            filter_set: Arc::new(Mutex::new(filter_set)),
            resources: Arc::new(Mutex::new(resources)),
        }
    }

    fn should_block(&self, url: &str, source_url: &str, request_type: &str) -> bool {
        // Usar Mutex para thread safety
        if let Ok(filter_set) = self.filter_set.lock() {
            // Crear engine temporal para esta solicitud
            let engine = Engine::from_filter_set(filter_set.clone(), true);

            // Crear la request para el engine
            let request = Request::new(url, source_url, request_type)
                .unwrap_or_else(|_| Request::new(url, "", "other").unwrap());

            let result = engine.check_network_request(&request);

            if result.matched {
                println!(
                    "🚫 Blocked: {} (type: {}, source: {})",
                    url, request_type, source_url
                );
                true
            } else {
                false
            }
        } else {
            eprintln!("❌ Failed to acquire lock on filter set");
            false
        }
    }

    fn get_cosmetic_filters(&self, url: &str) -> Vec<String> {
        if let Ok(filter_set) = self.filter_set.lock() {
            // Crear engine temporal para esta solicitud
            let engine = Engine::from_filter_set(filter_set.clone(), true);

            // Obtener filtros cosméticos para la URL
            let cosmetic_resources = engine.url_cosmetic_resources(url);

            // Extraer los selectores CSS
            let mut selectors = Vec::new();

            // Agregar selectores de hide rules
            for filter in &cosmetic_resources.hide_selectors {
                selectors.push(filter.clone());
            }

            // Agregar selectores de generic hide
            if cosmetic_resources.generichide {
                selectors.push("html > body".to_string());
            }

            selectors
        } else {
            eprintln!("❌ Failed to acquire lock on filter set for cosmetic filters");
            Vec::new()
        }
    }

    fn get_resource_replacement(&self, url: &str) -> Option<String> {
        if let Ok(resources) = self.resources.lock() {
            // Extraer el nombre del recurso de la URL
            if let Some(resource_name) = url.split('/').next_back() {
                resources.get(resource_name).cloned()
            } else {
                None
            }
        } else {
            eprintln!("❌ Failed to acquire lock on resources");
            None
        }
    }

    fn update_filters(&self) -> Result<usize, String> {
        println!("🔄 Updating AdBlock filters...");

        // Forzar la descarga de nuevas listas
        let easylist = Self::load_filter_list(&self.cache_dir, "easylist.txt", EASYLIST_URL);

        let easyprivacy =
            Self::load_filter_list(&self.cache_dir, "easyprivacy.txt", EASYPRIVACY_URL);

        if let Ok(mut filter_set) = self.filter_set.lock() {
            let mut total_filters = 0;

            // Crear nuevo filter set
            let mut new_filter_set = FilterSet::new(false);

            // Agregar EasyList
            if let Some(content) = easylist {
                let count = Self::add_filters_from_content(&mut new_filter_set, &content);
                println!("✅ Updated {} filters from EasyList", count);
                total_filters += count;
            }

            // Agregar EasyPrivacy
            if let Some(content) = easyprivacy {
                let count = Self::add_filters_from_content(&mut new_filter_set, &content);
                println!("✅ Updated {} filters from EasyPrivacy", count);
                total_filters += count;
            }

            // Reemplazar el filter set
            *filter_set = new_filter_set;

            println!(
                "✅ AdBlock Engine updated with {} total filters",
                total_filters
            );
            Ok(total_filters)
        } else {
            Err("Failed to acquire lock on filter set".to_string())
        }
    }

    fn load_filter_list(cache_dir: &Path, filename: &str, url: &str) -> Option<String> {
        let cache_path = cache_dir.join(filename);

        // Verificar si existe caché y si está actualizado
        if cache_path.exists() {
            if let Ok(metadata) = fs::metadata(&cache_path) {
                if let Ok(modified) = metadata.modified() {
                    let age = SystemTime::now()
                        .duration_since(modified)
                        .unwrap_or(Duration::from_secs(0));

                    // Si el caché tiene menos de CACHE_DURATION_DAYS días, usarlo
                    if age < Duration::from_secs(CACHE_DURATION_DAYS * 24 * 60 * 60) {
                        println!("📦 Using cached {}", filename);
                        if let Ok(content) = fs::read_to_string(&cache_path) {
                            return Some(content);
                        }
                    } else {
                        println!("🔄 Cache expired for {}, downloading...", filename);
                    }
                }
            }
        } else {
            println!("📥 Downloading {} for the first time...", filename);
        }

        // Descargar nueva lista
        match Self::download_filter_list(url) {
            Ok(content) => {
                // Guardar en caché
                if let Err(e) = fs::write(&cache_path, &content) {
                    eprintln!("⚠️ Failed to cache {}: {}", filename, e);
                } else {
                    println!("💾 Cached {} successfully", filename);
                }
                Some(content)
            }
            Err(e) => {
                eprintln!("❌ Failed to download {}: {}", filename, e);

                // Intentar usar caché antiguo como fallback
                if cache_path.exists() {
                    println!("🔙 Using old cache for {}", filename);
                    fs::read_to_string(&cache_path).ok()
                } else {
                    None
                }
            }
        }
    }

    fn download_filter_list(url: &str) -> Result<String, Box<dyn std::error::Error>> {
        use std::io::Read;

        // Crear cliente HTTP con timeout
        let client = ureq::AgentBuilder::new()
            .timeout_connect(std::time::Duration::from_secs(10))
            .timeout_read(std::time::Duration::from_secs(30))
            .build();

        let response = client.get(url).call()?;
        let mut content = String::new();
        response.into_reader().read_to_string(&mut content)?;

        Ok(content)
    }

    fn add_filters_from_content(filter_set: &mut FilterSet, content: &str) -> usize {
        let mut count = 0;

        for line in content.lines() {
            let line = line.trim();

            // Ignorar comentarios y líneas vacías
            if line.is_empty() || line.starts_with('!') || line.starts_with('[') {
                continue;
            }

            // Agregar filtro
            if filter_set.add_filter(line, ParseOptions::default()).is_ok() {
                count += 1;
            }
        }

        count
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("adblock")
        .setup(|app, _api| {
            // Obtener directorio de caché de la app
            let cache_dir = app.path().app_cache_dir().unwrap_or_else(|_| {
                // Fallback a directorio temporal
                std::env::temp_dir().join("tauri-adblock")
            });

            println!("📁 AdBlock cache directory: {:?}", cache_dir);

            // Inicializar el motor de AdBlock
            let adblock = AdBlockEngine::new(cache_dir);
            app.manage(adblock);

            Ok(())
        })
        .on_webview_ready(|window| {
            println!("🌐 WebView ready, setting up AdBlock...");

            // Inyectar script para AdBlock
            if let Err(e) = setup_adblock_injection(&window) {
                eprintln!("⚠️ Failed to inject AdBlock script: {}", e);
            }
        })
        .on_navigation(|window, url| {
            println!("🔗 Navigation to: {}", url);

            // Aplicar filtros cosméticos para la nueva página
            if let Err(e) = apply_cosmetic_filters(window, url.as_ref()) {
                eprintln!("⚠️ Failed to apply cosmetic filters: {}", e);
            }

            true // Permitir navegación
        })
        .invoke_handler(tauri::generate_handler![
            check_adblock,
            get_cosmetic_filters,
            update_filters,
            get_resource_replacement
        ])
        .build()
}

fn setup_adblock_injection<R: Runtime>(
    window: &Webview<R>,
) -> Result<(), Box<dyn std::error::Error>> {
    let script = r#"
    (function() {
        console.log('🛡️ AdBlock injection started');
        
        // Crear objeto global para AdBlock
        window.AdBlock = {
            enabled: true,
            
            // Interceptar fetch/XHR requests
            interceptRequests: function() {
                const originalFetch = window.fetch;
                const originalXHROpen = XMLHttpRequest.prototype.open;
                
                // Intercept fetch
                window.fetch = function(...args) {
                    const url = args[0];
                    const options = args[1] || {};
                    
                    // Verificar si la URL debe ser bloqueada
                    if (window.__TAURI__) {
                        window.__TAURI__.invoke('check_adblock', {
                            url: typeof url === 'string' ? url : url.url,
                            sourceUrl: window.location.href,
                            requestType: options.method || 'GET'
                        }).then(blocked => {
                            if (blocked) {
                                console.log('🚫 Blocked request:', url);
                                return Promise.reject(new Error('Request blocked by AdBlock'));
                            }
                        }).catch(() => {
                            // Continuar si hay error en la verificación
                        });
                    }
                    
                    return originalFetch.apply(this, args);
                };
                
                // Intercept XMLHttpRequest
                XMLHttpRequest.prototype.open = function(method, url, ...args) {
                    this._url = url;
                    this._method = method;
                    
                    if (window.__TAURI__) {
                        window.__TAURI__.invoke('check_adblock', {
                            url: url,
                            sourceUrl: window.location.href,
                            requestType: method
                        }).then(blocked => {
                            if (blocked) {
                                console.log('🚫 Blocked XHR:', url);
                                this.abort();
                            }
                        }).catch(() => {
                            // Continuar si hay error en la verificación
                        });
                    }
                    
                    return originalXHROpen.apply(this, [method, url, ...args]);
                };
                
                console.log('✅ Request interception enabled');
            },
            
            // Aplicar filtros cosméticos
            applyCosmeticFilters: function(selectors) {
                if (!Array.isArray(selectors) || selectors.length === 0) {
                    return;
                }
                
                console.log('🎨 Applying cosmetic filters:', selectors.length);
                
                // Crear o actualizar stylesheet
                let style = document.getElementById('adblock-styles');
                if (!style) {
                    style = document.createElement('style');
                    style.id = 'adblock-styles';
                    document.head.appendChild(style);
                }
                
                // Agregar nuevos selectores
                const css = selectors.map(selector => `${selector} { display: none !important; }`).join('\n');
                style.textContent += css;
                
                // Ocultar elementos existentes
                selectors.forEach(selector => {
                    try {
                        const elements = document.querySelectorAll(selector);
                        elements.forEach(el => {
                            el.style.display = 'none';
                            el.setAttribute('data-adblocked', 'true');
                        });
                    } catch (e) {
                        console.warn('Invalid selector:', selector, e);
                    }
                });
                
                console.log('✅ Cosmetic filters applied');
            },
            
            // Inicializar
            init: function() {
                this.interceptRequests();
                
                // Obtener filtros cosméticos para la página actual
                if (window.__TAURI__) {
                    window.__TAURI__.invoke('get_cosmetic_filters', {
                        url: window.location.href
                    }).then(selectors => {
                        this.applyCosmeticFilters(selectors);
                    }).catch(e => {
                        console.warn('Failed to get cosmetic filters:', e);
                    });
                }
                
                // Observar cambios en el DOM
                const observer = new MutationObserver(() => {
                    if (window.__TAURI__) {
                        window.__TAURI__.invoke('get_cosmetic_filters', {
                            url: window.location.href
                        }).then(selectors => {
                            this.applyCosmeticFilters(selectors);
                        }).catch(() => {});
                    }
                });
                
                observer.observe(document.body, {
                    childList: true,
                    subtree: true
                });
            }
        };
        
        // Inicializar cuando el DOM esté listo
        if (document.readyState === 'loading') {
            document.addEventListener('DOMContentLoaded', () => window.AdBlock.init());
        } else {
            window.AdBlock.init();
        }
        
        console.log('✅ AdBlock injection completed');
    })();
    "#;

    window.eval(script)?;
    Ok(())
}

fn apply_cosmetic_filters<R: Runtime>(
    window: &Webview<R>,
    url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let script = format!(
        r#"
    (function() {{
        console.log('🎨 Applying cosmetic filters for:', {});
        
        if (window.AdBlock && window.__TAURI__) {{
            window.__TAURI__.invoke('get_cosmetic_filters', {{
                url: {}
            }}).then(selectors => {{
                window.AdBlock.applyCosmeticFilters(selectors);
            }}).catch(e => {{
                console.warn('Failed to get cosmetic filters:', e);
            }});
        }}
    }})();
    "#,
        serde_json::json!(url),
        serde_json::json!(url)
    );

    window.eval(&script)?;
    Ok(())
}

#[tauri::command]
async fn check_adblock(
    url: String,
    source_url: String,
    request_type: String,
    state: State<'_, AdBlockEngine>,
) -> Result<bool, String> {
    Ok(state.should_block(&url, &source_url, &request_type))
}

#[tauri::command]
async fn get_cosmetic_filters(
    url: String,
    state: State<'_, AdBlockEngine>,
) -> Result<Vec<String>, String> {
    Ok(state.get_cosmetic_filters(&url))
}

#[tauri::command]
async fn update_filters(state: State<'_, AdBlockEngine>) -> Result<usize, String> {
    state.update_filters()
}

#[tauri::command]
async fn get_resource_replacement(
    url: String,
    state: State<'_, AdBlockEngine>,
) -> Result<Option<String>, String> {
    Ok(state.get_resource_replacement(&url))
}
