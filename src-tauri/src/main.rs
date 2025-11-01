#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use tauri::SystemTray;
use tauri::*;
use tauri::{CustomMenuItem, SystemTrayMenu, SystemTrayMenuItem};

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command

fn main() {
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let show = CustomMenuItem::new("show".to_string(), "Show");

    let tray_menu = SystemTrayMenu::new()
        .add_item(show)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);

    let system_tray = SystemTray::new().with_menu(tray_menu);

    let settings_button_injector_js = "
    const open_settings = new Event('open-settings');
    let top_bar = document.getElementsByClassName('center-content style-scope ytmusic-nav-bar')[0];
    let settings_button = document.createElement('button');
    settings_button.innerText = 'Settings';
    settings_button.addEventListener('click', (event) => {;
        window.dispatchEvent('open-settings');
    });
    top_bar.prepend(settings_button);
    ";

    Builder::default()
        .system_tray(system_tray)
        .setup(|_app| {
            Ok(())
        })
        .on_page_load(|window, _| {
            window
                .eval(settings_button_injector_js)
                .expect("could not inject javascript")
        })
        .on_window_event(|event| match event.event() {
            WindowEvent::CloseRequested { api, .. } => {
                event.window().hide().unwrap();
                api.prevent_close();
            }
            _ => {}
        })
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "quit" => {
                    std::process::exit(0);
                }
                "show" => {
                    let window = app.get_window("main").unwrap();
                    let _ = window.show();
                }
                _ => {}
            },
            _ => {}
        })
        .run(generate_context!())
        .expect("failed");
}
