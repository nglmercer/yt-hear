use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};

pub fn create_tray(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let show = MenuItem::with_id(app, "show", "Show", true, None::<String>)?;
    let hide = MenuItem::with_id(app, "hide", "Hide", true, None::<String>)?;
    let separator1 = PredefinedMenuItem::separator(app)?;
    let toggle_api = MenuItem::with_id(
        app,
        "toggle_api",
        "Start/Stop API Server",
        true,
        None::<String>,
    )?;
    let separator2 = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<String>)?;

    let menu = Menu::with_items(
        app,
        &[&show, &hide, &separator1, &toggle_api, &separator2, &quit],
    )?;

    let _tray = TrayIconBuilder::new()
        .menu(&menu)
        .tooltip("Tatar - YouTube Music")
        .show_menu_on_left_click(false)
        .icon(app.default_window_icon().unwrap().clone())
        .build(app)?;

    Ok(())
}

pub fn handle_tray_event(app: &AppHandle, event: TrayIconEvent) {
    match event {
        TrayIconEvent::Click { button, .. } => {
            if button == MouseButton::Left {
                let window = app.get_webview_window("main").unwrap();
                if window.is_visible().unwrap_or(false) {
                    let _ = window.hide();
                } else {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        }
        TrayIconEvent::DoubleClick { button, .. } => {
            if button == MouseButton::Left {
                let window = app.get_webview_window("main").unwrap();
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        _ => {}
    }
}

pub fn handle_menu_event(app: &AppHandle, event: tauri::menu::MenuEvent) {
    match event.id().as_ref() {
        "show" => {
            let window = app.get_webview_window("main").unwrap();
            let _ = window.show();
            let _ = window.set_focus();
        }
        "hide" => {
            let window = app.get_webview_window("main").unwrap();
            let _ = window.hide();
        }
        "toggle_api" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
                println!("📡 Sending request-server-port event to JS");
                if let Err(e) = window.emit("request-server-port", ()) {
                    eprintln!("Error emitting event: {}", e);
                }
            }
        }
        "quit" => {
            super::cleanup_and_exit(app);
        }
        _ => {}
    }
}
