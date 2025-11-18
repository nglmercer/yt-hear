// src-tauri/src/main.rs
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod adblock_plugin;
mod bridge;
mod tray;
mod window;
mod scripts;

use scripts::ScriptId;
use tauri::{AppHandle, Manager};

const MAIN_WINDOW_LABEL: &str = "main";

#[tauri::command]
fn debug_get_current_state(state: tauri::State<'_, bridge::AppState>) -> serde_json::Value {
    println!("🦀 Rust API: Reading state snapshot...");
    state.get_full_snapshot()
}
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(adblock_plugin::init())
        .manage(bridge::AppState::default()) 
        .invoke_handler(tauri::generate_handler![
            window_commands::minimize,
            window_commands::toggle_maximize,
            window_commands::close,
            bridge::push_telemetry,
            debug_get_current_state,
        ])
        .setup(|app| {
            setup_main_window(app)?;
            setup_tray(&app.handle())?;
            Ok(())
        })
        .on_page_load(|window, _| {
            if window.label() == MAIN_WINDOW_LABEL {
                println!("💉 Injecting Scripts...");
                for script_id in ScriptId::ALL_IN_ORDER {
                    if let Err(e) = window.eval(script_id.content()) {
                        eprintln!("❌ Script Error [{:?}]: {}", script_id, e);
                    }
                }
            }
        })
        .on_window_event(|window, event| {
            window::handle_window_event(window, event);
        })
        .on_menu_event(tray::handle_menu_event)
        .on_tray_icon_event(tray::handle_tray_event)
        .build(tauri::generate_context!())
        .expect("Failed to build Tauri application")
        .run(|app_handle, event| {
            window::handle_run_event(app_handle, &event);
        });
    std::panic::set_hook(Box::new(|info| {
        let msg = info.payload().downcast_ref::<&str>().unwrap_or(&"Unknown panic");
        let location = info.location().map(|l| l.to_string()).unwrap_or_default();
        eprintln!("🔥 CRITICAL RUST PANIC: {} at {}", msg, location);
        // Opcional: Escribir a un archivo de texto panic.log
        use std::io::Write;
        if let Ok(mut file) = std::fs::File::create("panic-crash.log") {
            let _ = writeln!(file, "Panic: {} at {}", msg, location);
        }
    }));
}

fn setup_main_window(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        #[cfg(debug_assertions)]
        window.open_devtools();
    }
    Ok(())
}

fn setup_tray(handle: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    tray::create_tray(handle).map_err(|e| {
        eprintln!("⚠️ Tray error: {}", e);
        e
    })
}

pub fn cleanup_and_exit(_app: &AppHandle) {
    std::thread::spawn(|| {
        std::thread::sleep(std::time::Duration::from_millis(100));
        std::process::exit(0);
    });
}

mod window_commands {
    use tauri::Window;

    #[tauri::command]
    pub fn minimize(window: Window) -> Result<(), String> {
        window.minimize().map_err(|e| e.to_string())
    }

    #[tauri::command]
    pub fn toggle_maximize(window: Window) -> Result<(), String> {
        let is_max = window.is_maximized().map_err(|e| e.to_string())?;
        if is_max {
            window.unmaximize().map_err(|e| e.to_string())
        } else {
            window.maximize().map_err(|e| e.to_string())
        }
    }

    #[tauri::command]
    pub fn close(window: Window) -> Result<(), String> {
        window.close().map_err(|e| e.to_string())
    }
}
