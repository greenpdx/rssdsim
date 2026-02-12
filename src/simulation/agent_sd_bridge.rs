/// Improved Agent-SD integration with bidirectional coupling
///
/// Provides mechanisms for:
/// - Agents affecting SD variables
/// - SD variables affecting agents
/// - Agent creation/destruction from flows
/// - Spatial agent distribution

use std::collections::HashMap;
use crate::simulation::{AgentManager, AgentType, AgentState, AgentRule};
use crate::model::Expression;

/// Bridge configuration for agent-SD coupling
#[derive(Debug, Clone)]
pub struct AgentSDConfig {
    /// Map agent population names to their coupling rules
    pub agent_couplings: HashMap<String, AgentCoupling>,
}

impl AgentSDConfig {
    pub fn new() -> Self {
        Self {
            agent_couplings: HashMap::new(),
        }
    }

    pub fn add_coupling(&mut self, agent_type: String, coupling: AgentCoupling) {
        self.agent_couplings.insert(agent_type, coupling);
    }
}

/// Coupling rules for a specific agent type
#[derive(Debug, Clone)]
pub struct AgentCoupling {
    /// Agent attributes that feed into SD variables
    pub attributes_to_sd: Vec<AttributeMapping>,

    /// SD variables that affect agent attributes
    pub sd_to_attributes: Vec<SDMapping>,

    /// Flow that creates agents
    pub creation_flow: Option<String>,

    /// Flow that destroys agents
    pub destruction_flow: Option<String>,

    /// Rate at which agents are created per unit flow
    pub agents_per_flow_unit: f64,
}

impl AgentCoupling {
    pub fn new() -> Self {
        Self {
            attributes_to_sd: Vec::new(),
            sd_to_attributes: Vec::new(),
            creation_flow: None,
            destruction_flow: None,
            agents_per_flow_unit: 1.0,
        }
    }
}

/// Mapping from agent attribute to SD variable
#[derive(Debug, Clone)]
pub struct AttributeMapping {
    pub attribute_name: String,
    pub sd_variable: String,
    pub aggregation: AggregationType,
}

/// Types of aggregation for agent attributes
#[derive(Debug, Clone)]
pub enum AggregationType {
    Sum,
    Mean,
    Count,
    Max,
    Min,
    Median,
}

/// Mapping from SD variable to agent attribute
#[derive(Debug, Clone)]
pub struct SDMapping {
    pub sd_variable: String,
    pub attribute_name: String,
    pub mapping_type: MappingType,
}

/// How SD variable affects agent attribute
#[derive(Debug, Clone)]
pub enum MappingType {
    Direct,                    // attribute = sd_value
    Scaled(f64),               // attribute = sd_value * scale
    PerCapita,                 // attribute = sd_value / agent_count
    Conditional(f64),          // if attribute > threshold, apply effect
}

/// Agent-SD bridge for bidirectional coupling
pub struct AgentSDBridge {
    pub config: AgentSDConfig,
}

impl AgentSDBridge {
    pub fn new(config: AgentSDConfig) -> Self {
        Self { config }
    }

    /// Update agent attributes based on SD variables
    pub fn update_agents_from_sd(
        &self,
        agents: &mut AgentManager,
        sd_variables: &HashMap<String, f64>,
    ) {
        for (agent_type, coupling) in &self.config.agent_couplings {
            // Pre-calculate per-capita denominator
            let agent_count = agents.get_population(agent_type)
                .map(|p| p.count_active() as f64)
                .unwrap_or(0.0);

            if let Some(population) = agents.get_population_mut(agent_type) {
                for agent in population.all_agents_mut() {
                    for mapping in &coupling.sd_to_attributes {
                        if let Some(&sd_value) = sd_variables.get(&mapping.sd_variable) {
                            let new_value = match mapping.mapping_type {
                                MappingType::Direct => sd_value,
                                MappingType::Scaled(scale) => sd_value * scale,
                                MappingType::PerCapita => {
                                    if agent_count > 0.0 { sd_value / agent_count } else { 0.0 }
                                }
                                MappingType::Conditional(threshold) => {
                                    let current = agent.get(&mapping.attribute_name).unwrap_or(0.0);
                                    if current > threshold { sd_value } else { current }
                                }
                            };

                            agent.set(mapping.attribute_name.clone(), new_value);
                        }
                    }
                }
            }
        }
    }

