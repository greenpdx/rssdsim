/// Model module - defines system dynamics model structure

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

pub mod stock;
pub mod flow;
pub mod auxiliary;
pub mod parameter;
pub mod expression;
pub mod dimension;
pub mod units;

pub use stock::Stock;
pub use flow::Flow;
pub use auxiliary::Auxiliary;
pub use parameter::Parameter;
pub use expression::Expression;
pub use dimension::{Dimension, DimensionManager, SubscriptRef};
pub use units::{DimensionalFormula, UnitChecker, BaseDimension};

/// Time configuration for simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeConfig {
    pub start: f64,
    pub stop: f64,
    pub dt: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub units: Option<String>,
}

impl Default for TimeConfig {
    fn default() -> Self {
        Self {
            start: 0.0,
            stop: 100.0,
            dt: 0.25,
            units: None,
        }
    }
}

/// Model metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
}

/// Complete system dynamics model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    #[serde(default)]
    pub metadata: ModelMetadata,
    pub time: TimeConfig,
    pub stocks: HashMap<String, Stock>,
    pub flows: HashMap<String, Flow>,
    pub auxiliaries: HashMap<String, Auxiliary>,
    pub parameters: HashMap<String, Parameter>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub dimensions: HashMap<String, Dimension>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub lookups: HashMap<String, crate::simulation::LookupTable>,
}

impl Model {
    pub fn new(name: &str) -> Self {
        Self {
            metadata: ModelMetadata {
                name: name.to_string(),
                description: None,
                author: None,
            },
            time: TimeConfig::default(),
            stocks: HashMap::new(),
            flows: HashMap::new(),
            auxiliaries: HashMap::new(),
            parameters: HashMap::new(),
            dimensions: HashMap::new(),
            lookups: HashMap::new(),
        }
    }

    pub fn add_stock(&mut self, stock: Stock) -> Result<(), String> {
        if self.stocks.contains_key(&stock.name) {
            return Err(format!("Stock '{}' already exists", stock.name));
        }
        self.stocks.insert(stock.name.clone(), stock);
        Ok(())
    }

    pub fn add_flow(&mut self, flow: Flow) -> Result<(), String> {
        if self.flows.contains_key(&flow.name) {
            return Err(format!("Flow '{}' already exists", flow.name));
        }
        self.flows.insert(flow.name.clone(), flow);
        Ok(())
    }

    pub fn add_auxiliary(&mut self, aux: Auxiliary) -> Result<(), String> {
        if self.auxiliaries.contains_key(&aux.name) {
            return Err(format!("Auxiliary '{}' already exists", aux.name));
        }
        self.auxiliaries.insert(aux.name.clone(), aux);
        Ok(())
    }

    pub fn add_parameter(&mut self, param: Parameter) -> Result<(), String> {
        if self.parameters.contains_key(&param.name) {
            return Err(format!("Parameter '{}' already exists", param.name));
        }
        self.parameters.insert(param.name.clone(), param);
        Ok(())
    }

    pub fn add_dimension(&mut self, dimension: Dimension) -> Result<(), String> {
        if self.dimensions.contains_key(&dimension.name) {
            return Err(format!("Dimension '{}' already exists", dimension.name));
        }
        self.dimensions.insert(dimension.name.clone(), dimension);
        Ok(())
    }

    pub fn add_lookup(&mut self, lookup: crate::simulation::LookupTable) -> Result<(), String> {
        if self.lookups.contains_key(&lookup.name) {
            return Err(format!("Lookup table '{}' already exists", lookup.name));
        }
        self.lookups.insert(lookup.name.clone(), lookup);
        Ok(())
    }

    /// Get variable value (parameter or from state)
    pub fn get_variable(&self, name: &str, state: &crate::simulation::SimulationState) -> Result<f64, String> {
        // Try parameter first
        if let Some(param) = self.parameters.get(name) {
            return Ok(param.value);
        }

        // Try stock
        if let Some(value) = state.stocks.get(name) {
            return Ok(*value);
        }

        // Try flow
        if let Some(value) = state.flows.get(name) {
            return Ok(*value);
        }

        // Try auxiliary
        if let Some(value) = state.auxiliaries.get(name) {
            return Ok(*value);
        }

        Err(format!("Variable '{}' not found", name))
    }
}

impl Default for ModelMetadata {
    fn default() -> Self {
        Self {
            name: "Untitled Model".to_string(),
            description: None,
            author: None,
        }
    }
}
