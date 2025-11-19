use crate::bridge::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post,patch},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::oneshot;
use tower_http::cors::CorsLayer;

// --- Structs de Payload ---
#[derive(Deserialize)]
struct SeekPayload {
    seconds: f64,
}

#[derive(Deserialize)]
struct VolumePayload {
    volume: f64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct QueueAddPayload {
    video_id: String,
    insert_position: Option<String>,
}

#[derive(Deserialize)]
struct QueueIndexPayload {
    index: usize,
}

#[derive(Deserialize)]
struct SearchPayload {
    query: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct QueueMovePayload {
    from_index: usize,
    to_index: usize,
}
// --- HANDLERS GET ---
async fn move_queue_item(
    State(state): State<AppState>, 
    Json(payload): Json<QueueMovePayload>
) -> Json<Value> {
    emit_cmd(
        &state,
        json!({ 
            "action": "moveInQueue", 
            "fromIndex": payload.from_index,
            "toIndex": payload.to_index 
        }),
    )
    .await;
    Json(json!({ "status": "ok" }))
}

async fn toggle_shuffle(State(state): State<AppState>) -> Json<Value> {
    // Necesitas implementar esto en JS también (ver paso 4)
    emit_cmd(&state, json!({ "action": "toggleShuffle" })).await;
    Json(json!({ "status": "ok" }))
}

async fn toggle_repeat(State(state): State<AppState>) -> Json<Value> {
    // Necesitas implementar esto en JS también (ver paso 4)
    emit_cmd(&state, json!({ "action": "toggleRepeat" })).await;
    Json(json!({ "status": "ok" }))
}
async fn get_song(State(state): State<AppState>) -> Result<Json<Value>, (StatusCode, String)> {
    match state.request_live_data("get-song-info", 1000).await {
        Ok(data) => Ok(Json(data)),
        Err(e) => Err((
            StatusCode::GATEWAY_TIMEOUT,
            json!({ "error": e }).to_string(),
        )),
    }
}

async fn get_queue(State(state): State<AppState>) -> Result<Json<Value>, (StatusCode, String)> {
    match state.request_live_data("get-queue", 2000).await {
        Ok(data) => Ok(Json(data)),
        Err(e) => Err((
            StatusCode::GATEWAY_TIMEOUT,
            json!({ "error": e }).to_string(),
        )),
    }
}

async fn get_volume(State(state): State<AppState>) -> Result<Json<Value>, (StatusCode, String)> {
    match state.request_live_data("get-volume", 1000).await {
        Ok(data) => Ok(Json(data)),
        Err(e) => Err((
            StatusCode::GATEWAY_TIMEOUT,
            json!({ "error": e }).to_string(),
        )),
    }
}

// --- HANDLERS DE COMANDOS ---

async fn emit_cmd(state: &AppState, action: Value) {
    state.emit_to_frontend("ytm:command", action).await;
}

// Wrappers
// CORRECCIÓN: Se añade .await a todas las llamadas emit_cmd
async fn next(State(state): State<AppState>) -> Json<Value> {
    emit_cmd(&state, json!({ "action": "next" })).await;
    Json(json!({ "status": "ok" }))
}
async fn previous(State(state): State<AppState>) -> Json<Value> {
    emit_cmd(&state, json!({ "action": "previous" })).await;
    Json(json!({ "status": "ok" }))
}
async fn play(State(state): State<AppState>) -> Json<Value> {
    emit_cmd(&state, json!({ "action": "play" })).await;
    Json(json!({ "status": "ok" }))
}
async fn pause(State(state): State<AppState>) -> Json<Value> {
    emit_cmd(&state, json!({ "action": "pause" })).await;
    Json(json!({ "status": "ok" }))
}
async fn toggle_play(State(state): State<AppState>) -> Json<Value> {
    emit_cmd(&state, json!({ "action": "playPause" })).await;
    Json(json!({ "status": "ok" }))
}
async fn like(State(state): State<AppState>) -> Json<Value> {
    emit_cmd(&state, json!({ "action": "like" })).await;
    Json(json!({ "status": "ok" }))
}
async fn dislike(State(state): State<AppState>) -> Json<Value> {
    emit_cmd(&state, json!({ "action": "dislike" })).await;
    Json(json!({ "status": "ok" }))
}
async fn toggle_mute(State(state): State<AppState>) -> Json<Value> {
    emit_cmd(&state, json!({ "action": "toggleMute" })).await;
    Json(json!({ "status": "ok" }))
}
async fn clear_queue(State(state): State<AppState>) -> Json<Value> {
    emit_cmd(&state, json!({ "action": "clearQueue" })).await;
    Json(json!({ "status": "ok" }))
}

// Comandos con argumentos
async fn seek_to(State(state): State<AppState>, Json(payload): Json<SeekPayload>) -> Json<Value> {
    emit_cmd(
        &state,
        json!({ "action": "seek", "value": payload.seconds }),
    )
    .await;
    Json(json!({ "status": "ok" }))
}

async fn go_back(State(state): State<AppState>, Json(payload): Json<SeekPayload>) -> Json<Value> {
    emit_cmd(
        &state,
        json!({ "action": "goBack", "value": payload.seconds }),
    )
    .await;
    Json(json!({ "status": "ok" }))
}

async fn go_forward(
    State(state): State<AppState>,
    Json(payload): Json<SeekPayload>,
) -> Json<Value> {
    emit_cmd(
        &state,
        json!({ "action": "goForward", "value": payload.seconds }),
    )
    .await;
    Json(json!({ "status": "ok" }))
}

async fn set_volume(
    State(state): State<AppState>,
    Json(payload): Json<VolumePayload>,
) -> Json<Value> {
    emit_cmd(
        &state,
        json!({ "action": "setVolume", "value": payload.volume }),
    )
    .await;
    Json(json!({ "status": "ok" }))
}

async fn add_to_queue(
    State(state): State<AppState>,
    Json(payload): Json<QueueAddPayload>,
) -> Json<Value> {
    emit_cmd(
        &state,
        json!({
            "action": "addToQueue",
            "videoId": payload.video_id,
            "insertPosition": payload.insert_position.unwrap_or("INSERT_AT_END".to_string())
        }),
    )
    .await;
    Json(json!({ "status": "ok" }))
}

async fn remove_queue_item(State(state): State<AppState>, Path(index): Path<usize>) -> Json<Value> {
    emit_cmd(
        &state,
        json!({ "action": "removeFromQueue", "value": index }),
    )
    .await;
    Json(json!({ "status": "ok" }))
}

async fn set_queue_index(
    State(state): State<AppState>,
    Json(payload): Json<QueueIndexPayload>,
) -> Json<Value> {
    emit_cmd(
        &state,
        json!({ "action": "setQueueIndex", "value": payload.index }),
    )
    .await;
    Json(json!({ "status": "ok" }))
}

async fn search(State(state): State<AppState>, Json(payload): Json<SearchPayload>) -> Json<Value> {
    emit_cmd(
        &state,
        json!({ "action": "search", "query": payload.query }),
    )
    .await;
    Json(json!({ "status": "triggered" }))
}

// --- SERVER ---

pub async fn start_server(port: u16, app_state: Arc<AppState>) -> Result<String, String> {
    {
        let shutdown_guard = app_state.http_server_shutdown.lock().await;
        if shutdown_guard.is_some() {
            return Err("Server is already running".to_string());
        }
    }

    let api_v1: Router<AppState> = Router::new()
        .route("/song", get(get_song))
        .route("/queue", get(get_queue).post(add_to_queue))
        .route("/queue", patch(set_queue_index))
        .route("/queue/:index", delete(remove_queue_item))
        .route("/queue/index", post(set_queue_index))
        .route("/queue/move", post(move_queue_item)) // NUEVA RUTA
        .route("/volume", get(get_volume).post(set_volume))
        .route("/toggle-mute", post(toggle_mute))
        .route("/play", post(play))
        .route("/pause", post(pause))
        .route("/toggle-play", post(toggle_play))
        .route("/next", post(next))
        .route("/previous", post(previous))
        .route("/seek-to", post(seek_to))
        .route("/go-back", post(go_back))
        .route("/go-forward", post(go_forward))
        .route("/like", post(like))
        .route("/dislike", post(dislike))
        .route("/search", post(search))
        .route("/shuffle", post(toggle_shuffle))
        .route("/repeat", post(toggle_repeat))
        .route("/clear-queue", post(clear_queue));
    let app = Router::new()
        .nest("/api/v1", api_v1)
        .layer(CorsLayer::permissive())
        .with_state((*app_state).clone());

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("🚀 Starting HTTP Server on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| format!("Failed to bind port {}: {}", port, e))?;

    let (tx, rx) = oneshot::channel();
    {
        let mut shutdown_guard = app_state.http_server_shutdown.lock().await;
        *shutdown_guard = Some(tx);
    }

    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app)
            .with_graceful_shutdown(async {
                rx.await.ok();
            })
            .await
        {
            eprintln!("❌ HTTP Server Error: {}", e);
        }
        println!("🛑 HTTP Server Stopped");
    });

    Ok(format!("Server running on port {}", port))
}

pub async fn stop_server(app_state: &AppState) -> Result<String, String> {
    let mut shutdown_guard = app_state.http_server_shutdown.lock().await;

    if let Some(tx) = shutdown_guard.take() {
        let _ = tx.send(());
        Ok("Server stopping...".to_string())
    } else {
        Err("Server is not running".to_string())
    }
}
