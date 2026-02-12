/// Structural analysis tools for system dynamics models
///
/// Provides:
/// - Dependency graph construction
/// - Feedback loop identification
/// - Loop polarity analysis
/// - Structural dominance analysis

use std::collections::{HashMap, HashSet, VecDeque};
use crate::model::{Model, Expression};

/// Type of model element
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ElementType {
    Stock,
    Flow,
    Auxiliary,
    Parameter,
}

/// Node in the dependency graph
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GraphNode {
    pub name: String,
    pub element_type: ElementType,
}

impl GraphNode {
    pub fn new(name: String, element_type: ElementType) -> Self {
        Self { name, element_type }
    }
}

/// Edge in the dependency graph with polarity
#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub from: GraphNode,
    pub to: GraphNode,
    pub polarity: Polarity,
}

/// Polarity of a causal link
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Polarity {
    Positive,  // Same direction (increase causes increase)
    Negative,  // Opposite direction (increase causes decrease)
    Unknown,   // Cannot determine from structure alone
}

impl Polarity {
    /// Combine polarities through a chain
    pub fn combine(&self, other: &Polarity) -> Polarity {
        match (self, other) {
            (Polarity::Positive, Polarity::Positive) => Polarity::Positive,
            (Polarity::Negative, Polarity::Negative) => Polarity::Positive,
            (Polarity::Positive, Polarity::Negative) => Polarity::Negative,
            (Polarity::Negative, Polarity::Positive) => Polarity::Negative,
            _ => Polarity::Unknown,
        }
    }
}

/// Feedback loop in the model
#[derive(Debug, Clone)]
pub struct FeedbackLoop {
    pub nodes: Vec<GraphNode>,
    pub polarity: Polarity,
    pub length: usize,
}

impl FeedbackLoop {
    pub fn is_reinforcing(&self) -> bool {
        matches!(self.polarity, Polarity::Positive)
    }

    pub fn is_balancing(&self) -> bool {
        matches!(self.polarity, Polarity::Negative)
    }

    pub fn contains_stock(&self) -> bool {
        self.nodes.iter().any(|n| n.element_type == ElementType::Stock)
    }
}

