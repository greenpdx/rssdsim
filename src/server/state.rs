use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use crate::model::Model;

#[derive(Clone)]
pub struct AppState {
    pub models: Arc<RwLock<HashMap<String, StoredModel>>>,
    pub simulations: Arc<RwLock<HashMap<String, SimulationHandle>>>,
}

#[derive(Clone)]
pub struct StoredModel {
    pub id: String,
    pub model: Model,
    pub created_at: i64,
}

pub struct SimulationHandle {
    pub id: String,
    pub model_id: String,
    pub status: SimulationStatus,
    pub current_time: f64,
    pub abort_handle: tokio::task::AbortHandle,
}

#[derive(Debug, Clone)]
pub enum SimulationStatus {
    Running,
    Paused,
    Completed,
    Error(String),
}

impl AppState {
    pub fn new() -> Self {
        Self {
            models: Arc::new(RwLock::new(HashMap::new())),
            simulations: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_model(&self, model: Model) -> String {
        let id = Uuid::new_v4().to_string();
        let stored = StoredModel {
            id: id.clone(),
            model,
            created_at: chrono::Utc::now().timestamp(),
        };

        self.models.write().await.insert(id.clone(), stored);
        id
    }

    pub async fn get_model(&self, id: &str) -> Option<Model> {
        self.models.read().await.get(id).map(|s| s.model.clone())
    }

    pub async fn get_stored_model(&self, id: &str) -> Option<StoredModel> {
        self.models.read().await.get(id).cloned()
    }

    pub async fn list_models(&self) -> Vec<StoredModel> {
        self.models.read().await.values().cloned().collect()
    }

    pub async fn remove_model(&self, id: &str) -> Option<StoredModel> {
        self.models.write().await.remove(id)
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
