/// Model parsers for JSON and YAML formats

use serde::{Deserialize, Serialize};
use crate::model::*;
use std::collections::HashMap;

pub trait ModelParser {
    fn parse(contents: &str) -> Result<Model, String>;
}

/// JSON model format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonModel {
    pub model: JsonModelContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonModelContent {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub time: TimeConfig,
    #[serde(default)]
    pub stocks: Vec<JsonStock>,
    #[serde(default)]
    pub flows: Vec<JsonFlow>,
    #[serde(default)]
    pub auxiliaries: Vec<JsonAuxiliary>,
    #[serde(default)]
    pub parameters: Vec<JsonParameter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonStock {
    pub name: String,
    pub initial: serde_json::Value,  // Can be number or string
    #[serde(default)]
    pub inflows: Vec<String>,
    #[serde(default)]
    pub outflows: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub units: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonFlow {
    pub name: String,
    pub equation: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub units: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonAuxiliary {
    pub name: String,
    pub equation: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub units: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonParameter {
    pub name: String,
    pub value: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub units: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl JsonModel {
    pub fn to_model(json: JsonModel) -> Result<Model, String> {
        let mut model = Model::new(&json.model.name);
        model.metadata.description = json.model.description;
        model.time = json.model.time;

        // Add parameters first (they might be referenced in initial values)
        for param in json.model.parameters {
            let p = Parameter {
                name: param.name,
                value: param.value,
                units: param.units,
                description: param.description,
            };
            model.add_parameter(p)?;
        }

        // Add stocks
        for stock in json.model.stocks {
            let initial_expr = match stock.initial {
                serde_json::Value::Number(n) => {
                    Expression::parse(&n.to_string())?
                }
                serde_json::Value::String(s) => {
                    Expression::parse(&s)?
                }
                _ => return Err("Initial value must be number or string".to_string()),
            };

            let s = Stock {
                name: stock.name,
                initial: initial_expr,
                inflows: stock.inflows,
                outflows: stock.outflows,
                units: stock.units,
                non_negative: false,
                max_value: None,
            };
            model.add_stock(s)?;
        }

        // Add flows
        for flow in json.model.flows {
            let f = Flow {
                name: flow.name,
                equation: Expression::parse(&flow.equation)?,
                units: flow.units,
            };
            model.add_flow(f)?;
        }

        // Add auxiliaries
        for aux in json.model.auxiliaries {
            let a = Auxiliary {
                name: aux.name,
                equation: Expression::parse(&aux.equation)?,
                units: aux.units,
            };
            model.add_auxiliary(a)?;
        }

        Ok(model)
    }
}

/// YAML model format (same structure as JSON)
pub type YamlModel = JsonModel;

impl ModelParser for JsonModel {
    fn parse(contents: &str) -> Result<Model, String> {
        let json_model: JsonModel = serde_json::from_str(contents)
            .map_err(|e| format!("JSON parse error: {}", e))?;
        Self::to_model(json_model)
    }
}

/// Parse YAML format (uses same structure as JSON)
pub fn parse_yaml(contents: &str) -> Result<Model, String> {
    let yaml_model: YamlModel = serde_yaml::from_str(contents)
        .map_err(|e| format!("YAML parse error: {}", e))?;
    JsonModel::to_model(yaml_model)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_json_simple() {
        let json = r#"
        {
            "model": {
                "name": "Test",
                "time": {
                    "start": 0,
                    "stop": 10,
                    "dt": 0.1
                },
                "stocks": [
                    {
                        "name": "Stock1",
                        "initial": 100
                    }
                ],
                "parameters": [
                    {
                        "name": "param1",
                        "value": 0.5
                    }
                ]
            }
        }
        "#;

        let model = JsonModel::parse(json).unwrap();
        assert_eq!(model.metadata.name, "Test");
        assert_eq!(model.stocks.len(), 1);
        assert_eq!(model.parameters.len(), 1);
    }
}
