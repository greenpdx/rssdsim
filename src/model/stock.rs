/// Stock (level) variable

use serde::{Deserialize, Serialize};
use super::Expression;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stock {
    pub name: String,
    pub initial: Expression,
    #[serde(default)]
    pub inflows: Vec<String>,
    #[serde(default)]
    pub outflows: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub units: Option<String>,
    #[serde(default)]
    pub non_negative: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_value: Option<f64>,
    /// Optional dimensions/subscripts for array variables
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<Vec<String>>,
}

impl Stock {
    pub fn new(name: &str, initial: &str) -> Self {
        Self {
            name: name.to_string(),
            initial: Expression::parse(initial).unwrap_or(Expression::Constant(0.0)),
            inflows: Vec::new(),
            outflows: Vec::new(),
            units: None,
            non_negative: false,
            max_value: None,
            dimensions: None,
        }
    }

    pub fn with_initial(mut self, initial: Expression) -> Self {
        self.initial = initial;
        self
    }

    pub fn with_inflows(mut self, inflows: Vec<String>) -> Self {
        self.inflows = inflows;
        self
    }

    pub fn with_outflows(mut self, outflows: Vec<String>) -> Self {
        self.outflows = outflows;
        self
    }

    pub fn with_units(mut self, units: &str) -> Self {
        self.units = Some(units.to_string());
        self
    }

    pub fn with_non_negative(mut self, non_negative: bool) -> Self {
        self.non_negative = non_negative;
        self
    }

    pub fn with_max_value(mut self, max_value: f64) -> Self {
        self.max_value = Some(max_value);
        self
    }

    pub fn with_dimensions(mut self, dimensions: Vec<String>) -> Self {
        self.dimensions = Some(dimensions);
        self
    }
}
