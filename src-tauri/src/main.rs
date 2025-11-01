#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod tray;
mod window;

use tauri::Manager;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command

fn main() {
    // Set up panic handler to ensure proper exit on compilation errors
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("Application panicked: {}", panic_info);
        // Force exit after a short delay to ensure cleanup
        std::thread::spawn(|| {
            std::thread::sleep(std::time::Duration::from_millis(100));
            std::process::exit(1);
        });
    }));

    let custom_ui_injector_js = r#"
    // Custom UI injector for YouTube Music
    console.log("Tatar - YouTube Music Desktop App");
    "#;

    let result = std::panic::catch_unwind(|| {
        tauri::Builder::default()
            .setup(move |app| {
                let window = app.get_webview_window("main").unwrap();
                if let Err(e) = window.eval(custom_ui_injector_js) {
                    eprintln!("Failed to inject custom UI: {}", e);
                }
                
                // Create system tray
                if let Err(e) = tray::create_tray(app) {
                    eprintln!("Failed to create system tray: {}", e);
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
            .expect("failed to run Tauri application")
            .run(|app_handle, event| {
                if window::handle_run_event(app_handle, &event) {
                    // If the event handler returns true, we should exit
                    return;
                }
            });
    });

    // Handle any panics from the application setup or runtime
    if let Err(_) = result {
        eprintln!("Application failed to start or encountered a critical error");
        // Force exit after a short delay to ensure cleanup
        std::thread::spawn(|| {
            std::thread::sleep(std::time::Duration::from_millis(100));
            std::process::exit(1);
        });
    }
}

pub fn cleanup_and_exit(_app: &tauri::AppHandle) {
    // Force exit after a short delay to ensure cleanup
    std::thread::spawn(|| {
        std::thread::sleep(std::time::Duration::from_millis(100));
        std::process::exit(0);
    });
}
