use axum::{
    extract::{Multipart, Path, State},
    Json,
};
use crate::server::{error::AppError, state::AppState, types::ModelInfo};
use crate::{io, model::Model};

/// List all uploaded models
pub async fn list_models(State(state): State<AppState>) -> Result<Json<Vec<ModelInfo>>, AppError> {
    let models = state.list_models().await;

    let infos: Vec<ModelInfo> = models
        .iter()
        .map(|stored| ModelInfo {
            id: stored.id.clone(),
            name: stored.model.metadata.name.clone(),
            created_at: stored.created_at,
            stocks_count: stored.model.stocks.len(),
            flows_count: stored.model.flows.len(),
        })
        .collect();

    Ok(Json(infos))
}

/// Upload a new model file
pub async fn upload_model(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<ModelInfo>, AppError> {
    let mut file_data = Vec::new();
    let mut filename = String::new();

    // Extract file from multipart form
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Multipart error: {}", e)))?
    {
        if field.name() == Some("file") {
            filename = field
                .file_name()
                .unwrap_or("model")
                .to_string();
            file_data = field
                .bytes()
                .await
                .map_err(|e| AppError::BadRequest(format!("Failed to read file: {}", e)))?
                .to_vec();
        }
    }

    if file_data.is_empty() {
        return Err(AppError::BadRequest("No file provided".into()));
    }

    // Parse model based on file extension
    let model = parse_model_from_bytes(&file_data, &filename)?;

    // Store model
    let id = state.add_model(model.clone()).await;

    Ok(Json(ModelInfo {
        id,
        name: model.metadata.name,
        created_at: chrono::Utc::now().timestamp(),
        stocks_count: model.stocks.len(),
        flows_count: model.flows.len(),
    }))
}

/// Get a specific model by ID
pub async fn get_model(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ModelInfo>, AppError> {
    // Get the stored model with metadata
    let stored = state.models.read().await.get(&id).cloned()
        .ok_or_else(|| AppError::NotFound("Model not found".into()))?;

    Ok(Json(ModelInfo {
        id: stored.id,
        name: stored.model.metadata.name,
        created_at: stored.created_at,
        stocks_count: stored.model.stocks.len(),
        flows_count: stored.model.flows.len(),
    }))
}

/// Delete a model
pub async fn delete_model(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    state
        .remove_model(&id)
        .await
        .ok_or_else(|| AppError::NotFound("Model not found".into()))?;

    Ok(Json(serde_json::json!({ "message": "Model deleted" })))
}

/// Get model structure with layout
pub async fn get_model_structure(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<crate::visualization::LayoutResult>, AppError> {
    let model = state
        .get_model(&id)
        .await
        .ok_or_else(|| AppError::NotFound("Model not found".into()))?;

    // Compute layout using hierarchical algorithm
    let layout = crate::visualization::LayoutEngine::hierarchical_layout(&model);

    Ok(Json(layout))
}

/// Helper function to parse model from bytes based on filename
fn parse_model_from_bytes(data: &[u8], filename: &str) -> Result<Model, AppError> {
    let contents = String::from_utf8_lossy(data);

    if filename.ends_with(".xmile") || filename.ends_with(".stmx") || filename.ends_with(".itmx") {
        io::xmile::parse_xmile(&contents).map_err(|e| {
            AppError::BadRequest(format!("Failed to parse XMILE: {}", e))
        })
    } else if filename.ends_with(".json") {
        // Try InsightMaker format first
        if let Ok(model) = io::insightmaker::parse_insightmaker(&contents) {
            return Ok(model);
        }

        // Fall back to standard JSON
        let json_model: io::parser::JsonModel = serde_json::from_str(&contents)
            .map_err(|e| AppError::BadRequest(format!("Failed to parse JSON: {}", e)))?;

        io::parser::JsonModel::to_model(json_model)
            .map_err(|e| AppError::BadRequest(format!("Invalid model: {}", e)))
    } else if filename.ends_with(".yaml") || filename.ends_with(".yml") {
        io::parser::parse_yaml(&contents)
            .map_err(|e| AppError::BadRequest(format!("Failed to parse YAML: {}", e)))
    } else {
        Err(AppError::BadRequest(format!(
            "Unsupported file format: {}",
            filename
        )))
    }
}
