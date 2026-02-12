/// Graph layout algorithms for system dynamics models

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::model::Model;
use super::graph::{build_graph_from_model, GraphNodeType, GraphEdgeType};

/// Node type for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeType {
    Stock,
    Flow,
    Auxiliary,
    Parameter,
}

/// Edge type for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EdgeType {
    Inflow,
    Outflow,
    Dependency,
}

/// Node layout information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeLayout {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: NodeType,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub units: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub equation: Option<String>,
}

/// Edge layout information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeLayout {
    pub id: String,
    pub from: String,
    pub to: String,
    #[serde(rename = "type")]
    pub edge_type: EdgeType,
}

/// Complete layout result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutResult {
    pub nodes: Vec<NodeLayout>,
    pub edges: Vec<EdgeLayout>,
    pub width: f64,
    pub height: f64,
}

/// Layout engine for computing graph layouts
pub struct LayoutEngine;

impl LayoutEngine {
    /// Compute hierarchical layout for a model
    pub fn hierarchical_layout(model: &Model) -> LayoutResult {
        let _graph = build_graph_from_model(model);

        // Target layout dimensions (fit typical screen)
        let target_width: f64 = 1400.0;
        let target_height: f64 = 800.0;

        // Node spacing
        let node_spacing_x: f64 = 200.0;
        let node_spacing_y: f64 = 180.0;

        // Calculate max nodes per row based on target width
        let max_nodes_per_row = ((target_width - 100.0) / node_spacing_x).floor() as usize;
        let max_nodes_per_row = max_nodes_per_row.max(3); // At least 3 per row

        // Organize nodes into layers
        let mut layers: Vec<Vec<String>> = vec![Vec::new(), Vec::new(), Vec::new()];

        // Layer 0: Stocks (primary elements)
        for name in model.stocks.keys() {
            layers[0].push(name.clone());
        }

        // Layer 1: Flows
        for name in model.flows.keys() {
            layers[1].push(name.clone());
        }

        // Layer 2: Auxiliaries and Parameters
        for name in model.auxiliaries.keys() {
            layers[2].push(name.clone());
        }
        for name in model.parameters.keys() {
            layers[2].push(name.clone());
        }

        // Compute positions with wrapping
        let mut nodes = Vec::new();

        let mut current_y = 100.0; // Start position

        for (_layer_idx, layer) in layers.iter().enumerate() {
            if layer.is_empty() {
                continue;
            }

            // Calculate how many rows needed for this layer
            let num_rows = (layer.len() + max_nodes_per_row - 1) / max_nodes_per_row;

            for row in 0..num_rows {
                let start_idx = row * max_nodes_per_row;
                let end_idx = ((row + 1) * max_nodes_per_row).min(layer.len());
                let nodes_in_row = end_idx - start_idx;

                // Center the row
                let row_width = (nodes_in_row as f64) * node_spacing_x;
                let start_x = (target_width - row_width) / 2.0 + node_spacing_x / 2.0;

                for (i, name) in layer[start_idx..end_idx].iter().enumerate() {
                    let x = start_x + (i as f64) * node_spacing_x;
                    let y = current_y;

                    // Determine node type and dimensions
                    let (node_type, width, height) = if model.stocks.contains_key(name) {
                        (NodeType::Stock, 120.0, 80.0)
                    } else if model.flows.contains_key(name) {
                        (NodeType::Flow, 100.0, 60.0)
                    } else if model.auxiliaries.contains_key(name) {
                        (NodeType::Auxiliary, 80.0, 80.0)
                    } else {
                        (NodeType::Parameter, 70.0, 70.0)
                    };

                    // Get additional info
                    let (value, units, equation) = if let Some(stock) = model.stocks.get(name) {
                        (None, stock.units.clone(), None)
                    } else if let Some(flow) = model.flows.get(name) {
                        (None, flow.units.clone(), Some(format!("{}", flow.equation)))
                    } else if let Some(aux) = model.auxiliaries.get(name) {
                        (None, aux.units.clone(), Some(format!("{}", aux.equation)))
                    } else if let Some(param) = model.parameters.get(name) {
                        (Some(param.value), param.units.clone(), None)
                    } else {
                        (None, None, None)
                    };

                    nodes.push(NodeLayout {
                        id: name.clone(),
                        node_type,
                        x,
                        y,
                        width,
                        height,
                        label: Some(name.clone()),
                        value,
                        units,
                        equation,
                    });
                }

                current_y += node_spacing_y;
            }

            // Add extra spacing between layers
            current_y += 50.0;
        }

        // Build edges
        let mut edges = Vec::new();
        let edge_id_counter = std::cell::Cell::new(0);

        for (stock_name, stock) in &model.stocks {
            for inflow in &stock.inflows {
                if model.flows.contains_key(inflow) {
                    edge_id_counter.set(edge_id_counter.get() + 1);
                    edges.push(EdgeLayout {
                        id: format!("edge_{}", edge_id_counter.get()),
                        from: inflow.clone(),
                        to: stock_name.clone(),
                        edge_type: EdgeType::Inflow,
                    });
                }
            }

            for outflow in &stock.outflows {
                if model.flows.contains_key(outflow) {
                    edge_id_counter.set(edge_id_counter.get() + 1);
                    edges.push(EdgeLayout {
                        id: format!("edge_{}", edge_id_counter.get()),
                        from: stock_name.clone(),
                        to: outflow.clone(),
                        edge_type: EdgeType::Outflow,
                    });
                }
            }
        }

        // Calculate bounding box and normalize positions
        let (actual_width, actual_height) = if nodes.is_empty() {
            (target_width, target_height)
        } else {
            // Find current bounds
            let min_x = nodes.iter().map(|n| n.x - n.width / 2.0).fold(f64::INFINITY, f64::min);
            let max_x = nodes.iter().map(|n| n.x + n.width / 2.0).fold(f64::NEG_INFINITY, f64::max);
            let min_y = nodes.iter().map(|n| n.y - n.height / 2.0).fold(f64::INFINITY, f64::min);
            let max_y = nodes.iter().map(|n| n.y + n.height / 2.0).fold(f64::NEG_INFINITY, f64::max);

            // Normalize positions to start from 50px margin
            let margin = 50.0;
            let offset_x = margin - min_x;
            let offset_y = margin - min_y;

            for node in &mut nodes {
                node.x += offset_x;
                node.y += offset_y;
            }

            let width = (max_x - min_x + 2.0 * margin).max(target_width);
            let height = (max_y - min_y + 2.0 * margin).max(target_height);

            (width, height)
        };

        LayoutResult {
            nodes,
            edges,
            width: actual_width,
            height: actual_height,
        }
    }

    /// Compute force-directed layout (placeholder for future enhancement)
    pub fn force_directed_layout(model: &Model) -> LayoutResult {
        // For now, just use hierarchical
        Self::hierarchical_layout(model)
    }

    /// Automatic layout selection based on model structure
    pub fn auto_layout(model: &Model) -> LayoutResult {
        // Simple heuristic: if mostly linear flow, use hierarchical
        // Otherwise could use force-directed
        Self::hierarchical_layout(model)
    }
}

impl NodeLayout {
    /// Get the center point of the node
    pub fn center(&self) -> (f64, f64) {
        (self.x, self.y)
    }
}