    /// Calculate SD variables from agent attributes
    pub fn calculate_sd_from_agents(
        &self,
        agents: &AgentManager,
    ) -> HashMap<String, f64> {
        let mut sd_values = HashMap::new();

        for (agent_type, coupling) in &self.config.agent_couplings {
            if let Some(population) = agents.get_population(agent_type) {
                for mapping in &coupling.attributes_to_sd {
                    let value = match mapping.aggregation {
                        AggregationType::Sum => {
                            population.sum_attribute(&mapping.attribute_name)
                        }
                        AggregationType::Mean => {
                            population.mean_attribute(&mapping.attribute_name)
                        }
                        AggregationType::Count => {
                            population.count_active() as f64
                        }
                        AggregationType::Max => {
                            population.max_attribute(&mapping.attribute_name)
                        }
                        AggregationType::Min => {
                            population.min_attribute(&mapping.attribute_name)
                        }
                        AggregationType::Median => {
                            Self::calculate_median(population, &mapping.attribute_name)
                        }
                    };

                    sd_values.insert(mapping.sd_variable.clone(), value);
                }
            }
        }

        sd_values
    }

    /// Handle agent creation from flows
    pub fn process_agent_creation(
        &self,
        agents: &mut AgentManager,
        flow_values: &HashMap<String, f64>,
        dt: f64,
    ) -> Result<(), String> {
        for (agent_type, coupling) in &self.config.agent_couplings {
            if let Some(creation_flow) = &coupling.creation_flow {
                if let Some(&flow_rate) = flow_values.get(creation_flow) {
                    // Number of agents to create this timestep
                    let n_agents = (flow_rate * dt * coupling.agents_per_flow_unit).round() as usize;

                    if n_agents > 0 {
                        agents.create_agents(agent_type, n_agents)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Handle agent destruction from flows
    pub fn process_agent_destruction(
        &self,
        agents: &mut AgentManager,
        flow_values: &HashMap<String, f64>,
        dt: f64,
    ) -> Result<(), String> {
        for (agent_type, coupling) in &self.config.agent_couplings {
            if let Some(destruction_flow) = &coupling.destruction_flow {
                if let Some(&flow_rate) = flow_values.get(destruction_flow) {
                    // Number of agents to destroy this timestep
                    let n_agents = (flow_rate * dt * coupling.agents_per_flow_unit).round() as usize;

                    if n_agents > 0 {
                        // First collect agent IDs to remove
                        let agent_ids: Vec<_> = if let Some(population) = agents.get_population(agent_type) {
                            population.agents.keys().copied().collect()
                        } else {
                            Vec::new()
                        };

                        // Then remove them
                        if let Some(population) = agents.get_population_mut(agent_type) {
                            let mut removed = 0;
                            for id in agent_ids {
                                if removed >= n_agents {
                                    break;
                                }
                                population.remove_agent(id);
                                removed += 1;
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn calculate_median(population: &crate::simulation::abm::AgentPopulation, attribute: &str) -> f64 {
        let mut values: Vec<f64> = population
            .all_agents()
            .filter(|a| a.active)
            .filter_map(|a| a.get(attribute))
            .collect();

        if values.is_empty() {
            return 0.0;
        }

        values.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let len = values.len();
        if len % 2 == 0 {
            (values[len / 2 - 1] + values[len / 2]) / 2.0
        } else {
            values[len / 2]
        }
    }
}

/// Spatial distribution for agents
#[derive(Debug, Clone)]
pub struct SpatialDistribution {
    pub dimension: usize,  // 1D, 2D, or 3D
    pub bounds: Vec<(f64, f64)>,  // Min/max for each dimension
}

impl SpatialDistribution {
    pub fn new_1d(min: f64, max: f64) -> Self {
        Self {
            dimension: 1,
            bounds: vec![(min, max)],
        }
    }

    pub fn new_2d(x_range: (f64, f64), y_range: (f64, f64)) -> Self {
        Self {
            dimension: 2,
            bounds: vec![x_range, y_range],
        }
    }

    pub fn new_3d(x: (f64, f64), y: (f64, f64), z: (f64, f64)) -> Self {
        Self {
            dimension: 3,
            bounds: vec![x, y, z],
        }
    }

    /// Generate random position in space
    pub fn random_position(&self, rng: &mut impl rand::Rng) -> Vec<f64> {
        use rand::distributions::{Distribution, Standard};
        let mut result = Vec::new();
        for (min, max) in &self.bounds {
            let val: f64 = Standard.sample(rng);
            result.push(min + (max - min) * val);
        }
        result
    }
}

/// Enhanced agent with spatial awareness
#[derive(Debug, Clone)]
pub struct SpatialAgent {
    pub agent: AgentState,
    pub position: Vec<f64>,
    pub velocity: Vec<f64>,
}

impl SpatialAgent {
    pub fn new(agent: AgentState, position: Vec<f64>) -> Self {
        let dimension = position.len();
        Self {
            agent,
            position,
            velocity: vec![0.0; dimension],
        }
    }

    /// Calculate distance to another agent
    pub fn distance_to(&self, other: &SpatialAgent) -> f64 {
        self.position
            .iter()
            .zip(&other.position)
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f64>()
            .sqrt()
    }

    /// Move agent based on velocity
    pub fn update_position(&mut self, dt: f64, bounds: &[(f64, f64)]) {
        for i in 0..self.position.len() {
            self.position[i] += self.velocity[i] * dt;

            // Bounce off boundaries
            if self.position[i] < bounds[i].0 {
                self.position[i] = bounds[i].0;
                self.velocity[i] = -self.velocity[i];
            } else if self.position[i] > bounds[i].1 {
                self.position[i] = bounds[i].1;
                self.velocity[i] = -self.velocity[i];
            }
        }
    }
}

/// Network structure for agent interactions
#[derive(Debug, Clone)]
pub struct AgentNetwork {
    pub edges: HashMap<usize, Vec<usize>>,  // agent_id -> list of connected agent_ids
    pub edge_weights: HashMap<(usize, usize), f64>,
}

impl AgentNetwork {
    pub fn new() -> Self {
        Self {
            edges: HashMap::new(),
            edge_weights: HashMap::new(),
        }
    }

    pub fn add_edge(&mut self, from: usize, to: usize, weight: f64) {
        self.edges.entry(from).or_insert_with(Vec::new).push(to);
        self.edge_weights.insert((from, to), weight);
    }

    pub fn get_neighbors(&self, agent_id: usize) -> Vec<usize> {
        self.edges.get(&agent_id).cloned().unwrap_or_default()
    }

    pub fn get_edge_weight(&self, from: usize, to: usize) -> f64 {
        self.edge_weights.get(&(from, to)).copied().unwrap_or(0.0)
    }

    /// Build network from spatial proximity
    pub fn from_spatial_proximity(
        agents: &[SpatialAgent],
        threshold_distance: f64,
    ) -> Self {
        let mut network = Self::new();

        for i in 0..agents.len() {
            for j in (i + 1)..agents.len() {
                let distance = agents[i].distance_to(&agents[j]);
                if distance < threshold_distance {
                    let weight = 1.0 / (1.0 + distance); // Inverse distance weighting
                    network.add_edge(agents[i].agent.id, agents[j].agent.id, weight);
                    network.add_edge(agents[j].agent.id, agents[i].agent.id, weight);
                }
            }
        }

        network
    }

    /// Calculate network statistics
    pub fn average_degree(&self) -> f64 {
        if self.edges.is_empty() {
            return 0.0;
        }

        let total_edges: usize = self.edges.values().map(|v| v.len()).sum();
        total_edges as f64 / self.edges.len() as f64
    }

    pub fn clustering_coefficient(&self, agent_id: usize) -> f64 {
        let neighbors = self.get_neighbors(agent_id);
        let k = neighbors.len();

        if k < 2 {
            return 0.0;
        }

        let mut triangles = 0;
        for i in 0..neighbors.len() {
            for j in (i + 1)..neighbors.len() {
                if self.edges.get(&neighbors[i])
                    .map(|n| n.contains(&neighbors[j]))
                    .unwrap_or(false)
                {
                    triangles += 1;
                }
            }
        }

        2.0 * triangles as f64 / (k * (k - 1)) as f64
    }
}

impl Default for AgentSDConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for AgentCoupling {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for AgentNetwork {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spatial_distribution() {
        let dist = SpatialDistribution::new_2d((0.0, 100.0), (0.0, 100.0));
        let mut rng = rand::thread_rng();

        let pos = dist.random_position(&mut rng);
        assert_eq!(pos.len(), 2);
        assert!(pos[0] >= 0.0 && pos[0] <= 100.0);
        assert!(pos[1] >= 0.0 && pos[1] <= 100.0);
    }

    #[test]
    fn test_agent_network() {
        let mut network = AgentNetwork::new();

        network.add_edge(1, 2, 1.0);
        network.add_edge(2, 3, 1.0);
        network.add_edge(1, 3, 0.5);

        assert_eq!(network.get_neighbors(1).len(), 2);
        // Average degree: (2 + 1 + 0) / 3 = 1.0 (nodes without edges have 0 degree)
        // But we only have 2 nodes in the map (1 and 2)
        let avg = network.average_degree();
        assert!(avg > 0.0 && avg <= 2.0, "Average degree was {}", avg);
    }

    #[test]
    fn test_agent_sd_mapping() {
        let mut config = AgentSDConfig::new();
        let mut coupling = AgentCoupling::new();

        coupling.attributes_to_sd.push(AttributeMapping {
            attribute_name: "wealth".to_string(),
            sd_variable: "total_wealth".to_string(),
            aggregation: AggregationType::Sum,
        });

        config.add_coupling("Person".to_string(), coupling);

        assert_eq!(config.agent_couplings.len(), 1);
    }
}
