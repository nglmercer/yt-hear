#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod adblock_plugin;
mod tray;
mod window;

use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .plugin(adblock_plugin::init())
        .invoke_handler(tauri::generate_handler![
            minimize_window,
            toggle_maximize,
            close_window,
        ])
        .setup(|app| {
            // DevTools en desarrollo
            #[cfg(debug_assertions)]
            {
                if let Some(window) = app.get_webview_window("main") {
                    window.open_devtools();
                }
            }

            // System tray
            if let Err(e) = tray::create_tray(app) {
                eprintln!("⚠️ Failed to create system tray: {}", e);
            }

            Ok(())
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
        .expect("Failed to run Tauri application")
        .run(|_app_handle, event| {
            if let tauri::RunEvent::ExitRequested { .. } = event {
                // Cleanup
            }
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
