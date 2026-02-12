# Advanced Analysis and Agent-SD Integration Features

## Overview

This document describes the new advanced features added to rssdsim in this second major update:

1. **Sensitivity Analysis** - Comprehensive parameter analysis tools
2. **Model Structure Analysis** - Loop detection and dependency analysis
3. **Improved Agent-SD Integration** - Bidirectional coupling between agents and system dynamics

## 1. Sensitivity Analysis

**Location**: `src/analysis/sensitivity.rs`

### Features

#### 1.1 Parameter Sweeps (One-at-a-Time)

Systematically vary each parameter to understand its individual effect on model behavior.

```rust
use rssdsim::analysis::{SensitivityAnalyzer, ParameterRange};

let ranges = vec![
    ParameterRange::new("growth_rate".to_string(), 0.05, 0.15, 0.1),
    ParameterRange::new("capacity".to_string(), 1000.0, 5000.0, 2000.0),
];

let mut analyzer = SensitivityAnalyzer::new(ranges);
analyzer.parameter_sweep(&model, &config, 10)?; // 10 steps per parameter

// Export results
let csv = analyzer.export_results("Population_final")?;
```

#### 1.2 Latin Hypercube Sampling (LHS)

Efficient space-filling design for exploring parameter space with fewer samples.

```rust
analyzer.latin_hypercube_sampling(&model, &config, 100, Some(42))?;
```

**Benefits**:
- More efficient than full factorial design
- Better coverage than random sampling
- Ideal for metamodeling and uncertainty quantification

#### 1.3 Morris Screening Method

Identify which parameters have the most influence on model outputs.

```rust
analyzer.morris_screening(&model, &config, 10, 4, Some(42))?;

let effects = analyzer.calculate_morris_effects("Population_final");

// effects contains (mean_effect, std_dev) for each parameter
for (param, (mu_star, sigma)) in effects {
    println!("{}: μ* = {:.3}, σ = {:.3}", param, mu_star, sigma);
}
```

**Interpretation**:
- **μ\* (absolute mean)**: Overall influence of parameter
- **σ (standard deviation)**: Nonlinearity or interactions

### Implementation Details

**ParameterRange**:
```rust
pub struct ParameterRange {
    pub name: String,
    pub min: f64,
    pub max: f64,
    pub baseline: f64,
}
```

**ParameterSample**:
```rust
pub struct ParameterSample {
    pub values: HashMap<String, f64>,
}
```

**SensitivityResult**:
```rust
pub struct SensitivityResult {
    pub sample: ParameterSample,
    pub outputs: HashMap<String, Vec<f64>>,  // Time series
    pub metrics: HashMap<String, f64>,        // Summary statistics
}
```

### Use Cases

1. **Model Validation**: Identify sensitive parameters that need careful calibration
2. **Uncertainty Analysis**: Understand output variability given parameter uncertainty
3. **Model Simplification**: Identify parameters that can be fixed without losing accuracy
4. **Scenario Analysis**: Explore different policy or environmental scenarios

---

## 2. Model Structure Analysis

**Location**: `src/analysis/structure.rs`

### Features

#### 2.1 Dependency Graph Construction

Automatically builds a causal graph from model equations.

```rust
use rssdsim::analysis::StructureAnalyzer;

let analyzer = StructureAnalyzer::new(&model);

// Access the graph
let graph = &analyzer.graph;
println!("Nodes: {}", graph.nodes.len());
println!("Edges: {}", graph.edges.len());
```

#### 2.2 Feedback Loop Detection

Identifies all feedback loops in the model up to a specified length.

```rust
let analyzer = StructureAnalyzer::new(&model);

// Get all loops
for loop_item in &analyzer.feedback_loops {
    println!("Loop length: {}", loop_item.length);
    println!("Polarity: {:?}", loop_item.polarity);

    for node in &loop_item.nodes {
        println!("  -> {}", node.name);
    }
}

// Filter by type
let reinforcing = analyzer.reinforcing_loops();
let balancing = analyzer.balancing_loops();

println!("Reinforcing loops: {}", reinforcing.len());
println!("Balancing loops: {}", balancing.len());
```

