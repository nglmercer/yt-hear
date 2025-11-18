use tauri::{AppHandle, Emitter, Runtime};
use serde_json::Value;
use std::sync::{Mutex, Arc}; // <--- IMPORTA ARC
use tokio::sync::oneshot;

pub struct AppState {
    last_song_info: Mutex<Value>,
    last_queue: Mutex<Value>,
    last_player_state: Mutex<Value>,
    pub http_server_shutdown: Mutex<Option<oneshot::Sender<()>>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            last_song_info: Mutex::new(Value::Null),
            last_queue: Mutex::new(Value::Null),
            last_player_state: Mutex::new(Value::Null),
            http_server_shutdown: Mutex::new(None),
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
}

#[tauri::command]
pub fn push_telemetry<R: Runtime>(
    app: AppHandle<R>, 
    // CORRECCIÓN: El estado ahora es Arc<AppState>, no AppState directo
    state: tauri::State<'_, Arc<AppState>>, 
    topic: String, 
    payload: Value
) {
    // Gracias a Deref, podemos usar state.lock() directamente aunque sea un Arc
    match topic.as_str() {
        "song-info" => *state.last_song_info.lock().unwrap() = payload.clone(),
        "queue" => *state.last_queue.lock().unwrap() = payload.clone(),
        "player-state" => *state.last_player_state.lock().unwrap() = payload.clone(),
        _ => println!("⚠️ Received unknown telemetry topic: {}", topic),
    }
    
    // Logging reducido para no saturar la consola
    if topic == "song-info" {
         println!("✅ Updated Song Info");
    }

    let event_name = format!("ytm:{}", topic);
    if let Err(e) = app.emit(&event_name, payload) {
        println!("❌ Error emitting event {}: {}", event_name, e);
    }
}