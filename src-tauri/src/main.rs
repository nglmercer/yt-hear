// src-tauri/src/main.rs
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod adblock_plugin;
mod tray;
mod window;

use tauri::{AppHandle, Manager};

const ADBLOCK_INIT_SCRIPT: &str = include_str!("adblock.js");
const MAIN_WINDOW_LABEL: &str = "main";

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(adblock_plugin::init())
        .invoke_handler(tauri::generate_handler![
            window_commands::minimize,
            window_commands::toggle_maximize,
            window_commands::close,
        ])
        .setup(|app| {
            setup_main_window(app)?;
            setup_tray(&app.handle())?;
            Ok(())
        })
        .on_page_load(|window, _| {
            if window.label() == MAIN_WINDOW_LABEL {
                let _ = window.eval(ADBLOCK_INIT_SCRIPT);
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