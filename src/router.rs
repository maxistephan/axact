use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade, Path
    },
    routing::{get, Router},
    http::{Response, StatusCode},
    response::{Html, IntoResponse},
    body::StreamBody,
    Server,
};
use tokio::{sync::broadcast, fs::File};
use std::collections::HashMap;
use systemstat::Platform;
use sysinfo::{CpuExt, SystemExt};
use tokio_util::io::ReaderStream;
use crate::argparser::ArgParser;

use super::{AppState, Snapshot, RessourceData, gpu};

#[axum::debug_handler]
async fn root_get() -> impl IntoResponse {
    let markup = tokio::fs::read_to_string("/etc/axact/static/index.html").await.unwrap();

    Html(markup)
}

#[axum::debug_handler]
async fn indexmjs_get() -> impl IntoResponse {
    let markup = tokio::fs::read_to_string("/etc/axact/static/index.mjs").await.unwrap();

    Response::builder()
        .header("content-type", "application/javascript;charset=utf-8")
        .body(markup)
        .unwrap()
}

#[axum::debug_handler]
async fn indexcss_get() -> impl IntoResponse {
    let markup = tokio::fs::read_to_string("/etc/axact/static/index.css").await.unwrap();

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

async fn image_get(Path(path): Path<String>) -> impl IntoResponse {
    // `File` implements `AsyncRead`
    let file: File = match tokio::fs::File::open(format!("/etc/axact/static/images/{path}")).await {
        Ok(file) => file,
        Err(err) => return Err((StatusCode::NOT_FOUND, format!("File not found: {err}"))),
    };
    let f_suffix = match path.split('.').collect::<Vec<&str>>().last() {
        Some(suffix) => suffix.to_owned(),
        None => return Err((StatusCode::BAD_REQUEST, format!("{path} has no file suffix."))),
    };
    // convert the `AsyncRead` into a `Stream`
    let stream: ReaderStream<File> = ReaderStream::new(file);
    // convert the `Stream` into an `axum::body::HttpBody`
    let body: StreamBody<ReaderStream<File>> = StreamBody::new(stream);

    match Response::builder()
        .header("content-type", format!("image/{f_suffix}"))
        .header("attachment", format!("filename=\"{path}\""))
        .body(body) {
            Ok(resp) => Ok(resp),
            Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Error building response: {err}")))
    }
}

async fn background_task(
    ressource_tx: broadcast::Sender<Snapshot>,
    temperature_tx: broadcast::Sender<Snapshot>,
    show_gpu_temp: bool,
) {
    let mut sys = sysinfo::System::new();

    let statsys = systemstat::System::new();

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
        match statsys.cpu_temp() {
            Ok(cpu_temp) => {
                let mut temp_map: HashMap<String, f32> = HashMap::new();
                temp_map.insert("cpu_temp".to_string(), cpu_temp);
                if show_gpu_temp {
                    match gpu::get_gpu_avg_temp() {
                        Ok(gpu_temp) => {
                            temp_map.insert("gpu_temp".to_string(), gpu_temp as f32);
                        },
                        Err(err) => println!("\nGPU temp: {}", err)
                    }
                }
                let _ = temperature_tx.send(Snapshot::Temperature(temp_map));
            },
            Err(err) => println!("\nCPU temp: {}", err)
        }
        // Sleep MINIMUM_CPU_UPDATE_INTERVAL
        std::thread::sleep(sysinfo::System::MINIMUM_CPU_UPDATE_INTERVAL);
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
        .route("/images/*path", get(image_get))
        .with_state(app_state.clone());

    tokio::task::spawn(
        background_task(ressource_tx.clone(), temperature_tx.clone(), args.show_gpu_temp)
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
