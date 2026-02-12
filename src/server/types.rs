use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub created_at: i64,
    pub stocks_count: usize,
    pub flows_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StartSimulationRequest {
    pub model_id: String,
    #[serde(default = "default_integrator")]
    pub integrator: String,
    #[serde(default = "default_stream")]
    pub stream: bool,
    pub decimation: Option<usize>,
    pub parameters: Option<HashMap<String, f64>>,
}

fn default_integrator() -> String {
    "euler".to_string()
}

fn default_stream() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SimulationStatus {
    pub id: String,
    pub model_id: String,
    pub status: String,
    pub progress: f64,
    pub current_time: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimeConfig {
    pub start: f64,
    pub stop: f64,
    pub dt: f64,
    pub units: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebSocketMessage {
    #[serde(rename = "start")]
    Start {
        model_name: String,
        variables: Vec<String>,
        time_config: TimeConfig,
    },
    #[serde(rename = "data")]
    Data {
        time: f64,
        values: HashMap<String, f64>,
    },
    #[serde(rename = "complete")]
    Complete {
        total_steps: usize,
        elapsed_ms: u128,
    },
    #[serde(rename = "error")]
    Error { message: String },
}

#[derive(Debug, Deserialize)]
pub struct ParameterUpdate {
    pub parameter: String,
    pub value: f64,
}
