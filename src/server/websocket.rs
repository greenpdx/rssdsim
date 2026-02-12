use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use std::collections::HashMap;
use crate::server::{error::AppError, state::AppState, types::WebSocketMessage};
use crate::simulation::{IntegrationMethod, SimulationConfig, SimulationEngine};

/// WebSocket upgrade handler
pub async fn handler(
    ws: WebSocketUpgrade,
    Path(model_id): Path<String>,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, model_id, state))
}

/// Handle WebSocket connection for simulation streaming
async fn handle_socket(socket: WebSocket, model_id: String, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    // Get model
    let model = match state.get_model(&model_id).await {
        Some(m) => m,
        None => {
            let _ = send_error(&mut sender, "Model not found").await;
            return;
        }
    };

    // Send start message
    let start_msg = WebSocketMessage::Start {
        model_name: model.metadata.name.clone(),
        variables: model.stocks.keys().cloned().collect(),
        time_config: crate::server::types::TimeConfig {
            start: model.time.start,
            stop: model.time.stop,
            dt: model.time.dt,
            units: model.time.units.clone().unwrap_or_else(|| "time".into()),
        },
    };

    if send_message(&mut sender, &start_msg).await.is_err() {
        return;
    }

    // Create simulation config
    let config = SimulationConfig {
        integration_method: IntegrationMethod::Euler,
        output_interval: None,
    };

    // Create simulation engine
    let mut engine = match SimulationEngine::new(model.clone(), config) {
        Ok(e) => e,
        Err(e) => {
            let _ = send_error(&mut sender, &format!("Failed to create simulation: {}", e)).await;
            return;
        }
    };

    // Run simulation and stream results
    let start_time = std::time::Instant::now();
    let mut step = 0;
    let decimation = 10; // Send every 10th step

    while engine.current_time() < model.time.stop {
        // Check for incoming messages (pause, parameter updates)
        while let Ok(Some(Ok(msg))) = tokio::time::timeout(
            std::time::Duration::from_millis(1),
            receiver.next()
        ).await {
            if let Message::Text(text) = msg {
                if let Err(e) = handle_client_message(&text.to_string(), &mut engine).await {
                    tracing::warn!("Error handling client message: {}", e);
                }
            }
        }

        // Step simulation
        if let Err(e) = engine.step() {
            let _ = send_error(&mut sender, &format!("Simulation error: {}", e)).await;
            return;
        }

        // Send data every Nth step
        if step % decimation == 0 {
            let state = engine.current_state();
            let mut values = HashMap::new();

            // Collect stock values
            for (name, value) in &state.stocks {
                values.insert(name.clone(), *value);
            }

            let data_msg = WebSocketMessage::Data {
                time: state.time,
                values,
            };

            if send_message(&mut sender, &data_msg).await.is_err() {
                return;
            }
        }

        step += 1;

        // Yield to allow other tasks to run
        tokio::task::yield_now().await;
    }

    // Send completion message
    let complete_msg = WebSocketMessage::Complete {
        total_steps: step,
        elapsed_ms: start_time.elapsed().as_millis(),
    };

    let _ = send_message(&mut sender, &complete_msg).await;
}

/// Send a message to the client
async fn send_message(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    msg: &WebSocketMessage,
) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string(msg)?;
    sender.send(Message::Text(json.into())).await?;
    Ok(())
}

/// Send an error message to the client
async fn send_error(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    error: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let msg = WebSocketMessage::Error {
        message: error.to_string(),
    };
    send_message(sender, &msg).await
}

/// Handle incoming messages from client (parameter updates, etc.)
async fn handle_client_message(
    text: &str,
    engine: &mut SimulationEngine,
) -> Result<(), String> {
    // Try to parse as parameter update
    if let Ok(update) = serde_json::from_str::<crate::server::types::ParameterUpdate>(text) {
        engine.set_parameter(&update.parameter, update.value)?;
        tracing::info!("Updated parameter {} = {}", update.parameter, update.value);
    }

    Ok(())
}
