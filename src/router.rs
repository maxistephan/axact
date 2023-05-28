use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    routing::{get, Router},
    http::Response,
    response::{Html, IntoResponse},
    Server,
};
use tokio::sync::broadcast;
use std::collections::HashMap;
use systemstat::Platform;
use sysinfo::{CpuExt, SystemExt};
use crate::argparser::ArgParser;

use super::{AppState, Snapshot, RessourceData, TempData, fan, gpu};

#[axum::debug_handler]
async fn root_get() -> impl IntoResponse {
    let markup = tokio::fs::read_to_string("/usr/share/axact/index.html").await.unwrap();

    Html(markup)
}

#[axum::debug_handler]
async fn indexmjs_get() -> impl IntoResponse {
    let markup = tokio::fs::read_to_string("/usr/share/axact/index.mjs").await.unwrap();

    Response::builder()
        .header("content-type", "application/javascript;charset=utf-8")
        .body(markup)
        .unwrap()
}

#[axum::debug_handler]
async fn indexcss_get() -> impl IntoResponse {
    let markup = tokio::fs::read_to_string("/usr/share/axact/index.css").await.unwrap();

    Response::builder()
        .header("content-type", "text/css;charset=utf-8")
        .body(markup)
        .unwrap()
}

#[axum::debug_handler]
async fn realtime_ressources_get(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|ws: WebSocket| async { realtime_ressources_stream(state, ws).await })
}

async fn realtime_ressources_stream(app_state: AppState, mut ws: WebSocket) {
    let mut rx = app_state.ressource_tx.subscribe();

    while let Ok(msg) = rx.recv().await {
        ws.send(Message::Text(serde_json::to_string(&msg).unwrap()))
            .await
            .unwrap();
    }
}

#[axum::debug_handler]
async fn realtime_temperature_get(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|ws: WebSocket| async { realtime_temperature_stream(state, ws).await })
}

async fn realtime_temperature_stream(app_state: AppState, mut ws: WebSocket) {
    let mut rx = app_state.temperature_tx.subscribe();

    while let Ok(msg) = rx.recv().await {
        ws.send(Message::Text(serde_json::to_string(&msg).unwrap()))
            .await
            .unwrap();
    }
}

pub async fn start_server(args: ArgParser) {
    let (ressource_tx, _) = broadcast::channel::<Snapshot>(1);
    let (temperature_tx, _) = broadcast::channel::<Snapshot>(1);

    tracing_subscriber::fmt::init();

    let app_state = AppState {
        ressource_tx: ressource_tx.clone(),
        temperature_tx: temperature_tx.clone(),
    };

    let router = Router::new()
        .route("/", get(root_get))
        .route("/index.mjs", get(indexmjs_get))
        .route("/index.css", get(indexcss_get))
        .route("/realtime/ressources", get(realtime_ressources_get))
        .route("/realtime/temperature", get(realtime_temperature_get))
        .with_state(app_state.clone());

    tokio::task::spawn(
        background_task(ressource_tx.clone(), temperature_tx.clone(), args.use_liquidctl)
    );

    let server = Server::bind(
        &format!("{}:{}", args.host, args.port).as_str()
        .parse()
        .unwrap()
    ).serve(router.into_make_service());
    let addr = server.local_addr();
    println!("Listening on http://{addr}");
    server.await.unwrap();
}

async fn background_task(
    ressource_tx: broadcast::Sender<Snapshot>,
    temperature_tx: broadcast::Sender<Snapshot>,
    use_fan_curve: bool,
) {
    let mut sys = sysinfo::System::new();

    let statsys = systemstat::System::new();
    let mut last_speed: i32 = 20;

    if use_fan_curve {
        // initialize fan speed to MIN_FAN_PERCENTAGE
        fan::initialize_liquidctl();
        println!("Initializing fan speed to {}% ...", last_speed);
        fan::run_liquidctl(last_speed);
        println!();
    }

    loop {
        sys.refresh_all();
        let mut map : HashMap<String, RessourceData> = HashMap::new();

        let v: Vec<f32> = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect();
        map.insert("cpu".to_string(), RessourceData::CPUData(v));

        let mut m_map: HashMap<String, u64> = HashMap::new();
        m_map.insert("mem_used".to_string(), sys.used_memory());
        m_map.insert("mem_total".to_string(), sys.total_memory());
        map.insert("mem".to_string(), RessourceData::MemData(m_map));

        let _ = ressource_tx.send(Snapshot::Ressource(map));

        // Get the CPU Temp
        let cpu_temp_ret = statsys.cpu_temp();
        if let Err(err) = cpu_temp_ret {
            println!("\nCPU temp: {}", err);
        } else if let Ok(cpu_temp) = cpu_temp_ret {
            // Adjust fan speed if necessary
            let gpu_temp_ret = gpu::get_gpu_avg_temp();
            if let Err(err) = gpu_temp_ret {
                println!("\nGPU temp: {}", err);
            } else if let Ok(gpu_temp) = gpu_temp_ret {
                let mut temp_map: HashMap<String, TempData> = HashMap::new();
                if use_fan_curve {
                    last_speed = fan::liquidctl_modify_fan_speed(
                        cpu_temp,
                        gpu_temp,
                        last_speed,
                    );
                }
                temp_map.insert("cpu_temp".to_string(), TempData::Temperature(cpu_temp));
                temp_map.insert("gpu_temp".to_string(), TempData::Temperature(gpu_temp as f32));
                temp_map.insert("fan_speed".to_string(), TempData::FanSpeed(last_speed));
                let _ = temperature_tx.send(Snapshot::Temperature(temp_map));
            }
        }
        // Wait a second for next check
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}