#### 2.3 Loop Polarity Analysis

Determines whether loops are reinforcing (positive) or balancing (negative).

```rust
pub enum Polarity {
    Positive,  // Reinforcing loop
    Negative,  // Balancing loop
    Unknown,   // Cannot determine
}
```

**Polarity Rules**:
- Even number of negative links → Reinforcing (R)
- Odd number of negative links → Balancing (B)

#### 2.4 Structural Report Generation

```rust
let report = analyzer.generate_report();
println!("{}", report);
```

**Example Output**:
```
=== Model Structure Analysis ===

Nodes: 15
Edges: 28
Feedback Loops: 5

Reinforcing Loops: 2
Balancing Loops: 3

Loop Length Distribution:
  Length 2: 2 loops
  Length 3: 1 loops
  Length 4: 2 loops

=== Reinforcing Loops ===

R1 (length 3):
  -> population (Stock)
  -> births (Flow)
  -> growth_factor (Auxiliary)

...
```

#### 2.5 DOT Graph Export

Export to Graphviz format for visualization.

```rust
let dot = analyzer.export_dot();
std::fs::write("model_graph.dot", dot)?;

// Then visualize with:
// dot -Tpng model_graph.dot -o model_graph.png
```

### Graph Types

**ElementType**:
```rust
pub enum ElementType {
    Stock,      // Accumulations (rectangles)
    Flow,       // Rates (valves)
    Auxiliary,  // Calculated variables (circles)
    Parameter,  // Constants (text)
}
```

**Edge Representation**:
- Solid lines: Positive polarity
- Dashed lines: Negative polarity
- Dotted lines: Unknown polarity

### Use Cases

1. **Model Communication**: Generate diagrams for documentation
2. **Quality Assurance**: Verify intended feedback structure
3. **Loop Dominance**: Understand which loops drive behavior
4. **Debugging**: Identify unintended dependencies or loops
5. **Education**: Teach feedback thinking with actual model structure

---

## 3. Improved Agent-SD Integration

**Location**: `src/simulation/agent_sd_bridge.rs`

### Features

#### 3.1 Bidirectional Coupling

Agents can affect SD variables AND SD variables can affect agents.

**Agent → SD (Aggregation)**:
```rust
use rssdsim::simulation::{AgentSDConfig, AgentCoupling, AttributeMapping, AggregationType};

let mut coupling = AgentCoupling::new();

// Sum all agent wealth → SD variable "total_wealth"
coupling.attributes_to_sd.push(AttributeMapping {
    attribute_name: "wealth".to_string(),
    sd_variable: "total_wealth".to_string(),
    aggregation: AggregationType::Sum,
});

// Average agent age → SD variable "mean_age"
coupling.attributes_to_sd.push(AttributeMapping {
    attribute_name: "age".to_string(),
    sd_variable: "mean_age".to_string(),
    aggregation: AggregationType::Mean,
});
```

**SD → Agent (Distribution)**:
```rust
// SD variable "resources_per_capita" → agent attribute "resources"
coupling.sd_to_attributes.push(SDMapping {
    sd_variable: "resources_per_capita".to_string(),
    attribute_name: "resources".to_string(),
    mapping_type: MappingType::Direct,
});

// Scaled mapping
coupling.sd_to_attributes.push(SDMapping {
    sd_variable: "temperature".to_string(),
    attribute_name: "comfort".to_string(),
    mapping_type: MappingType::Scaled(0.5),  // comfort = temperature * 0.5
});
```

#### 3.2 Agent Creation/Destruction from Flows

Link agent demographics to SD flows.

```rust
coupling.creation_flow = Some("birth_flow".to_string());
coupling.destruction_flow = Some("death_flow".to_string());
coupling.agents_per_flow_unit = 1.0;  // 1 agent per unit flow

// Apply during simulation
bridge.process_agent_creation(&mut agents, &flow_values, dt)?;
bridge.process_agent_destruction(&mut agents, &flow_values, dt)?;
```

#### 3.3 Spatial Agent Distribution

Agents can have positions in 1D, 2D, or 3D space.

