use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    http::Response,
    response::{Html, IntoResponse},
    routing::get,
    Router, Server,
};
use sysinfo::{CpuExt, System, SystemExt};
use tokio::sync::broadcast;
use std::{collections::HashMap};
use serde_with::serde_as;
use serde::{Deserialize, Serialize};

#[serde_as]
#[derive(Clone, Serialize, Deserialize, Debug)]
enum HashValue {
    U64(u64),
    Vec32(Vec<f32>),
}

#[derive(Clone)]
struct AppState {
    tx: broadcast::Sender<Snapshot>,
}

type Snapshot = HashMap<String, HashValue>;

#[tokio::main]
async fn main() {
    let (tx, _) = broadcast::channel::<Snapshot>(1);

    tracing_subscriber::fmt::init();

    let app_state = AppState { tx: tx.clone() };

    let router = Router::new()
        .route("/", get(root_get))
        .route("/index.mjs", get(indexmjs_get))
        .route("/index.css", get(indexcss_get))
        .route("/realtime/ressources", get(realtime_ressource_get))
        .with_state(app_state.clone());

    // Update CPU usage in the background
    tokio::task::spawn_blocking(move || {
        let mut sys = System::new();
        loop {
            sys.refresh_all();
            let mut map: HashMap<String, HashValue> = HashMap::new();

            let v: Vec<f32> = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect();
            map.insert("cpus".to_string(), HashValue::Vec32(v));
            map.insert("mem_used".to_string(), HashValue::U64(sys.used_memory()));
            map.insert("mem_total".to_string(), HashValue::U64(sys.total_memory()));
            let _ = tx.send(map);
            std::thread::sleep(System::MINIMUM_CPU_UPDATE_INTERVAL);
        }
    });

    let server = Server::bind(&"0.0.0.0:7032".parse().unwrap()).serve(router.into_make_service());
    let addr = server.local_addr();
    println!("Listening on {addr}");

    server.await.unwrap();
}

#[axum::debug_handler]
async fn root_get() -> impl IntoResponse {
    let markup = tokio::fs::read_to_string("src/index.html").await.unwrap();

    Html(markup)
}

#[axum::debug_handler]
async fn indexmjs_get() -> impl IntoResponse {
    let markup = tokio::fs::read_to_string("src/index.mjs").await.unwrap();

    Response::builder()
        .header("content-type", "application/javascript;charset=utf-8")
        .body(markup)
        .unwrap()
}

#[axum::debug_handler]
async fn indexcss_get() -> impl IntoResponse {
    let markup = tokio::fs::read_to_string("src/index.css").await.unwrap();

    Response::builder()
        .header("content-type", "text/css;charset=utf-8")
        .body(markup)
        .unwrap()
}

#[axum::debug_handler]
async fn realtime_ressource_get(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|ws: WebSocket| async { realtime_ressource_stream(state, ws).await })
}

async fn realtime_ressource_stream(app_state: AppState, mut ws: WebSocket) {
    let mut rx = app_state.tx.subscribe();

    while let Ok(msg) = rx.recv().await {
        ws.send(Message::Text(serde_json::to_string(&msg).unwrap()))
            .await
            .unwrap();
    }
}
