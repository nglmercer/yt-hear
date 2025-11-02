use tauri::{AppHandle, RunEvent, WindowEvent};

pub fn handle_window_event(window: &tauri::Window, event: &WindowEvent) {
    if let WindowEvent::CloseRequested { api, .. } = event {
        // Check if this is a direct window close request (not from system shutdown)
        if is_direct_close_request(window) {
            // Hide to tray instead of closing
            if let Err(e) = window.hide() {
                eprintln!("Failed to hide window: {}", e);
                // If hiding fails, allow the window to close
                return;
            }
            api.prevent_close();
        } else {
            // Allow close for system shutdown or other non-user-initiated closes
            // Don't prevent close in these cases
        }
    }
}

// Helper function to determine if this is a direct user close request
fn is_direct_close_request(_window: &tauri::Window) -> bool {
    // In a real implementation, you might want to track the source of the close request
    // For now, we'll assume all close requests are direct user requests
    // This could be enhanced with additional logic if needed
    true
}

pub fn handle_run_event(app_handle: &AppHandle, event: &RunEvent) -> bool {
    match event {
        RunEvent::ExitRequested { .. } => {
            // Allow the app to exit properly when requested
            // Don't prevent exit, let it happen naturally
            return true;
        }
        RunEvent::WindowEvent { label, event: WindowEvent::Destroyed, .. } => {
            // If main window is destroyed, clean up and exit
            if label == "main" {
                super::cleanup_and_exit(app_handle);
                return true;
            }
        }
        _ => {}
    }
    false
}
