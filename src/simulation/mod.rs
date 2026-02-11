/// Simulation module - executes model simulations

use std::collections::HashMap;
use crate::model::Model;

pub mod engine;
pub mod integrator;

pub use engine::SimulationEngine;
pub use integrator::{Integrator, EulerIntegrator};

/// Simulation state at a point in time
#[derive(Debug, Clone)]
pub struct SimulationState {
    pub time: f64,
    pub stocks: HashMap<String, f64>,
    pub flows: HashMap<String, f64>,
    pub auxiliaries: HashMap<String, f64>,
}

impl SimulationState {
    pub fn new() -> Self {
        Self {
            time: 0.0,
            stocks: HashMap::new(),
            flows: HashMap::new(),
            auxiliaries: HashMap::new(),
        }
    }

    pub fn initialize_from_model(model: &Model) -> Result<Self, String> {
        let mut state = Self::new();
        state.time = model.time.start;

        // Initialize stocks with their initial values
        for (name, stock) in &model.stocks {
            let initial_value = stock.initial.evaluate(&crate::model::expression::EvaluationContext {
                model,
                state: &state,
                time: model.time.start,
            })?;
            state.stocks.insert(name.clone(), initial_value);
        }

        // Initialize flows to zero
        for name in model.flows.keys() {
            state.flows.insert(name.clone(), 0.0);
        }

        // Initialize auxiliaries
        for name in model.auxiliaries.keys() {
            state.auxiliaries.insert(name.clone(), 0.0);
        }

        Ok(state)
    }
}

impl Default for SimulationState {
    fn default() -> Self {
        Self::new()
    }
}

/// Simulation configuration
#[derive(Debug, Clone)]
pub struct SimulationConfig {
    pub integration_method: IntegrationMethod,
    pub output_interval: Option<f64>,
}

#[derive(Debug, Clone, Copy)]
pub enum IntegrationMethod {
    Euler,
    RK4,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            integration_method: IntegrationMethod::Euler,
            output_interval: None,
        }
    }
}

/// Complete simulation results
#[derive(Debug, Clone)]
pub struct SimulationResults {
    pub times: Vec<f64>,
    pub states: Vec<SimulationState>,
}

impl SimulationResults {
    pub fn new() -> Self {
        Self {
            times: Vec::new(),
            states: Vec::new(),
        }
    }

    pub fn add_point(&mut self, time: f64, state: SimulationState) {
        self.times.push(time);
        self.states.push(state);
    }

    pub fn get_variable_series(&self, var_name: &str) -> Option<Vec<f64>> {
        let mut series = Vec::new();

        for state in &self.states {
            if let Some(val) = state.stocks.get(var_name) {
                series.push(*val);
            } else if let Some(val) = state.flows.get(var_name) {
                series.push(*val);
            } else if let Some(val) = state.auxiliaries.get(var_name) {
                series.push(*val);
            } else {
                return None;
            }
        }

        Some(series)
    }
}

impl Default for SimulationResults {
    fn default() -> Self {
        Self::new()
    }
}
