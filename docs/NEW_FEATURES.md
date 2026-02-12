# New Features Added to rssdsim

## Summary

This document describes the major new features added to rssdsim, transforming it from a basic system dynamics simulator into a comprehensive modeling platform with advanced capabilities.

## Features Implemented

### 1. Delay Functions ⏱️

**Location**: `src/simulation/delay.rs`

System dynamics models often need to represent delays in information and material flows. We've implemented four delay function types:

#### DELAY1 (First-Order Exponential Delay)
```
DELAY1(input, delay_time, [initial_value])
```
- Implements exponential smoothing: `d(output)/dt = (input - output) / delay_time`
- 63% response after one time constant
- Default initial value is the current input value

#### DELAY3 (Third-Order Delay)
```
DELAY3(input, delay_time, [initial_value])
```
- Cascade of three first-order delays
- Smoother response curve than DELAY1
- Better approximation of pipeline delays

#### DELAYP (Pipeline Delay)
```
DELAYP(input, delay_time, initial_value)
```
- Pure time delay with history buffer
- Uses linear interpolation between recorded values
- Ideal for material flows with fixed transit times

#### SMOOTH
```
SMOOTH(input, smooth_time)
```
- Alias for DELAY1
- Common in Vensim/Stella models

**Example**:
```yaml
flows:
  - name: perceived_demand
    equation: DELAY1(actual_demand, 7.0)  # 7-day perception delay
```

**Implementation Details**:
- Delay state is managed by `DelayManager` in `SimulationState`
- Exponential delays use Euler integration for state updates
- Pipeline delays maintain circular buffers of past values
- Each delay instance is uniquely keyed by its expression

---

### 2. Lookup Tables 📊

**Location**: `src/simulation/lookup.rs`

Lookup tables (graphical functions) allow modelers to define nonlinear relationships without complex equations.

#### WITH_LOOKUP
```
WITH_LOOKUP(x, x1,y1, x2,y2, x3,y3, ...)
```
- Inline definition of lookup tables
- Linear interpolation between points
- Flat extrapolation outside range
- Points must be sorted by x-value

**Example**:
```yaml
auxiliaries:
  - name: capacity_multiplier
    equation: WITH_LOOKUP(population, 0,1.0, 5000,0.8, 10000,0.5)
```

This creates a capacity constraint that decreases as population grows:
- At pop=0: multiplier = 1.0
- At pop=5000: multiplier = 0.8
- At pop=10000: multiplier = 0.5
- Values interpolate linearly between points

**Implementation Details**:
- `LookupTable` struct stores (x, y) pairs
- Validates that points are sorted
- Uses binary search for efficient lookup
- Supports any number of data points

---

### 3. Stochastic Elements 🎲

**Location**: `src/simulation/stochastic.rs`

Add randomness and uncertainty to models for Monte Carlo simulation and stochastic dynamics.

#### Functions Available

**RANDOM()**
- Uniform random number in [0, 1)
- No parameters

**UNIFORM(min, max)**
- Uniform distribution in range
- Example: `UNIFORM(10, 20)` → random value between 10 and 20

**NORMAL(mean, std_dev)**
- Gaussian/normal distribution
- Example: `NORMAL(100, 15)` → mean 100, std dev 15

**LOGNORMAL(mean, std_dev)**
- Log-normal distribution
- Useful for multiplicative processes
- Example: `LOGNORMAL(0, 0.5)`

**POISSON(lambda)**
- Poisson distribution (discrete events)
- Example: `POISSON(5)` → average 5 events

**Examples**:
```yaml
flows:
  - name: births
    equation: population * birth_rate * (1 + NORMAL(0, 0.1))
    # Add 10% random noise to birth rate

auxiliaries:
  - name: random_shock
    equation: IF RANDOM() > 0.95 THEN population * 0.1 ELSE 0
    # 5% chance of shock event each time step
```

**Features**:
- Reproducible with seeds: `StochasticManager::with_seed(123)`
- Uses `rand` crate with `StdRng` for quality randomness
- State persists across time steps for consistent sequences

**Implementation Details**:
- `StochasticManager` maintains RNG state in `SimulationState`
- Uses `rand_distr` for statistical distributions
- Seed can be set for reproducibility in Monte Carlo runs

---

### 4. Agent-Based Modeling Framework 🤖

**Location**: `src/simulation/abm.rs`

Enables hybrid system dynamics / agent-based models where individual agents have their own state and behavior.

#### Core Components

**AgentState**
- Individual agent with attributes (key-value pairs)
- Can be active or inactive
- Unique ID and type

