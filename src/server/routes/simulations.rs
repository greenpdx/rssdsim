use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;
use crate::server::{
    error::AppError,
    state::AppState,
    types::{SimulationStatus, StartSimulationRequest},
};

/// Start a new simulation
pub async fn start_simulation(
    State(state): State<AppState>,
    Json(request): Json<StartSimulationRequest>,
) -> Result<Json<SimulationStatus>, AppError> {
    // Verify model exists
    let _model = state
        .get_model(&request.model_id)
        .await
        .ok_or_else(|| AppError::NotFound("Model not found".into()))?;

    let sim_id = Uuid::new_v4().to_string();

    // For streaming simulations, client should connect to WebSocket endpoint
    // For non-streaming, we would run the simulation here and return results

    Ok(Json(SimulationStatus {
        id: sim_id,
        model_id: request.model_id,
        status: "created".into(),
        progress: 0.0,
        current_time: 0.0,
    }))
}

/// Get simulation status
pub async fn get_status(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<SimulationStatus>, AppError> {
    let simulations = state.simulations.read().await;
    let sim = simulations
        .get(&id)
        .ok_or_else(|| AppError::NotFound("Simulation not found".into()))?;

    Ok(Json(SimulationStatus {
        id: sim.id.clone(),
        model_id: sim.model_id.clone(),
        status: format!("{:?}", sim.status),
        progress: 0.0, // TODO: Calculate from current_time and model.time.stop
        current_time: sim.current_time,
    }))
}

/// Stop a running simulation
pub async fn stop_simulation(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut simulations = state.simulations.write().await;

    if let Some(sim) = simulations.remove(&id) {
        sim.abort_handle.abort();
        Ok(Json(serde_json::json!({ "message": "Simulation stopped" })))
    } else {
        Err(AppError::NotFound("Simulation not found".into()))
    }
}
