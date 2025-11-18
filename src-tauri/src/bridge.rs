use serde_json::Value;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Runtime};
use tokio::sync::oneshot;

pub struct AppState {
    last_song_info: Mutex<Value>,
    last_queue: Mutex<Value>,
    last_player_state: Mutex<Value>,
    pub http_server_shutdown: Mutex<Option<oneshot::Sender<()>>>,
    pub app_handle: Mutex<Option<AppHandle>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            last_song_info: Mutex::new(Value::Null),
            last_queue: Mutex::new(Value::Null),
            last_player_state: Mutex::new(Value::Null),
            http_server_shutdown: Mutex::new(None),
            app_handle: Mutex::new(None),
        }
    }
}

impl AppState {
    pub fn get_song_info(&self) -> Value {
        self.last_song_info.lock().unwrap().clone()
    }

    pub fn get_queue(&self) -> Value {
        self.last_queue.lock().unwrap().clone()
    }

    pub fn get_player_state(&self) -> Value {
        self.last_player_state.lock().unwrap().clone()
    }

    pub fn get_full_snapshot(&self) -> serde_json::Value {
        serde_json::json!({
            "songInfo": self.get_song_info(),
            "queue": self.get_queue(),
            "playerState": self.get_player_state()
        })
    }
    pub fn emit_to_frontend(&self, event: &str, payload: Value) {
        let handle_guard = self.app_handle.lock().unwrap();
        if let Some(handle) = handle_guard.as_ref() {
            if let Err(e) = handle.emit(event, payload) {
                eprintln!("❌ Error emitiendo evento {}: {}", event, e);
            } else {
                println!("📡 Evento emitido a Tauri: {}", event);
            }
        } else {
            eprintln!("⚠️ No se pudo emitir: AppHandle no inicializado");
        }
    }
}

#[tauri::command]
pub fn push_telemetry<R: Runtime>(
    app: AppHandle<R>,
    state: tauri::State<'_, Arc<AppState>>,
    topic: String,
    payload: Value,
) {
    match topic.as_str() {
        "song-info" => *state.last_song_info.lock().unwrap() = payload.clone(),
        "queue" => *state.last_queue.lock().unwrap() = payload.clone(),
        "player-state" => *state.last_player_state.lock().unwrap() = payload.clone(),
        _ => println!("⚠️ Received unknown telemetry topic: {}", topic),
    }
    let event_name = format!("ytm:{}", topic);
    if let Err(e) = app.emit(&event_name, payload) {
        println!("❌ Error emitting event {}: {}", event_name, e);
    }
}