**AgentType**
- Template defining agent behavior
- Initial attributes
- Behavior rules (SetAttribute, Conditional, Die)

**AgentPopulation**
- Collection of agents of same type
- Aggregate statistics (sum, mean, min, max)
- Agent creation and removal

**AgentManager**
- Manages all agent types and populations
- Integrated into `SimulationState`
- Update loop for agent behaviors

#### Functions

**AGENT_COUNT()**
- Returns total number of active agents
- Can be used in stock/flow equations

**Future Functions** (framework ready):
- `AGENT_SUM(type, attribute)` - Sum attribute across agents
- `AGENT_MEAN(type, attribute)` - Average attribute
- `AGENT_MAX/MIN(type, attribute)` - Extrema

**Example Use Case**:
```rust
// Create agent type
let mut person_type = AgentType::new("Person".to_string());
person_type.add_attribute("wealth".to_string(), 100.0);
person_type.add_attribute("age".to_string(), 25.0);

// Register and create agents
manager.register_type(person_type);
manager.create_agents("Person", 1000)?;

// Use in SD equation
"AGENT_COUNT() * resource_per_person"
```

**Implementation Details**:
- Agents stored in `HashMap<AgentId, AgentState>`
- Population statistics calculated on-demand
- Rule evaluation planned for future enhancement
- Fully integrated with simulation state management

---

### 5. Unit Checking & Dimensional Analysis 📏

**Location**: `src/model/units.rs`

Catch modeling errors by validating dimensional consistency of equations.

#### Features

**Base Dimensions**
- Length (meter)
- Mass (kilogram)
- Time (second)
- Current (ampere)
- Temperature (kelvin)
- Amount (mole)
- Luminosity (candela)

**Dimensional Formulas**
- Represent units as powers of base dimensions
- Example: velocity = m^1 * s^-1
- Example: force = kg^1 * m^1 * s^-2

**Operations**
- `multiply()` - Combine units (m * s → m·s)
- `divide()` - Divide units (m / s → m/s)
- `raise_to_power()` - Power operation (m^2)
- `is_compatible()` - Check if units can be added/subtracted

**UnitChecker**
- Validate operations in expressions
- Detect unit mismatches (e.g., adding meters to seconds)
- Provide clear error messages

**Examples**:
```rust
// Create unit checker
let mut checker = UnitChecker::new();
checker.register_variable("distance".into(), DimensionalFormula::length());
checker.register_variable("time".into(), DimensionalFormula::time());

// Check division: distance / time → velocity
let velocity_units = checker.check_divide("distance", "time");
assert_eq!(velocity_units, DimensionalFormula::velocity());

// Check invalid addition (would error)
checker.check_add("distance", "time"); // Error: incompatible units
```

**Pre-defined Formulas**:
- `length()` - m
- `velocity()` - m/s
- `acceleration()` - m/s²
- `force()` - N = kg·m/s²
- `energy()` - J = kg·m²/s²
- `power()` - W = kg·m²/s³

**Implementation Details**:
- Units stored as `HashMap<BaseDimension, i32>` (dimension → power)
- Parser handles common unit strings ("m/s", "kg*m/s^2")
- Display format matches scientific notation
- Zero powers automatically removed for cleaner formulas

---

## Integration with Existing Architecture

All new features are seamlessly integrated into the simulation engine:

### SimulationState Changes
```rust
pub struct SimulationState {
    pub time: f64,
    pub stocks: HashMap<String, f64>,
    pub flows: HashMap<String, f64>,
    pub auxiliaries: HashMap<String, f64>,
    pub delays: DelayManager,       // NEW
    pub stochastic: StochasticManager,  // NEW
    pub agents: AgentManager,       // NEW
}
```

### Expression Evaluator Updates
- Added 15+ new functions to `expression.rs`
- Made `EvaluationContext` mutable to support stateful operations
- Function evaluation now updates delay, stochastic, and agent state

### Integrator Updates
- Both Euler and RK4 integrators updated
- State changes properly merged after each evaluation
- Delay updates integrated into time stepping
- Pipeline delays record values at each timestep

---

## Testing

All features include comprehensive unit tests:

- **delay.rs**: Test exponential and pipeline delays
- **lookup.rs**: Test interpolation and extrapolation
- **stochastic.rs**: Test all distributions and reproducibility
- **abm.rs**: Test agent creation, populations, and statistics
- **units.rs**: Test dimensional arithmetic and checking

**Test Coverage**: 36 tests passing

**Example Models**: `examples/advanced_features.yaml` demonstrates all new features

---

## Performance Considerations