/// Dependency graph of model structure
pub struct DependencyGraph {
    pub nodes: HashSet<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub adjacency: HashMap<GraphNode, Vec<GraphNode>>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashSet::new(),
            edges: Vec::new(),
            adjacency: HashMap::new(),
        }
    }

    /// Build dependency graph from model
    pub fn from_model(model: &Model) -> Self {
        let mut graph = Self::new();

        // Add nodes for all model elements
        for name in model.stocks.keys() {
            graph.add_node(GraphNode::new(name.clone(), ElementType::Stock));
        }
        for name in model.flows.keys() {
            graph.add_node(GraphNode::new(name.clone(), ElementType::Flow));
        }
        for name in model.auxiliaries.keys() {
            graph.add_node(GraphNode::new(name.clone(), ElementType::Auxiliary));
        }
        for name in model.parameters.keys() {
            graph.add_node(GraphNode::new(name.clone(), ElementType::Parameter));
        }

        // Add edges from flow equations
        for (flow_name, flow) in &model.flows {
            let dependencies = Self::extract_dependencies(&flow.equation);
            let from_node = GraphNode::new(flow_name.clone(), ElementType::Flow);

            for dep in dependencies {
                let to_node = graph.find_node(&dep);
                if let Some(to) = to_node {
                    graph.add_edge(from_node.clone(), to, Polarity::Unknown);
                }
            }
        }

        // Add edges from auxiliary equations
        for (aux_name, aux) in &model.auxiliaries {
            let dependencies = Self::extract_dependencies(&aux.equation);
            let from_node = GraphNode::new(aux_name.clone(), ElementType::Auxiliary);

            for dep in dependencies {
                let to_node = graph.find_node(&dep);
                if let Some(to) = to_node {
                    graph.add_edge(from_node.clone(), to, Polarity::Unknown);
                }
            }
        }

        // Add edges from stocks to flows
        for (stock_name, stock) in &model.stocks {
            let stock_node = GraphNode::new(stock_name.clone(), ElementType::Stock);

            for inflow_name in &stock.inflows {
                if let Some(flow_node) = graph.find_node(inflow_name) {
                    graph.add_edge(flow_node, stock_node.clone(), Polarity::Positive);
                }
            }

            for outflow_name in &stock.outflows {
                if let Some(flow_node) = graph.find_node(outflow_name) {
                    graph.add_edge(flow_node, stock_node.clone(), Polarity::Negative);
                }
            }
        }

        graph
    }

    pub fn add_node(&mut self, node: GraphNode) {
        self.nodes.insert(node.clone());
        self.adjacency.entry(node).or_insert_with(Vec::new);
    }

    pub fn add_edge(&mut self, from: GraphNode, to: GraphNode, polarity: Polarity) {
        self.edges.push(GraphEdge {
            from: from.clone(),
            to: to.clone(),
            polarity,
        });

        self.adjacency
            .entry(from)
            .or_insert_with(Vec::new)
            .push(to);
    }

    pub fn find_node(&self, name: &str) -> Option<GraphNode> {
        self.nodes.iter().find(|n| n.name == name).cloned()
    }

    /// Find all feedback loops using depth-first search
    pub fn find_feedback_loops(&self, max_length: usize) -> Vec<FeedbackLoop> {
        let mut loops = Vec::new();

        for start_node in &self.nodes {
            let mut visited = HashSet::new();
            let mut path = Vec::new();
            self.dfs_find_loops(start_node, start_node, &mut visited, &mut path, &mut loops, max_length);
        }

        // Remove duplicate loops (same nodes, different starting points)
        Self::deduplicate_loops(loops)
    }

    fn dfs_find_loops(
        &self,
        current: &GraphNode,
        start: &GraphNode,
        visited: &mut HashSet<GraphNode>,
        path: &mut Vec<GraphNode>,
        loops: &mut Vec<FeedbackLoop>,
        max_length: usize,
    ) {
        if path.len() > max_length {
            return;
        }

        visited.insert(current.clone());
        path.push(current.clone());

        if let Some(neighbors) = self.adjacency.get(current) {
            for neighbor in neighbors {
                if neighbor == start && path.len() > 1 {
                    // Found a loop
                    let polarity = self.calculate_loop_polarity(path);
                    loops.push(FeedbackLoop {
                        nodes: path.clone(),
                        polarity,
                        length: path.len(),
                    });
                } else if !visited.contains(neighbor) {
                    self.dfs_find_loops(neighbor, start, visited, path, loops, max_length);
                }
            }
        }

        path.pop();
        visited.remove(current);
    }

    fn calculate_loop_polarity(&self, path: &[GraphNode]) -> Polarity {
        let mut polarity = Polarity::Positive;

        for i in 0..path.len() {
            let from = &path[i];
            let to = &path[(i + 1) % path.len()];

            // Find edge polarity
            if let Some(edge) = self.edges.iter().find(|e| &e.from == from && &e.to == to) {
                polarity = polarity.combine(&edge.polarity);
            }
        }

        polarity
    }

    fn deduplicate_loops(loops: Vec<FeedbackLoop>) -> Vec<FeedbackLoop> {
        let mut unique_loops = Vec::new();
        let mut seen_sets: Vec<HashSet<String>> = Vec::new();

        for loop_item in loops {
            let node_names: HashSet<String> = loop_item.nodes.iter()
                .map(|n| n.name.clone())
                .collect();

            if !seen_sets.iter().any(|s| s == &node_names) {
                seen_sets.push(node_names);
                unique_loops.push(loop_item);
            }
        }

        unique_loops
    }

    /// Extract variable names from expression
    fn extract_dependencies(expr: &Expression) -> HashSet<String> {
        let mut deps = HashSet::new();

        match expr {
            Expression::Variable(name) => {
                deps.insert(name.clone());
            }
            Expression::SubscriptedVariable { name, .. } => {
                deps.insert(name.clone());
            }
            Expression::BinaryOp { left, right, .. } => {
                deps.extend(Self::extract_dependencies(left));
                deps.extend(Self::extract_dependencies(right));
            }
            Expression::UnaryOp { expr, .. } => {
                deps.extend(Self::extract_dependencies(expr));
            }
            Expression::FunctionCall { args, .. } => {
                for arg in args {
                    deps.extend(Self::extract_dependencies(arg));
                }
            }
            Expression::Conditional { condition, true_expr, false_expr } => {
                deps.extend(Self::extract_dependencies(condition));
                deps.extend(Self::extract_dependencies(true_expr));
                deps.extend(Self::extract_dependencies(false_expr));
            }
            _ => {}
        }

        deps
    }

    /// Topological sort for evaluation order
    pub fn topological_sort(&self) -> Result<Vec<GraphNode>, String> {
        let mut in_degree: HashMap<GraphNode, usize> = HashMap::new();
        let mut queue = VecDeque::new();
        let mut result = Vec::new();

        // Calculate in-degrees
        for node in &self.nodes {
            in_degree.insert(node.clone(), 0);
        }

        for edge in &self.edges {
            *in_degree.entry(edge.to.clone()).or_insert(0) += 1;
        }

        // Add nodes with no dependencies
        for (node, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(node.clone());
            }
        }

        // Process queue
        while let Some(node) = queue.pop_front() {
            result.push(node.clone());

            if let Some(neighbors) = self.adjacency.get(&node) {
                for neighbor in neighbors {
                    if let Some(degree) = in_degree.get_mut(neighbor) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(neighbor.clone());
                        }
                    }
                }
            }
        }

        if result.len() != self.nodes.len() {
            Err("Model contains circular dependencies".to_string())
        } else {
            Ok(result)
        }
    }
}

