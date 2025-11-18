use tauri::{AppHandle, Emitter, Runtime};
use serde_json::Value;
use std::sync::Mutex;

// Definimos el estado. Usamos Value (JSON) para flexibilidad, 
// pero podrías usar structs tipados si quisieras más control.
pub struct AppState {
    // Hacemos los campos privados para obligar a usar los getters
    last_song_info: Mutex<Value>,
    last_queue: Mutex<Value>,
    last_player_state: Mutex<Value>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            last_song_info: Mutex::new(Value::Null),
            last_queue: Mutex::new(Value::Null),
            last_player_state: Mutex::new(Value::Null),
        }
    }
}

impl AppState {
    // --- GETTERS PÚBLICOS (Para usar en tus plugins/API) ---

    /// Obtiene una copia del estado actual de la canción
    pub fn get_song_info(&self) -> Value {
        self.last_song_info.lock().unwrap().clone()
    }

    /// Obtiene una copia de la cola de reproducción
    pub fn get_queue(&self) -> Value {
        self.last_queue.lock().unwrap().clone()
    }

    /// Obtiene una copia del estado del reproductor (volumen, shuffle, etc)
    pub fn get_player_state(&self) -> Value {
        self.last_player_state.lock().unwrap().clone()
    }

    /// Obtiene todo el estado combinado (útil para un endpoint /status)
    pub fn get_full_snapshot(&self) -> serde_json::Value {
        serde_json::json!({
            "songInfo": self.get_song_info(),
            "queue": self.get_queue(),
            "playerState": self.get_player_state()
        })
    }
}

// --- COMANDO PARA RECIBIR DATOS DESDE JS ---

#[tauri::command]
pub fn push_telemetry<R: Runtime>(
    app: AppHandle<R>, 
    state: tauri::State<'_, AppState>,
    topic: String, 
    payload: Value
) {
    // 1. Actualizar la memoria caché (State)
    match topic.as_str() {
        "song-info" => *state.last_song_info.lock().unwrap() = payload.clone(),
        "queue" => *state.last_queue.lock().unwrap() = payload.clone(),
        "player-state" => *state.last_player_state.lock().unwrap() = payload.clone(),
        _ => println!("⚠️ Received unknown telemetry topic: {}", topic),
    }
    println!("✅ Received telemetry topic: {}", topic);
    // 2. Re-emitir como evento de Tauri
    // Esto permite que el frontend (si tienes UI propia) u otros plugins escuchen cambios
    // Eventos: "ytm:song-info", "ytm:queue", "ytm:player-state"
    let event_name = format!("ytm:{}", topic);
    if let Err(e) = app.emit(&event_name, payload) {
        println!("❌ Error emitting event {}: {}", event_name, e);
    }
}