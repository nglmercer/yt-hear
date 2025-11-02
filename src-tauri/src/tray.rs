use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

pub fn create_tray(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    // Create menu items with IDs for proper event handling
    let show = MenuItem::with_id(app, "show", "Show", true, None::<String>)?;
    let hide = MenuItem::with_id(app, "hide", "Hide", true, None::<String>)?;
    let separator1 = PredefinedMenuItem::separator(app)?;
    let settings = MenuItem::with_id(app, "settings", "Settings", true, None::<String>)?;
    let adblock_status = MenuItem::with_id(
        app,
        "adblock_status",
        "AdBlock Status",
        true,
        None::<String>,
    )?;
    let update_adblock = MenuItem::with_id(
        app,
        "update_adblock",
        "Update AdBlock Filters",
        true,
        None::<String>,
    )?;
    let about = MenuItem::with_id(app, "about", "About", true, None::<String>)?;
    let separator2 = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<String>)?;

    let menu = Menu::with_items(
        app,
        &[
            &show,
            &hide,
            &separator1,
            &settings,
            &adblock_status,
            &update_adblock,
            &about,
            &separator2,
            &quit,
        ],
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
            // Left click - toggle window visibility
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
            // Double click - show and focus window
            if button == MouseButton::Left {
                let window = app.get_webview_window("main").unwrap();
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        TrayIconEvent::Enter { .. } => {
            // Mouse enter - could show tooltip or update icon
        }
        TrayIconEvent::Leave { .. } => {
            // Mouse leave - could hide tooltip or revert icon
        }
        _ => {}
    }
}

pub fn handle_menu_event(app: &AppHandle, event: tauri::menu::MenuEvent) {
    match event.id.as_ref() {
        "show" => {
            let window = app.get_webview_window("main").unwrap();
            let _ = window.show();
            let _ = window.set_focus();
        }
        "hide" => {
            let window = app.get_webview_window("main").unwrap();
            let _ = window.hide();
        }
        "settings" => {
            let settings_window = app.get_webview_window("settings");
            match settings_window {
                Some(window) => {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
                None => {
                    let _ = tauri::WebviewWindowBuilder::new(
                        app,
                        "settings",
                        tauri::WebviewUrl::App("/settings".into()),
                    )
                    .title("Settings")
                    .inner_size(800.0, 600.0)
                    .resizable(false)
                    .center()
                    .build();
                }
            }
        }
        "adblock_status" => {
            let window = app.get_webview_window("main").unwrap();
            let _ = window.eval("if (window.checkAdBlockStatus) { checkAdBlockStatus().then(status => { alert('AdBlock Status: ' + JSON.stringify(status, null, 2)); }); } else { alert('AdBlock status check not available'); }");
        }
        "update_adblock" => {
            let window = app.get_webview_window("main").unwrap();
            let _ = window.eval("if (window.updateAdBlockFilters) { updateAdBlockFilters().then(count => { if (count !== null) { alert('AdBlock filters updated: ' + count + ' filters'); } else { alert('Failed to update AdBlock filters'); } }); } else { alert('AdBlock update not available'); }");
        }
        "about" => {
            let window = app.get_webview_window("main").unwrap();
            let _ = window.eval("alert('Tatar - YouTube Music Desktop App\\nVersion: 0.0.2\\nA lightweight YouTube Music desktop client');");
        }
        "quit" => {
            // Proper cleanup before exit
            super::cleanup_and_exit(app);
        }
        _ => {}
    }
}
