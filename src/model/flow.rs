/// Flow (rate) variable

use serde::{Deserialize, Serialize};
use super::Expression;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Flow {
    pub name: String,
    pub equation: Expression,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub units: Option<String>,
}

impl Flow {
    pub fn new(name: &str, equation: &str) -> Self {
        Self {
            name: name.to_string(),
            equation: Expression::parse(equation).unwrap_or(Expression::Constant(0.0)),
            units: None,
        }
    }

    pub fn with_equation(mut self, equation: Expression) -> Self {
        self.equation = equation;
        self
    }

    pub fn with_units(mut self, units: &str) -> Self {
        self.units = Some(units.to_string());
        self
    }
}
