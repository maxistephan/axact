use tokio::sync::broadcast;
use std::collections::HashMap;
use serde_with::serde_as;
use serde::{Deserialize, Serialize};

pub mod argparser;
// mod fan;
mod gpu;
pub mod router;

#[serde_as]
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum RessourceData {
    MemData(HashMap<String, u64>),
    CPUData(Vec<f32>),
}

#[serde_as]
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum Snapshot {
    Ressource(HashMap<String, RessourceData>),
    Temperature(HashMap<String, f32>)
}

#[derive(Clone)]
pub struct AppState {
    pub ressource_tx: broadcast::Sender<Snapshot>,
    pub temperature_tx: broadcast::Sender<Snapshot>,
}