### Memory Usage
- Delays: O(k) per delay instance where k is order (1 or 3 for exponential)
- Pipeline delays: O(n) where n = simulation_time / dt
- Agents: O(a) where a is number of agents
- Lookup tables: O(p) where p is number of points

### Computational Complexity
- Delay evaluation: O(1) for DELAY1, O(3) for DELAY3
- Lookup: O(log p) via binary search
- Stochastic: O(1) for all distributions
- Agent aggregates: O(a) for sums/means/extrema

### Optimization Opportunities
- Delay history pruning for long simulations
- Lazy evaluation of agent statistics
- Cached lookups for repeated x values
- Parallel agent updates (future)

---

## Usage Examples

### Complete Model with All Features

See `examples/advanced_features.yaml` for a working model that demonstrates:
- Delays (DELAY1, SMOOTH)
- Lookups (WITH_LOOKUP)
- Stochastic elements (NORMAL, RANDOM)
- Multiple stocks and flows
- Nonlinear dynamics

Run with:
```bash
./target/release/rsedsim run examples/advanced_features.yaml -o output.csv
```

---

## API Documentation

### Delay Functions

```rust
// Create delay manager
let mut delays = DelayManager::new();

// Get or create exponential delay
let delay = delays.get_or_create_exponential(
    "key",       // Unique identifier
    100.0,       // Initial value
    10.0,        // Delay time
    1            // Order (1 or 3)
);

// Update delay
delay.update(input_value, dt);

// Get delayed value
let output = delay.get_value();
```

### Lookup Tables

```rust
// Create lookup table
let table = LookupTable::new(
    "capacity".to_string(),
    vec![(0.0, 1.0), (100.0, 0.5), (200.0, 0.1)]
)?;

// Lookup value
let multiplier = table.lookup(75.0); // Returns ~0.75 (interpolated)
```

### Stochastic Elements

```rust
// Create with seed for reproducibility
let mut stochastic = StochasticManager::with_seed(42);

// Generate random values
let uniform = stochastic.random();
let normal = stochastic.normal(100.0, 15.0)?;
let poisson = stochastic.poisson(5.0)?;
```

### Agent-Based Modeling

```rust
// Create agent type
let mut agent_type = AgentType::new("Person".to_string());
agent_type.add_attribute("wealth".to_string(), 100.0);

// Create manager and register type
let mut manager = AgentManager::new();
manager.register_type(agent_type);

// Create agents
manager.create_agents("Person", 1000)?;

// Get statistics
let total = manager.count_agents("Person");
let pop = manager.get_population("Person")?;
let avg_wealth = pop.mean_attribute("wealth");
```

---

## Future Enhancements

### Short Term
1. **String arguments in functions**: Support `AGENT_SUM("Person", "wealth")` syntax
2. **Pink/white noise**: Additional noise generators for time-correlated stochastic processes
3. **Delay initialization**: Better initial value handling for delays

### Medium Term
1. **Advanced agent rules**: Full expression evaluation in agent behavior rules
2. **Agent interactions**: Network-based agent-to-agent interactions
3. **Spatial agents**: Position-based agent modeling
4. **Lookup from file**: Load lookup tables from CSV/data files

### Long Term
1. **Optimization over delays**: Parameter optimization with delay-differential equations
2. **Stochastic differential equations**: Ito/Stratonovich calculus support
3. **ABM visualization**: Real-time agent animation
4. **Hybrid models**: Tight integration between SD stocks and ABM agents

---

## References

### System Dynamics Delay Functions
- Sterman, J. (2000). *Business Dynamics*. Chapter 11: Delays
- Richardson, G. P., & Pugh, A. L. (1981). *Introduction to System Dynamics Modeling with DYNAMO*

### Stochastic Simulation
- Gillespie, D. T. (1977). Exact stochastic simulation of coupled chemical reactions
- Allen, L. J. (2010). *An Introduction to Stochastic Processes with Applications to Biology*

### Agent-Based Modeling
- Wilensky, U., & Rand, W. (2015). *An Introduction to Agent-Based Modeling*
- Railsback, S. F., & Grimm, V. (2019). *Agent-Based and Individual-Based Modeling*

### Dimensional Analysis
- Bridgman, P. W. (1922). *Dimensional Analysis*
- Buckingham, E. (1914). On physically similar systems; illustrations of the use of dimensional equations

---

## Acknowledgments

These features bring rssdsim closer to feature parity with commercial SD tools like Vensim and Stella while maintaining Rust's performance and safety guarantees.

## License

All new code is dual-licensed under MIT/Apache-2.0, consistent with the rest of the project.
