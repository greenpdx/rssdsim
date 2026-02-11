# API Documentation

Complete reference for the rsedsim Rust API.

## Table of Contents

1. [Model API](#model-api)
2. [Simulation API](#simulation-api)
3. [Agent API](#agent-api)
4. [Array Operations](#array-operations)
5. [Functions](#functions)
6. [I/O API](#io-api)
7. [Protocol API](#protocol-api)

---

## Model API

### Model Structure

```rust
use rsedsim::model::{Model, Stock, Flow, Auxiliary, Parameter};

// Create a new model
let mut model = Model::new("SIR Model");

// Configure time
model.set_time_config(TimeConfig {
    start: 0.0,
    stop: 100.0,
    dt: 0.25,
    units: Some("days".to_string()),
});
```

### Stocks

```rust
use rsedsim::model::Stock;

// Create a stock
let susceptible = Stock::builder()
    .name("Susceptible")
    .initial_value("990")  // Can be expression
    .units("people")
    .outflows(vec!["infection_rate"])
    .build()?;

model.add_stock(susceptible)?;

// Multi-dimensional stock
let population = Stock::builder()
    .name("Population")
    .initial_value("initial_population[region, age_group]")
    .dimensions(vec!["region", "age_group"])
    .build()?;
```

### Flows

```rust
use rsedsim::model::Flow;

let infection_rate = Flow::builder()
    .name("infection_rate")
    .equation("contact_rate * infectivity * Susceptible * Infected / total_population")
    .units("people/day")
    .build()?;

model.add_flow(infection_rate)?;
```

### Auxiliaries

```rust
use rsedsim::model::Auxiliary;

let total_population = Auxiliary::builder()
    .name("total_population")
    .equation("Susceptible + Infected + Recovered")
    .units("people")
    .build()?;

model.add_auxiliary(total_population)?;
```

### Parameters

```rust
use rsedsim::model::Parameter;

let contact_rate = Parameter::builder()
    .name("contact_rate")
    .value(5.0)
    .units("contacts/person/day")
    .description("Average number of contacts per person per day")
    .build()?;

model.add_parameter(contact_rate)?;
```

### Dimensions (Subscripts)

```rust
use rsedsim::array::Dimension;

// Create a dimension
let region = Dimension::builder()
    .name("region")
    .elements(vec!["North", "South", "East", "West"])
    .build()?;

model.add_dimension(region)?;

// Create a subset
model.add_subset(
    "region",
    "coastal",
    vec!["North", "East"]  // Subset of region
)?;
```

### Equations

```rust
use rsedsim::model::Expression;

// Parse equation from string
let expr = Expression::parse("contact_rate * infectivity * Susceptible")?;

// Build expression programmatically
let expr = Expression::multiply(
    Expression::variable("contact_rate"),
    Expression::multiply(
        Expression::variable("infectivity"),
        Expression::variable("Susceptible")
    )
);

// Evaluate expression
let context = EvaluationContext::new(&model, &state);
let value = expr.evaluate(&context)?;
```

### Model Validation

```rust
// Validate model before running
let validation = model.validate()?;

if !validation.is_valid() {
    for error in validation.errors() {
        eprintln!("Error: {}", error);
    }
}

// Check units
if validation.has_units_errors() {
    for error in validation.units_errors() {
        eprintln!("Units error: {}", error);
    }
}

// Check for circular dependencies
if let Some(cycle) = model.find_circular_dependency() {
    eprintln!("Circular dependency: {:?}", cycle);
}
```

---

## Simulation API

### Basic Simulation

```rust
use rsedsim::simulation::{SimulationEngine, SimulationConfig, IntegrationMethod};

// Create simulation
let config = SimulationConfig::builder()
    .integration_method(IntegrationMethod::RK4)
    .output_interval(1.0)  // Save every 1 time unit
    .build();

let mut sim = SimulationEngine::new(model, config)?;

// Run simulation
let results = sim.run()?;

// Access results
for (time, state) in results.iter() {
    println!("Time: {}", time);
    println!("  Susceptible: {}", state.get_stock("Susceptible")?);
    println!("  Infected: {}", state.get_stock("Infected")?);
}
```

### Step-by-Step Execution

```rust
// Initialize simulation
sim.initialize()?;

// Step manually
while sim.time() < sim.config().stop_time {
    sim.step()?;

    // Access current state
    let infected = sim.state().get_stock("Infected")?;
    println!("t={}: Infected = {}", sim.time(), infected);

    // Modify parameters dynamically
    if sim.time() > 50.0 {
        sim.set_parameter("contact_rate", 3.0)?;
    }
}
```

### Integration Methods

```rust
use rsedsim::simulation::integrator::*;

// Euler (simple, fast, less accurate)
let integrator = Box::new(EulerIntegrator);

// Runge-Kutta 4th order (balanced)
let integrator = Box::new(RK4Integrator);

// Adaptive RK45 (variable step size)
let integrator = Box::new(RK45Integrator::new()
    .with_tolerance(1e-6)
    .with_min_step(0.001)
    .with_max_step(1.0)
);

// Backward Euler (for stiff systems)
let integrator = Box::new(BackwardEulerIntegrator::new()
    .with_max_iterations(100)
    .with_tolerance(1e-6)
);

sim.set_integrator(integrator);
```

### Sensitivity Analysis

```rust
use rsedsim::analysis::SensitivityAnalysis;

let sensitivity = SensitivityAnalysis::new(model)
    .parameter("contact_rate", 1.0..=10.0, 20)  // 20 samples
    .parameter("infectivity", 0.1..=0.5, 20)
    .output_variable("Infected")
    .method(SamplingMethod::LatinHypercube)
    .run()?;

// Get sensitivity indices
println!("Sensitivity indices:");
for (param, indices) in sensitivity.sobol_indices() {
    println!("  {}: S1={:.3}, ST={:.3}", param, indices.first_order, indices.total);
}
```

### Monte Carlo Simulation

```rust
use rsedsim::analysis::MonteCarlo;

let mc = MonteCarlo::new(model)
    .parameter_distribution("contact_rate", Distribution::Normal { mean: 5.0, std: 1.0 })
    .parameter_distribution("infectivity", Distribution::Uniform { min: 0.2, max: 0.3 })
    .runs(1000)
    .parallel(true)  // Use all CPU cores
    .run()?;

// Analyze results
let stats = mc.statistics("Infected");
println!("Peak infected:");
println!("  Mean: {:.1}", stats.mean);
println!("  Std: {:.1}", stats.std);
println!("  95% CI: [{:.1}, {:.1}]", stats.ci_lower(0.95), stats.ci_upper(0.95));
```

### Optimization

```rust
use rsedsim::optimization::Optimizer;

// Find parameters that minimize peak infections
let optimizer = Optimizer::new(model)
    .objective(|results| {
        results.max_value("Infected").unwrap()
    })
    .minimize()
    .parameter("contact_rate", 1.0..=10.0)
    .parameter("infectivity", 0.1..=0.5)
    .method(OptimizationMethod::NelderMead)
    .max_iterations(1000)
    .run()?;

println!("Optimal parameters:");
println!("  contact_rate: {:.2}", optimizer.best_parameters()["contact_rate"]);
println!("  infectivity: {:.2}", optimizer.best_parameters()["infectivity"]);
println!("  Peak infected: {:.1}", optimizer.best_objective());
```

---

## Agent API

### Agent Definition

```rust
use rsedsim::agent::{Agent, AgentBehavior};

struct PersonAgent {
    infected: bool,
    location: (f64, f64),
    contacts: Vec<AgentId>,
}

impl AgentBehavior for PersonAgent {
    fn step(&mut self, agent: &mut Agent, context: &SimulationContext) -> Result<(), AgentError> {
        // Read SD variables
        let infectivity = context.get_sd_variable("infectivity")?;

        // Agent logic
        if !self.infected {
            let infected_contacts = self.contacts.iter()
                .filter(|id| context.get_agent(*id).unwrap().get("infected") == true)
                .count();

            let infection_prob = infectivity * (infected_contacts as f64) / self.contacts.len() as f64;

            if context.random() < infection_prob {
                self.infected = true;
            }
        }

        Ok(())
    }
}
```

### Agent Population

```rust
use rsedsim::agent::AgentPopulation;

let population = AgentPopulation::builder()
    .name("People")
    .count(10000)
    .agent_factory(|| {
        Agent::new(PersonAgent {
            infected: false,
            location: random_location(),
            contacts: vec![],
        })
    })
    .sd_inputs(vec!["infectivity", "contact_rate"])
    .sd_output("total_infected", AggregationFn::count(|agent| agent.get("infected")))
    .build()?;
```

### Hybrid Model

```rust
use rsedsim::agent::HybridModel;

let hybrid = HybridModel::builder()
    .sd_model(sd_model)
    .add_population(population)
    .coupling_order(CouplingOrder::SDFirst)  // SD updates before agents
    .build()?;

// Run hybrid simulation
let results = hybrid.run(config)?;
```

---

## Array Operations

### Multi-dimensional Variables

```rust
use rsedsim::array::{Value, ArrayOps};
use ndarray::Array2;

// Create 2D array
let population = Array2::<f64>::zeros((4, 3));  // 4 regions x 3 age groups

// Set values
population[[0, 0]] = 10000.0;  // North, Young
population[[0, 1]] = 25000.0;  // North, Adult

// Wrap in Value enum
let value = Value::Matrix(population);

// Operations
let total = value.sum();  // Sum all elements
let by_region = value.sum_over(1);  // Sum over age_group dimension
let by_age = value.sum_over(0);  // Sum over region dimension
```

### Subscript Access

```rust
// Access by subscript name
let north_young = model.get_subscripted_value("Population", &["North", "Young"])?;

// Access subset
let coastal_population = model.get_subscripted_value("Population", &["coastal", "*"])?;
// Returns array for all age groups in coastal regions (North and East)
```

### Vector Operations

```rust
use rsedsim::array::VectorOps;

// Element-wise operations
let birth_rate = population.element_wise_mul(&birth_rate_fraction);
let aging_rate = population.element_wise_div(&lifespan);

// Aggregations
let total_population = population.sum();
let average_age = population.weighted_mean(&age_values);

// Filtering
let senior_population = population.select_dimension("age_group", &["Senior"]);
```

---

## Functions

### Built-in Functions

```rust
use rsedsim::functions::*;

// Time functions
let current_time = TIME();
let dt = TIME_STEP();
let initial_value = INITIAL(variable);

// Delay functions
let delayed = DELAY1(input, delay_time);
let smooth = SMOOTH(input, averaging_time);
let pipeline_delay = DELAY_FIXED(input, delay_time);

// Lookup/table functions
let value = TABLE(x, vec![(0.0, 0.0), (10.0, 100.0), (20.0, 150.0)]);
let value = WITH_LOOKUP(x, "infection_rate_table");

// Statistical functions
let random = RANDOM_UNIFORM(0.0, 1.0);
let normal = RANDOM_NORMAL(5.0, 1.0);  // mean, std
let poisson = RANDOM_POISSON(3.5);  // lambda

// Logical functions
let result = IF_THEN_ELSE(condition, true_value, false_value);

// Math functions
let minimum = MIN(a, b, c);
let maximum = MAX(vec![a, b, c, d]);

// Input functions
let step = STEP(height, step_time);
let ramp = RAMP(slope, start_time, end_time);
let pulse = PULSE(start, width);
let pulse_train = PULSE_TRAIN(start, width, interval);

// Array functions
let total = SUM(array);
let average = MEAN(array);
let min_val = VMIN(array);
let max_val = VMAX(array);
```

### Custom Functions

```rust
use rsedsim::functions::{Function, FunctionSignature, FunctionError};

struct MyCustomFunction;

impl Function for MyCustomFunction {
    fn name(&self) -> &str {
        "MY_FUNC"
    }

    fn call(&self, args: &[Value], context: &EvalContext) -> Result<Value, FunctionError> {
        if args.len() != 2 {
            return Err(FunctionError::InvalidArgumentCount {
                expected: 2,
                got: args.len(),
            });
        }

        let a = args[0].as_scalar()?;
        let b = args[1].as_scalar()?;

        Ok(Value::Scalar(a * b + 10.0))
    }

    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            min_args: 2,
            max_args: Some(2),
            return_type: ValueType::Scalar,
        }
    }
}

// Register custom function
model.register_function(Box::new(MyCustomFunction))?;
```

---

## I/O API

### Model Loading

```rust
use rsedsim::io::{ModelParser, JsonParser, YamlParser, XmileParser};

// JSON
let parser = JsonParser::new();
let model = parser.parse_file("model.json")?;

// YAML
let parser = YamlParser::new();
let model = parser.parse_file("model.yaml")?;

// XMILE (Stella/Vensim)
let parser = XmileParser::new();
let model = parser.parse_file("model.stmx")?;

// Auto-detect format
let model = Model::load("model.json")?;  // Detects format from extension
```

### Results Export

```rust
use rsedsim::io::{ResultWriter, CsvWriter, JsonWriter, Hdf5Writer};

// CSV
let writer = CsvWriter::new();
writer.write_file(&results, "results.csv")?;

// JSON
let writer = JsonWriter::new()
    .pretty(true)
    .include_metadata(true);
writer.write_file(&results, "results.json")?;

// HDF5 (for large datasets)
let writer = Hdf5Writer::new()
    .compression(9);
writer.write_file(&results, "results.h5")?;
```

### Streaming Output

```rust
use rsedsim::io::StreamWriter;

// Write results as simulation runs (for long simulations)
let stream = StreamWriter::csv("results.csv")?;

sim.on_step(|state| {
    stream.write_state(state)?;
});

sim.run()?;
stream.close()?;
```

---

## Protocol API

### MCP Server

```rust
use rsedsim::protocol::mcp::{McpServer, McpMessage};

// Create MCP server
let mut server = McpServer::new();

// Start server on stdio
tokio::spawn(async move {
    server.serve_stdio().await.unwrap();
});

// Or HTTP
tokio::spawn(async move {
    server.serve_http("localhost:3000").await.unwrap();
});
```

### MCP Client

```rust
use rsedsim::protocol::mcp::McpClient;

let mut client = McpClient::new();
client.connect_stdio().await?;

// Call a tool
let response = client.call_tool("run_simulation", json!({
    "model": "sir_model.json",
    "parameters": {
        "contact_rate": 10
    }
})).await?;
```

### A2A Node

```rust
use rsedsim::protocol::a2a::{A2aNode, AgentId, A2aPayload};

// Create node
let agent_id = AgentId::new("sim1", "agent_1");
let mut node = A2aNode::new(agent_id);

// Configure transport
let transport = UdpTransport::bind("0.0.0.0:5000").await?;
node.set_transport(Box::new(transport));

// Register message handler
node.register_handler("direct_message", |msg| {
    println!("Received: {:?}", msg);
});

// Send message
node.send(
    AgentId::new("sim1", "agent_2"),
    A2aPayload::DirectMessage {
        content: json!({"hello": "world"})
    }
).await?;

// Run node (starts event loop)
tokio::spawn(async move {
    node.run().await.unwrap();
});
```

---

## Error Handling

All APIs use `Result<T, E>` for error handling:

```rust
use rsedsim::error::*;

match model.add_stock(stock) {
    Ok(_) => println!("Stock added successfully"),
    Err(ModelError::DuplicateName(name)) => {
        eprintln!("Stock '{}' already exists", name);
    }
    Err(e) => eprintln!("Error: {}", e),
}

// Or use `?` operator
fn build_model() -> Result<Model, ModelError> {
    let mut model = Model::new("My Model");
    model.add_stock(stock1)?;
    model.add_stock(stock2)?;
    model.add_flow(flow1)?;
    Ok(model)
}
```

Common error types:
- `ModelError`: Model construction/validation errors
- `SimError`: Simulation execution errors
- `ParseError`: Model parsing errors
- `IoError`: File I/O errors
- `AgentError`: Agent behavior errors
- `McpError`: MCP protocol errors
- `A2aError`: A2A protocol errors

---

## Examples

### Complete SIR Model

```rust
use rsedsim::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create model
    let mut model = Model::new("SIR Model");
    model.set_time_config(TimeConfig {
        start: 0.0,
        stop: 100.0,
        dt: 0.25,
        units: Some("days".into()),
    });

    // Add stocks
    model.add_stock(Stock::new("Susceptible", "990"))?;
    model.add_stock(Stock::new("Infected", "10"))?;
    model.add_stock(Stock::new("Recovered", "0"))?;

    // Add flows
    model.add_flow(Flow::new(
        "infection_rate",
        "contact_rate * infectivity * Susceptible * Infected / total_population"
    ))?;
    model.add_flow(Flow::new("recovery_rate", "Infected / duration"))?;

    // Add auxiliaries
    model.add_auxiliary(Auxiliary::new(
        "total_population",
        "Susceptible + Infected + Recovered"
    ))?;

    // Add parameters
    model.add_parameter(Parameter::new("contact_rate", 5.0))?;
    model.add_parameter(Parameter::new("infectivity", 0.25))?;
    model.add_parameter(Parameter::new("duration", 5.0))?;

    // Connect flows to stocks
    model.connect_flow("infection_rate", "Susceptible", FlowDirection::Outflow)?;
    model.connect_flow("infection_rate", "Infected", FlowDirection::Inflow)?;
    model.connect_flow("recovery_rate", "Infected", FlowDirection::Outflow)?;
    model.connect_flow("recovery_rate", "Recovered", FlowDirection::Inflow)?;

    // Validate
    model.validate()?;

    // Run simulation
    let config = SimulationConfig::default();
    let mut sim = SimulationEngine::new(model, config)?;
    let results = sim.run()?;

    // Export results
    CsvWriter::new().write_file(&results, "sir_results.csv")?;

    Ok(())
}
```

---

For more examples, see the [examples/](../examples/) directory.
