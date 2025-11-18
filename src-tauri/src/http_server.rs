use crate::bridge::AppState;
use axum::{
    extract::State,
    routing::get,
    Json, Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::oneshot;
use tower_http::cors::CorsLayer;

async fn get_full_state(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    Json(state.get_full_snapshot())
}

async fn get_song(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    Json(state.get_song_info())
}

pub async fn start_server(port: u16, app_state: Arc<AppState>) -> Result<String, String> {
    {
        let shutdown_guard = app_state.http_server_shutdown.lock().unwrap();
        if shutdown_guard.is_some() {
            return Err("Server is already running".to_string());
        }
    } 
    let app = Router::new()
        .route("/", get(get_full_state))
        .route("/song", get(get_song))
        .layer(CorsLayer::permissive())
        .with_state(app_state.clone());

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    println!("🚀 Starting HTTP Server on http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await
        .map_err(|e| format!("Failed to bind port {}: {}", port, e))?;

    let (tx, rx) = oneshot::channel();
    {
        let mut shutdown_guard = app_state.http_server_shutdown.lock().unwrap();
        if shutdown_guard.is_some() {
             return Err("Server started by another process concurrently".to_string());
        }
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

pub fn stop_server(app_state: &AppState) -> Result<String, String> {
    let mut shutdown_guard = app_state.http_server_shutdown.lock().unwrap();
    
    if let Some(tx) = shutdown_guard.take() {
        let _ = tx.send(());
        Ok("Server stopping...".to_string())
    } else {
        Err("Server is not running".to_string())
    }
}