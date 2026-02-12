/// Graph construction from system dynamics models

use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;
use crate::model::Model;

/// Node types in the dependency graph
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphNodeType {
    Stock,
    Flow,
    Auxiliary,
    Parameter,
}

/// Node data in the dependency graph
#[derive(Debug, Clone)]
pub struct GraphNode {
    pub name: String,
    pub node_type: GraphNodeType,
}

/// Edge types in the dependency graph
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphEdgeType {
    Inflow,
    Outflow,
    Dependency,
}

/// Dependency graph representation
pub struct DependencyGraph {
    pub graph: DiGraph<GraphNode, GraphEdgeType>,
    pub node_map: HashMap<String, NodeIndex>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            node_map: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, name: String, node_type: GraphNodeType) -> NodeIndex {
        let node = GraphNode {
            name: name.clone(),
            node_type,
        };
        let idx = self.graph.add_node(node);
        self.node_map.insert(name, idx);
        idx
    }

    pub fn add_edge(&mut self, from: &str, to: &str, edge_type: GraphEdgeType) {
        if let (Some(&from_idx), Some(&to_idx)) = (self.node_map.get(from), self.node_map.get(to)) {
            self.graph.add_edge(from_idx, to_idx, edge_type);
        }
    }

    pub fn get_node_index(&self, name: &str) -> Option<NodeIndex> {
        self.node_map.get(name).copied()
    }
}

/// Build dependency graph from a model
pub fn build_graph_from_model(model: &Model) -> DependencyGraph {
    let mut graph = DependencyGraph::new();

    // Add all nodes
    for name in model.stocks.keys() {
        graph.add_node(name.clone(), GraphNodeType::Stock);
    }

    for name in model.flows.keys() {
        graph.add_node(name.clone(), GraphNodeType::Flow);
    }

    for name in model.auxiliaries.keys() {
        graph.add_node(name.clone(), GraphNodeType::Auxiliary);
    }

    for name in model.parameters.keys() {
        graph.add_node(name.clone(), GraphNodeType::Parameter);
    }

    // Add stock-flow edges
    for (stock_name, stock) in &model.stocks {
        for inflow in &stock.inflows {
            if model.flows.contains_key(inflow) {
                graph.add_edge(inflow, stock_name, GraphEdgeType::Inflow);
            }
        }

        for outflow in &stock.outflows {
            if model.flows.contains_key(outflow) {
                graph.add_edge(stock_name, outflow, GraphEdgeType::Outflow);
            }
        }
    }

    // Add dependency edges from equations
    // This is a simplified version - a full implementation would parse expressions
    // For now, we'll skip complex dependency analysis

    graph
}