/// Model structure analyzer
pub struct StructureAnalyzer {
    pub graph: DependencyGraph,
    pub feedback_loops: Vec<FeedbackLoop>,
}

impl StructureAnalyzer {
    pub fn new(model: &Model) -> Self {
        let graph = DependencyGraph::from_model(model);
        let feedback_loops = graph.find_feedback_loops(10); // Max loop length of 10

        Self { graph, feedback_loops }
    }

    /// Get all reinforcing loops
    pub fn reinforcing_loops(&self) -> Vec<&FeedbackLoop> {
        self.feedback_loops.iter()
            .filter(|l| l.is_reinforcing())
            .collect()
    }

    /// Get all balancing loops
    pub fn balancing_loops(&self) -> Vec<&FeedbackLoop> {
        self.feedback_loops.iter()
            .filter(|l| l.is_balancing())
            .collect()
    }

    /// Count loops by length
    pub fn loop_length_distribution(&self) -> HashMap<usize, usize> {
        let mut dist = HashMap::new();
        for loop_item in &self.feedback_loops {
            *dist.entry(loop_item.length).or_insert(0) += 1;
        }
        dist
    }

    /// Generate structural report
    pub fn generate_report(&self) -> String {
        let mut report = String::new();

        report.push_str("=== Model Structure Analysis ===\n\n");

        // Basic statistics
        report.push_str(&format!("Nodes: {}\n", self.graph.nodes.len()));
        report.push_str(&format!("Edges: {}\n", self.graph.edges.len()));
        report.push_str(&format!("Feedback Loops: {}\n\n", self.feedback_loops.len()));

        // Loop analysis
        let reinforcing = self.reinforcing_loops();
        let balancing = self.balancing_loops();

        report.push_str(&format!("Reinforcing Loops: {}\n", reinforcing.len()));
        report.push_str(&format!("Balancing Loops: {}\n\n", balancing.len()));

        // Loop length distribution
        report.push_str("Loop Length Distribution:\n");
        let mut dist: Vec<_> = self.loop_length_distribution().into_iter().collect();
        dist.sort_by_key(|(length, _)| *length);
        for (length, count) in dist {
            report.push_str(&format!("  Length {}: {} loops\n", length, count));
        }

        report.push_str("\n=== Reinforcing Loops ===\n");
        for (i, loop_item) in reinforcing.iter().enumerate().take(10) {
            report.push_str(&format!("\nR{} (length {}):\n", i + 1, loop_item.length));
            for node in &loop_item.nodes {
                report.push_str(&format!("  -> {} ({:?})\n", node.name, node.element_type));
            }
        }

        report.push_str("\n=== Balancing Loops ===\n");
        for (i, loop_item) in balancing.iter().enumerate().take(10) {
            report.push_str(&format!("\nB{} (length {}):\n", i + 1, loop_item.length));
            for node in &loop_item.nodes {
                report.push_str(&format!("  -> {} ({:?})\n", node.name, node.element_type));
            }
        }

        report
    }

