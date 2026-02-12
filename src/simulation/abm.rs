/// Agent-Based Modeling (ABM) framework
///
/// Provides infrastructure for hybrid system dynamics / agent-based models
/// where individual agents can have their own state and behavior rules.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Unique identifier for an agent
pub type AgentId = usize;

/// Agent state represented as key-value pairs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    pub id: AgentId,
    pub agent_type: String,
    pub attributes: HashMap<String, f64>,
    pub active: bool,
}

impl AgentState {
    pub fn new(id: AgentId, agent_type: String) -> Self {
        Self {
            id,
            agent_type,
            attributes: HashMap::new(),
            active: true,
        }
    }

    pub fn get(&self, key: &str) -> Option<f64> {
        self.attributes.get(key).copied()
    }

    pub fn set(&mut self, key: String, value: f64) {
        self.attributes.insert(key, value);
    }

    pub fn deactivate(&mut self) {
        self.active = false;
    }
}

/// Agent behavior rule
#[derive(Debug, Clone)]
pub enum AgentRule {
    /// Set attribute to expression result
    SetAttribute {
        attribute: String,
        expression: String,
    },
    /// Conditional rule: if condition then action
    Conditional {
        condition: String,
        then_rules: Vec<AgentRule>,
        else_rules: Vec<AgentRule>,
    },
    /// Die/remove agent
    Die,
}

/// Agent type definition
#[derive(Debug, Clone)]
pub struct AgentType {
    pub name: String,
    pub initial_attributes: HashMap<String, f64>,
    pub rules: Vec<AgentRule>,
}

impl AgentType {
    pub fn new(name: String) -> Self {
        Self {
            name,
            initial_attributes: HashMap::new(),
            rules: Vec::new(),
        }
    }

    pub fn add_attribute(&mut self, name: String, initial_value: f64) {
        self.initial_attributes.insert(name, initial_value);
    }

    pub fn add_rule(&mut self, rule: AgentRule) {
        self.rules.push(rule);
    }

    /// Create a new agent of this type
    pub fn create_agent(&self, id: AgentId) -> AgentState {
        let mut agent = AgentState::new(id, self.name.clone());
        agent.attributes = self.initial_attributes.clone();
        agent
    }
}

/// Population of agents of a specific type
#[derive(Debug, Clone)]
pub struct AgentPopulation {
    pub agent_type: String,
    pub agents: HashMap<AgentId, AgentState>,
    next_id: AgentId,
}

impl AgentPopulation {
    pub fn new(agent_type: String) -> Self {
        Self {
            agent_type,
            agents: HashMap::new(),
            next_id: 0,
        }
    }

    /// Create a new agent
    pub fn create_agent(&mut self, agent_type: &AgentType) -> AgentId {
        let id = self.next_id;
        self.next_id += 1;

        let agent = agent_type.create_agent(id);
        self.agents.insert(id, agent);
        id
    }

    /// Remove an agent
    pub fn remove_agent(&mut self, id: AgentId) {
        self.agents.remove(&id);
    }

    /// Get active agent count
    pub fn count_active(&self) -> usize {
        self.agents.values().filter(|a| a.active).count()
    }

    /// Get all agents
    pub fn all_agents(&self) -> impl Iterator<Item = &AgentState> {
        self.agents.values()
    }

    /// Get mutable access to all agents
    pub fn all_agents_mut(&mut self) -> impl Iterator<Item = &mut AgentState> {
        self.agents.values_mut()
    }

    /// Get agent by ID
    pub fn get_agent(&self, id: AgentId) -> Option<&AgentState> {
        self.agents.get(&id)
    }

    /// Get mutable agent by ID
    pub fn get_agent_mut(&mut self, id: AgentId) -> Option<&mut AgentState> {
        self.agents.get_mut(&id)
    }

    /// Calculate aggregate statistics
    pub fn sum_attribute(&self, attribute: &str) -> f64 {
        self.agents
            .values()
            .filter(|a| a.active)
            .filter_map(|a| a.get(attribute))
            .sum()
    }

    pub fn mean_attribute(&self, attribute: &str) -> f64 {
        let active_count = self.count_active();
        if active_count == 0 {
            return 0.0;
        }
        self.sum_attribute(attribute) / active_count as f64
    }

    pub fn max_attribute(&self, attribute: &str) -> f64 {
        self.agents
            .values()
            .filter(|a| a.active)
            .filter_map(|a| a.get(attribute))
            .fold(f64::NEG_INFINITY, f64::max)
    }

    pub fn min_attribute(&self, attribute: &str) -> f64 {
        self.agents
            .values()
            .filter(|a| a.active)
            .filter_map(|a| a.get(attribute))
            .fold(f64::INFINITY, f64::min)
    }
}