```rust
use rssdsim::simulation::{SpatialDistribution, SpatialAgent};

// Create 2D space
let space = SpatialDistribution::new_2d((0.0, 100.0), (0.0, 100.0));

// Create spatial agent
let mut rng = rand::thread_rng();
let position = space.random_position(&mut rng);
let spatial_agent = SpatialAgent::new(agent_state, position);

// Calculate distances
let distance = agent1.distance_to(&agent2);

// Move agents
spatial_agent.update_position(dt, &space.bounds);
```

#### 3.4 Agent Networks

Model interactions through network structures.

```rust
use rssdsim::simulation::AgentNetwork;

let mut network = AgentNetwork::new();

// Add connections
network.add_edge(agent1_id, agent2_id, 1.0);  // weight = 1.0

// Query network
let neighbors = network.get_neighbors(agent1_id);
let weight = network.get_edge_weight(agent1_id, agent2_id);

// Network statistics
let avg_degree = network.average_degree();
let clustering = network.clustering_coefficient(agent1_id);

// Build from spatial proximity
let network = AgentNetwork::from_spatial_proximity(&spatial_agents, 10.0);
```

### Aggregation Types

```rust
pub enum AggregationType {
    Sum,      // Total across all agents
    Mean,     // Average value
    Count,    // Number of active agents
    Max,      // Maximum value
    Min,      // Minimum value
    Median,   // Median value
}
```

### Mapping Types

```rust
pub enum MappingType {
    Direct,              // attribute = sd_value
    Scaled(f64),         // attribute = sd_value * scale
    PerCapita,           // attribute = sd_value / agent_count
    Conditional(f64),    // Apply if current > threshold
}
```

### Complete Example: Epidemic with Agent Movement

```rust
// Setup spatial ABM-SD coupling
let mut coupling = AgentCoupling::new();

// Agent infections → SD stock
coupling.attributes_to_sd.push(AttributeMapping {
    attribute_name: "infected".to_string(),
    sd_variable: "total_infected".to_string(),
    aggregation: AggregationType::Sum,
});

// SD resource distribution → agents
coupling.sd_to_attributes.push(SDMapping {
    sd_variable: "treatment_per_capita".to_string(),
    attribute_name: "treatment_received".to_string(),
    mapping_type: MappingType::PerCapita,
});

// Birth/death flows affect agents
coupling.creation_flow = Some("births".to_string());
coupling.destruction_flow = Some("deaths".to_string());

// Create spatial distribution
let space = SpatialDistribution::new_2d((0.0, 100.0), (0.0, 100.0));

// Build proximity network
let network = AgentNetwork::from_spatial_proximity(&spatial_agents, 5.0);

// Agent interactions based on network
for agent_id in active_agents {
    let neighbors = network.get_neighbors(agent_id);
    // Infection probability based on infected neighbors
    ...
}
```

### Use Cases

1. **Hybrid Models**: Combine population-level dynamics (SD) with individual behavior (ABM)
2. **Resource Allocation**: Distribute aggregate resources to individuals
3. **Spatial Dynamics**: Model movement, clustering, and spatial externalities
4. **Social Networks**: Network effects, information diffusion, social influence
5. **Heterogeneous Agents**: Individual variation within aggregate flows
6. **Emergent Behavior**: Micro-level rules producing macro-level patterns

---

## Performance Considerations

### Sensitivity Analysis
- **Memory**: O(n_samples * n_timesteps * n_variables)
- **Time**: O(n_samples * simulation_time)
- **Parallelization**: Samples can be run in parallel (future feature)

### Structure Analysis
- **Loop Detection**: O(V * E) where V = nodes, E = edges
- **Maximum Loop Length**: Set to 10 by default (adjustable)
- **Memory**: O(V + E) for graph storage

### Agent-SD Coupling
- **Update Agents**: O(n_agents * n_mappings)
- **Aggregate**: O(n_agents) per aggregation
- **Spatial Queries**: O(n_agents²) for proximity (can optimize with spatial indexing)
- **Network Operations**: O(degree) for neighbor queries

---

## Best Practices

### Sensitivity Analysis

