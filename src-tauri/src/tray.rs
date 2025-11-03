use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

pub fn create_tray(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let show = MenuItem::with_id(app, "show", "Show", true, None::<String>)?;
    let hide = MenuItem::with_id(app, "hide", "Hide", true, None::<String>)?;
    let separator1 = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<String>)?;

+   let menu = Menu::with_items(app, &[&show, &hide, &separator1, &quit])?;

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
        "quit" => {
            // Proper cleanup before exit
            super::cleanup_and_exit(app);
        }
        _ => {}
    }
}
