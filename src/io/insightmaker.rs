/// InsightMaker format parser
///
/// InsightMaker uses a JSON-based format with a specific structure
/// that differs from standard XMILE but contains similar SD concepts

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::model::*;

/// InsightMaker top-level structure
#[derive(Debug, Deserialize, Serialize)]
pub struct InsightMakerModel {
    #[serde(default)]
    pub name: String,

    #[serde(default)]
    pub settings: InsightMakerSettings,

    #[serde(default)]
    pub primitives: Vec<InsightMakerPrimitive>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct InsightMakerSettings {
    #[serde(default)]
    pub start: f64,

    #[serde(default = "default_stop")]
    pub stop: f64,

    #[serde(default = "default_dt")]
    pub dt: f64,

    #[serde(default)]
    pub time_units: Option<String>,
}

fn default_stop() -> f64 { 100.0 }
fn default_dt() -> f64 { 1.0 }

#[derive(Debug, Deserialize, Serialize)]
pub struct InsightMakerPrimitive {
    pub id: String,

    #[serde(rename = "type")]
    pub primitive_type: String,

    pub name: String,

    #[serde(default)]
    pub value: Option<String>, // Initial value or equation

    #[serde(default)]
    pub equation: Option<String>,

    #[serde(default)]
    pub units: Option<String>,

    #[serde(default)]
    pub inflows: Vec<String>,

    #[serde(default)]
    pub outflows: Vec<String>,
}

pub fn parse_insightmaker(json: &str) -> Result<Model, String> {
    let im_model: InsightMakerModel = serde_json::from_str(json)
        .map_err(|e| format!("Failed to parse InsightMaker JSON: {}", e))?;

    let mut model = Model::new(&im_model.name);

    // Set time configuration
    model.time.start = im_model.settings.start;
    model.time.stop = im_model.settings.stop;
    model.time.dt = im_model.settings.dt;
    model.time.units = im_model.settings.time_units;

    // Build ID to name mapping for references
    let mut id_to_name: HashMap<String, String> = HashMap::new();
    for prim in &im_model.primitives {
        id_to_name.insert(prim.id.clone(), prim.name.clone());
    }

    // First pass: collect parameters and constants
    let mut params = Vec::new();
    for prim in &im_model.primitives {
        match prim.primitive_type.as_str() {
            "Variable" | "Constant" | "Parameter" => {
                if let Some(ref val_str) = prim.value {
                    if let Ok(val) = val_str.parse::<f64>() {
                        params.push((prim.name.clone(), val, prim.units.clone()));
                    }
                }
            }
            _ => {}
        }
    }

    // Add parameters
    for (name, value, units) in params {
        let param = Parameter {
            name: name.clone(),
            value,
            units,
            description: None,
        };
        model.add_parameter(param)?;
    }

    // Second pass: add stocks
    for prim in &im_model.primitives {
        if prim.primitive_type == "Stock" {
            let initial_expr = if let Some(ref eq) = prim.equation {
                eq.clone()
            } else if let Some(ref val) = prim.value {
                val.clone()
            } else {
                "0".to_string()
            };

            // Replace IDs with names in flow references
            let inflows: Vec<String> = prim.inflows.iter()
                .filter_map(|id| id_to_name.get(id).cloned())
                .collect();

            let outflows: Vec<String> = prim.outflows.iter()
                .filter_map(|id| id_to_name.get(id).cloned())
                .collect();

            let stock = Stock {
                name: prim.name.clone(),
                initial: Expression::parse(&initial_expr)?,
                inflows,
                outflows,
                units: prim.units.clone(),
                non_negative: false,
                max_value: None,
            };

            model.add_stock(stock)?;
        }
    }

    // Third pass: add flows
    for prim in &im_model.primitives {
        if prim.primitive_type == "Flow" {
            let eq = if let Some(ref equation) = prim.equation {
                equation.clone()
            } else if let Some(ref val) = prim.value {
                val.clone()
            } else {
                "0".to_string()
            };

            let flow = Flow {
                name: prim.name.clone(),
                equation: Expression::parse(&eq)?,
                units: prim.units.clone(),
            };

            model.add_flow(flow)?;
        }
    }

    // Fourth pass: add auxiliaries (converters/variables)
    for prim in &im_model.primitives {
        match prim.primitive_type.as_str() {
            "Converter" | "Variable" => {
                // Skip if already added as parameter
                if model.parameters.contains_key(&prim.name) {
                    continue;
                }

                if let Some(ref eq) = prim.equation {
                    let aux = Auxiliary {
                        name: prim.name.clone(),
                        equation: Expression::parse(eq)?,
                        units: prim.units.clone(),
                    };

                    model.add_auxiliary(aux)?;
                }
            }
            _ => {}
        }
    }

    Ok(model)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_insightmaker_simple() {
        let json = r#"
        {
            "name": "Test Model",
            "settings": {
                "start": 0,
                "stop": 100,
                "dt": 0.25
            },
            "primitives": [
                {
                    "id": "1",
                    "type": "Stock",
                    "name": "Population",
                    "value": "100",
                    "inflows": ["2"]
                },
                {
                    "id": "2",
                    "type": "Flow",
                    "name": "growth",
                    "equation": "Population * 0.1"
                },
                {
                    "id": "3",
                    "type": "Parameter",
                    "name": "growth_rate",
                    "value": "0.1"
                }
            ]
        }
        "#;

        let model = parse_insightmaker(json).unwrap();
        assert_eq!(model.metadata.name, "Test Model");
        assert_eq!(model.stocks.len(), 1);
        assert_eq!(model.flows.len(), 1);
        assert_eq!(model.parameters.len(), 1);
    }
}
