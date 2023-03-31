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
enum Snapshot {
    CPUData(Vec<f32>),
    MemData(HashMap<String, u64>),
}

#[derive(Clone)]
struct AppState {
    cpu_tx: broadcast::Sender<Snapshot>,
    mem_tx: broadcast::Sender<Snapshot>,
}

#[tokio::main]
async fn main() {
    let (cpu_tx, _) = broadcast::channel::<Snapshot>(1);
    let (mem_tx, _) = broadcast::channel::<Snapshot>(1);

    tracing_subscriber::fmt::init();

    let app_state = AppState { 
        cpu_tx: cpu_tx.clone(),
        mem_tx: mem_tx.clone(),
    };

    let router = Router::new()
        .route("/", get(root_get))
        .route("/index.mjs", get(indexmjs_get))
        .route("/index.css", get(indexcss_get))
        .route("/realtime/cpus", get(realtime_cpus_get))
        .route("/realtime/mem", get(realtime_mem_get))
        .with_state(app_state.clone());

    // Update CPU usage in the background
    tokio::task::spawn_blocking(move || {
        let mut sys = System::new();
        loop {
            sys.refresh_all();

            let v: Vec<f32> = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect();
            let _ = cpu_tx.send(Snapshot::CPUData(v));

            let mut map: HashMap<String, u64> = HashMap::new();
            map.insert("mem_used".to_string(), sys.used_memory());
            map.insert("mem_total".to_string(), sys.total_memory());
            let _ = mem_tx.send(Snapshot::MemData(map));

            std::thread::sleep(System::MINIMUM_CPU_UPDATE_INTERVAL);
        }
    });

    let server = Server::bind(
        &"0.0.0.0:7032"
        .parse()
        .unwrap()
    ).serve(router.into_make_service());
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
async fn realtime_cpus_get(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|ws: WebSocket| async { realtime_cpus_stream(state, ws).await })
}

async fn realtime_cpus_stream(app_state: AppState, mut ws: WebSocket) {
    let mut rx = app_state.cpu_tx.subscribe();

    while let Ok(msg) = rx.recv().await {
        ws.send(Message::Text(serde_json::to_string(&msg).unwrap()))
            .await
            .unwrap();
    }
}

#[axum::debug_handler]
async fn realtime_mem_get(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|ws: WebSocket| async { realtime_mem_stream(state, ws).await })
}

async fn realtime_mem_stream(app_state: AppState, mut ws: WebSocket) {
    let mut rx = app_state.mem_tx.subscribe();

    while let Ok(msg) = rx.recv().await {
        ws.send(Message::Text(serde_json::to_string(&msg).unwrap()))
            .await
            .unwrap();
    }
}