/// Manager for all agent populations in a simulation
#[derive(Debug, Clone)]
pub struct AgentManager {
    pub agent_types: HashMap<String, AgentType>,
    pub populations: HashMap<String, AgentPopulation>,
}

impl AgentManager {
    pub fn new() -> Self {
        Self {
            agent_types: HashMap::new(),
            populations: HashMap::new(),
        }
    }

    /// Register an agent type
    pub fn register_type(&mut self, agent_type: AgentType) {
        let name = agent_type.name.clone();
        self.agent_types.insert(name.clone(), agent_type);
        self.populations.insert(name.clone(), AgentPopulation::new(name));
    }

    /// Create agents of a given type
    pub fn create_agents(&mut self, type_name: &str, count: usize) -> Result<(), String> {
        let agent_type = self
            .agent_types
            .get(type_name)
            .ok_or_else(|| format!("Agent type '{}' not found", type_name))?
            .clone();

        let population = self
            .populations
            .get_mut(type_name)
            .ok_or_else(|| format!("Population for type '{}' not found", type_name))?;

        for _ in 0..count {
            population.create_agent(&agent_type);
        }

        Ok(())
    }

    /// Get population
    pub fn get_population(&self, type_name: &str) -> Option<&AgentPopulation> {
        self.populations.get(type_name)
    }

    /// Get mutable population
    pub fn get_population_mut(&mut self, type_name: &str) -> Option<&mut AgentPopulation> {
        self.populations.get_mut(type_name)
    }

    /// Update all agents based on their rules
    pub fn update_agents(&mut self, _dt: f64, _time: f64) -> Result<(), String> {
        // Simplified update - in practice you'd evaluate rules here
        // For now, this is a placeholder for agent rule evaluation

        for (type_name, population) in &mut self.populations {
            let agent_type = self.agent_types.get(type_name).unwrap();

            // For each agent, apply rules
            let mut agents_to_remove = Vec::new();

            for agent in population.all_agents_mut() {
                if !agent.active {
                    continue;
                }

                // Apply rules (simplified - would need full expression evaluation)
                for rule in &agent_type.rules {
                    match rule {
                        AgentRule::Die => {
                            agent.deactivate();
                            agents_to_remove.push(agent.id);
                        }
                        AgentRule::SetAttribute { .. } => {
                            // Would evaluate expression and set attribute
                            // Placeholder for now
                        }
                        AgentRule::Conditional { .. } => {
                            // Would evaluate condition and apply appropriate rules
                            // Placeholder for now
                        }
                    }
                }
            }

            // Remove dead agents
            for id in agents_to_remove {
                population.remove_agent(id);
            }
        }

        Ok(())
    }

    /// Get total count of active agents across all types
    pub fn total_agent_count(&self) -> usize {
        self.populations.values().map(|p| p.count_active()).sum()
    }

    /// Get count of agents of a specific type
    pub fn count_agents(&self, type_name: &str) -> usize {
        self.populations
            .get(type_name)
            .map(|p| p.count_active())
            .unwrap_or(0)
    }
}

impl Default for AgentManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_creation() {
        let mut agent_type = AgentType::new("Person".to_string());
        agent_type.add_attribute("age".to_string(), 25.0);
        agent_type.add_attribute("income".to_string(), 50000.0);

        let agent = agent_type.create_agent(0);
        assert_eq!(agent.get("age"), Some(25.0));
        assert_eq!(agent.get("income"), Some(50000.0));
    }

    #[test]
    fn test_population() {
        let mut agent_type = AgentType::new("Person".to_string());
        agent_type.add_attribute("wealth".to_string(), 100.0);

        let mut pop = AgentPopulation::new("Person".to_string());

        let id1 = pop.create_agent(&agent_type);
        let id2 = pop.create_agent(&agent_type);
        let id3 = pop.create_agent(&agent_type);

        assert_eq!(pop.count_active(), 3);

        // Modify agent wealth
        pop.get_agent_mut(id1).unwrap().set("wealth".to_string(), 150.0);
        pop.get_agent_mut(id2).unwrap().set("wealth".to_string(), 75.0);

        assert_eq!(pop.sum_attribute("wealth"), 325.0);
        assert_eq!(pop.mean_attribute("wealth"), 325.0 / 3.0);

        // Deactivate one
        pop.get_agent_mut(id3).unwrap().deactivate();
        assert_eq!(pop.count_active(), 2);
    }

    #[test]
    fn test_agent_manager() {
        let mut manager = AgentManager::new();

        let mut agent_type = AgentType::new("Person".to_string());
        agent_type.add_attribute("energy".to_string(), 100.0);

        manager.register_type(agent_type);
        manager.create_agents("Person", 10).unwrap();

        assert_eq!(manager.count_agents("Person"), 10);
        assert_eq!(manager.total_agent_count(), 10);
    }
}