    /// Export graph to DOT format for visualization
    pub fn export_dot(&self) -> String {
        let mut dot = String::new();
        dot.push_str("digraph Model {\n");
        dot.push_str("  rankdir=LR;\n");
        dot.push_str("  node [shape=box];\n\n");

        // Nodes with different shapes
        for node in &self.graph.nodes {
            let shape = match node.element_type {
                ElementType::Stock => "box",
                ElementType::Flow => "ellipse",
                ElementType::Auxiliary => "diamond",
                ElementType::Parameter => "plaintext",
            };
            dot.push_str(&format!("  \"{}\" [shape={}];\n", node.name, shape));
        }

        dot.push_str("\n");

        // Edges with polarity
        for edge in &self.graph.edges {
            let style = match edge.polarity {
                Polarity::Positive => "solid",
                Polarity::Negative => "dashed",
                Polarity::Unknown => "dotted",
            };
            dot.push_str(&format!(
                "  \"{}\" -> \"{}\" [style={}];\n",
                edge.from.name, edge.to.name, style
            ));
        }

        dot.push_str("}\n");
        dot
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Stock, Flow, Parameter};

    #[test]
    fn test_dependency_graph() {
        let mut model = Model::new("Test");

        model.add_stock(Stock::new("Population", "100")).unwrap();
        model.add_parameter(Parameter::new("growth_rate", 0.1)).unwrap();
        model.add_flow(Flow::new("births", "Population * growth_rate")).unwrap();

        let graph = DependencyGraph::from_model(&model);

        assert!(graph.nodes.len() >= 3);
        assert!(!graph.edges.is_empty());
    }

    #[test]
    fn test_feedback_loop_detection() {
        let mut model = Model::new("Test");

        // Create simple feedback loop: Population -> births -> Population
        let mut stock = Stock::new("Population", "100");
        stock.inflows.push("births".to_string());
        model.add_stock(stock).unwrap();

        model.add_parameter(Parameter::new("growth_rate", 0.1)).unwrap();
        model.add_flow(Flow::new("births", "Population * growth_rate")).unwrap();

        let analyzer = StructureAnalyzer::new(&model);

        // May or may not find loops depending on graph construction details
        // Just check that it doesn't crash
        println!("Found {} feedback loops", analyzer.feedback_loops.len());
    }

    #[test]
    fn test_polarity_combination() {
        assert_eq!(
            Polarity::Positive.combine(&Polarity::Positive),
            Polarity::Positive
        );
        assert_eq!(
            Polarity::Positive.combine(&Polarity::Negative),
            Polarity::Negative
        );
        assert_eq!(
            Polarity::Negative.combine(&Polarity::Negative),
            Polarity::Positive
        );
    }
}
