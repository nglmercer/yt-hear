#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod adblock_plugin;
mod tray;
mod window;

use tauri::Manager;

const ADBLOCK_INIT_SCRIPT: &str = include_str!("adblock.js");

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(adblock_plugin::init())
        .invoke_handler(tauri::generate_handler![
            minimize_window,
            toggle_maximize,
            close_window,
            adblock_plugin::is_url_blocked,
            adblock_plugin::is_adblock_ready,
            adblock_plugin::get_cosmetic_filters,
            adblock_plugin::get_cache_stats,
        ])
        .setup(|app| {
            // Inject AdBlock script into main window
            if let Some(window) = app.get_webview_window("main") {
                #[cfg(debug_assertions)]
                {
                    window.open_devtools();
                }
            }

            // System tray
            if let Err(e) = tray::create_tray(app.handle()) {
                eprintln!("⚠️ Error creating system tray: {}", e);
            }

            Ok(())
        })
        .on_page_load(|window, _event| {
            let _ = window.eval(ADBLOCK_INIT_SCRIPT);
        })
        .on_window_event(|window, event| {
            window::handle_window_event(window, event);
        })
        .on_menu_event(|app, event| {
            tray::handle_menu_event(app, event);
        })
        .on_tray_icon_event(|app, event| {
            tray::handle_tray_event(app, event);
        })
        .build(tauri::generate_context!())
        .expect("Error al ejecutar la aplicación Tauri")
        .run(|app_handle, event| {
            window::handle_run_event(app_handle, &event);
        });
}

#[tauri::command]
fn minimize_window(window: tauri::Window) {
    let _ = window.minimize();
}

#[tauri::command]
fn toggle_maximize(window: tauri::Window) {
    if window.is_maximized().unwrap_or(false) {
        let _ = window.unmaximize();
    } else {
        let _ = window.maximize();
    }
}

#[tauri::command]
fn close_window(window: tauri::Window) {
    let _ = window.close();
}

pub fn cleanup_and_exit(_app: &tauri::AppHandle) {
    std::thread::spawn(|| {
        std::thread::sleep(std::time::Duration::from_millis(100));
        std::process::exit(0);
    });
}
