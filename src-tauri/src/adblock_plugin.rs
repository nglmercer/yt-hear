use tauri::{
    plugin::{Builder, TauriPlugin},
    Runtime, Manager,
};
use adblock::{Engine, FilterSet, lists::ParseOptions, request::Request};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, Duration};

const EASYLIST_URL: &str = "https://easylist.to/easylist/easylist.txt";
const EASYPRIVACY_URL: &str = "https://easylist.to/easylist/easyprivacy.txt";
const CACHE_DURATION_DAYS: u64 = 7; // Actualizar cada 7 días

// Motor de AdBlock - almacenamos los filtros cargados
struct AdBlockEngine {
    cache_dir: PathBuf,
    filter_set: FilterSet,
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
        let easylist = Self::load_filter_list(
            &cache_dir,
            "easylist.txt",
            EASYLIST_URL
        );
        
        let easyprivacy = Self::load_filter_list(
            &cache_dir,
            "easyprivacy.txt",
            EASYPRIVACY_URL
        );

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
            if filter_set.add_filter(filter, ParseOptions::default()).is_ok() {
                total_filters += 1;
            }
        }
        
        println!("✅ AdBlock Engine initialized with {} total filters", total_filters);

        Self {
            cache_dir,
            filter_set,
        }
    }

    fn should_block(&self, url: &str, source_url: &str, request_type: &str) -> bool {
        // Crear una nueva instancia del Engine para esta solicitud
        // Esto evita problemas de thread safety
        let engine = Engine::from_filter_set(self.filter_set.clone(), true);
        
        // Crear la request para el engine
        let request = Request::new(
            url,
            source_url,
            request_type,
        ).unwrap_or_else(|_| {
            Request::new(url, "", "other").unwrap()
        });

        let result = engine.check_network_request(&request);
        
        if result.matched {
            println!("🚫 Blocked: {} (type: {}, source: {})", url, request_type, source_url);
            true
        } else {
            false
        }
    }

    fn load_filter_list(cache_dir: &PathBuf, filename: &str, url: &str) -> Option<String> {
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
            let cache_dir = app.path()
                .app_cache_dir()
                .unwrap_or_else(|_| {
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
            println!("🌐 WebView ready, injecting AdBlock interceptor...");
            
            // Inyectar script que reporta las peticiones a Rust
            let script = r#"
            (function() {
                console.log('🛡️ AdBlock interceptor active');
                
                // Función helper para invocar comandos de Tauri
                async function invokeCommand(command, args) {
                    if (window.__TAURI__ && window.__TAURI__.core) {
                        return await window.__TAURI__.core.invoke(command, args);
                    } else if (window.__TAURI__ && window.__TAURI__.invoke) {
                        return await window.__TAURI__.invoke(command, args);
                    } else {
                        console.warn('Tauri API not available');
                        return false;
                    }
                }
                
                // Interceptar fetch
                const originalFetch = window.fetch;
                window.fetch = async function(...args) {
                    const url = typeof args[0] === 'string' ? args[0] : args[0]?.url || '';
                    const sourceUrl = window.location.href;
                    
                    try {
                        // Llamar al backend de Rust para verificar si debe bloquearse
                        const shouldBlock = await invokeCommand('plugin:adblock|check_adblock', {
                            url: url.toString(),
                            sourceUrl: sourceUrl,
                            requestType: 'xmlhttprequest'
                        });
                        
                        if (shouldBlock) {
                            console.log('🚫 Fetch blocked:', url);
                            return Promise.reject(new Error('Blocked by AdBlock'));
                        }
                    } catch (e) {
                        console.error('AdBlock check error:', e);
                    }
                    
                    return originalFetch.apply(this, args);
                };

                // Interceptar XMLHttpRequest
                const OriginalXHR = XMLHttpRequest;
                window.XMLHttpRequest = function() {
                    const xhr = new OriginalXHR();
                    const originalOpen = xhr.open;
                    
                    xhr.open = async function(method, url, ...rest) {
                        this._url = url;
                        const sourceUrl = window.location.href;
                        
                        try {
                            const shouldBlock = await invokeCommand('plugin:adblock|check_adblock', {
                                url: url.toString(),
                                sourceUrl: sourceUrl,
                                requestType: 'xmlhttprequest'
                            });
                            
                            if (shouldBlock) {
                                console.log('🚫 XHR blocked:', url);
                                this.abort();
                                return;
                            }
                        } catch (e) {
                            console.error('AdBlock check error:', e);
                        }
                        
                        return originalOpen.call(this, method, url, ...rest);
                    };
                    
                    return xhr;
                };

                // Observar elementos dinámicos (scripts, iframes, imágenes)
                const observer = new MutationObserver((mutations) => {
                    mutations.forEach((mutation) => {
                        mutation.addedNodes.forEach(async (node) => {
                            if (node.tagName && node.src) {
                                const sourceUrl = window.location.href;
                                let requestType = 'other';
                                
                                if (node.tagName === 'SCRIPT') requestType = 'script';
                                else if (node.tagName === 'IFRAME') requestType = 'subdocument';
                                else if (node.tagName === 'IMG') requestType = 'image';
                                
                                try {
                                    const shouldBlock = await invokeCommand('plugin:adblock|check_adblock', {
                                        url: node.src,
                                        sourceUrl: sourceUrl,
                                        requestType: requestType
                                    });
                                    
                                    if (shouldBlock) {
                                        console.log('🚫 Blocked element:', node.tagName, node.src);
                                        node.remove();
                                    }
                                } catch (e) {
                                    console.error('AdBlock check error:', e);
                                }
                            }
                        });
                    });
                });

                observer.observe(document.documentElement, {
                    childList: true,
                    subtree: true
                });

                console.log('✅ AdBlock interceptor initialized');
            })();
            "#;

            if let Err(e) = window.eval(script) {
                eprintln!("⚠️ Failed to inject AdBlock script: {}", e);
            }
        })
        .on_navigation(|_window, url| {
            println!("🔗 Navigation to: {}", url);
            true // Permitir navegación
        })
        .invoke_handler(tauri::generate_handler![check_adblock])
        .build()
}

#[tauri::command]
async fn check_adblock(
    url: String,
    source_url: String,
    request_type: String,
    state: tauri::State<'_, AdBlockEngine>,
) -> Result<bool, String> {
    Ok(state.should_block(&url, &source_url, &request_type))
}