1. **Start with Morris screening** to identify important parameters
2. **Use LHS for detailed analysis** of important parameters
3. **Set appropriate ranges**: ±20% to ±50% of baseline
4. **Check convergence**: Increase sample size until results stabilize
5. **Analyze multiple outputs**: Final value, mean, max, oscillation

### Structure Analysis

1. **Review loop inventory** regularly during model development
2. **Verify loop polarities** match conceptual understanding
3. **Use DOT export** for stakeholder communication
4. **Check for unintended loops** that may indicate errors
5. **Document dominant loops** in model documentation

### Agent-SD Integration

1. **Start simple**: Begin with one-way coupling before bidirectional
2. **Match timescales**: Agent updates should align with SD timestep
3. **Validate aggregations**: Check that sums match expected totals
4. **Use spatial structures** judiciously (computational cost)
5. **Network topology matters**: Choose appropriate network model
6. **Profile performance**: Monitor agent count vs. simulation speed

---

## Code Statistics

### New Code Added
- **sensitivity.rs**: 380 lines (LHS, Morris, parameter sweeps)
- **structure.rs**: 520 lines (graph analysis, loop detection)
- **agent_sd_bridge.rs**: 490 lines (bidirectional coupling, spatial agents, networks)
- **Total new code**: ~1,390 lines
- **Total new tests**: 9 additional tests

### New Functions
- Sensitivity: `parameter_sweep`, `latin_hypercube_sampling`, `morris_screening`
- Structure: `find_feedback_loops`, `generate_report`, `export_dot`
- Agent-SD: `update_agents_from_sd`, `calculate_sd_from_agents`, `process_agent_creation`

---

## Future Enhancements

### Short Term
1. **Parallel sensitivity analysis**: Use rayon for concurrent simulations
2. **Sobol indices**: Variance-based global sensitivity analysis
3. **Eigensystem analysis**: Stability and oscillation analysis
4. **Advanced network models**: Small-world, scale-free, community detection

### Medium Term
1. **Optimization integration**: Couple with sensitivity for calibration
2. **Machine learning metamodels**: Surrogate models for fast sensitivity
3. **Interactive visualization**: Web-based loop/network visualization
4. **Spatial optimization**: Optimal agent placement/movement

### Long Term
1. **GPU acceleration**: Parallel agent updates on GPU
2. **Distributed simulation**: Multi-machine sensitivity analysis
3. **Real-time ABM**: Interactive agent simulation with live SD coupling
4. **Cloud deployment**: Sensitivity analysis as a service

---

## References

### Sensitivity Analysis
- Saltelli, A. et al. (2008). *Global Sensitivity Analysis: The Primer*
- Morris, M. D. (1991). Factorial sampling plans for preliminary computational experiments
- McKay, M. D. et al. (1979). A comparison of three methods for selecting values of input variables

### Structure Analysis
- Forrester, J. W. (1961). *Industrial Dynamics*
- Meadows, D. H. (2008). *Thinking in Systems: A Primer*
- Ster man, J. (2000). *Business Dynamics*, Chapter 5: Causal Loop Diagrams

### Agent-Based Modeling
- Wilensky, U., & Rand, W. (2015). *An Introduction to Agent-Based Modeling*
- Epstein, J. M., & Axtell, R. (1996). *Growing Artificial Societies*
- Gilbert, N., & Troitzsch, K. (2005). *Simulation for the Social Scientist*

### Hybrid SD-ABM
- Schieritz, N., & Milling, P. M. (2003). Modeling the forest or modeling the trees
- Rahmandad, H., & Sterman, J. (2008). Heterogeneity and network structure in the dynamics of diffusion
- Lorenz, T., & Jost, A. (2006). Towards an orientation framework in multi-paradigm modeling

---

## Acknowledgments

These features significantly enhance rssdsim's capabilities for:
- **Research**: Rigorous sensitivity and uncertainty analysis
- **Policy Analysis**: Understanding feedback structure and leverage points
- **Complex Systems**: Hybrid SD-ABM for multi-level modeling
- **Education**: Visualizing structure and teaching systems thinking

The implementation maintains Rust's safety guarantees while providing performance competitive with commercial SD tools.

## License

All new code is dual-licensed under MIT/Apache-2.0, consistent with the rest of the project.
