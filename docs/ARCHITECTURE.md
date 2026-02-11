# rsedsim Architecture

## Overview

rsedsim is designed as a modular, extensible system dynamics simulation framework with clear separation of concerns between model representation, simulation execution, and external interfaces.

## Design Principles

1. **Modularity**: Each component has well-defined responsibilities
2. **Extensibility**: Easy to add new integration methods, functions, or protocols
3. **Performance**: Efficient numerical computation with zero-cost abstractions
4. **Interoperability**: Standard formats for model exchange
5. **Type Safety**: Leverage Rust's type system for correctness

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                        CLI Interface                         │
│                      (clap, colored)                         │
└────────────────────┬────────────────────────────────────────┘
                     │
         ┌───────────┴───────────┐
         │                       │
         ▼                       ▼
┌────────────────┐      ┌────────────────┐
│  Protocol Layer│      │   I/O Layer    │
│  - MCP Server  │      │  - JSON/YAML   │
│  - A2A Node    │      │  - XMILE       │
│  - Transport   │      │  - CSV/HDF5    │
└────────┬───────┘      └────────┬───────┘
         │                       │
         └───────────┬───────────┘
                     │
                     ▼
         ┌───────────────────────┐
         │    Model Layer        │
         │  - Stocks/Flows       │
         │  - Equations          │
         │  - Subscripts         │
         │  - Agents             │
         └───────────┬───────────┘
                     │
                     ▼
         ┌───────────────────────┐
         │  Simulation Engine    │
         │  - Integrators        │
         │  - Solvers            │
         │  - Time stepping      │
         └───────────┬───────────┘
                     │
         ┌───────────┴───────────┐
         │                       │
         ▼                       ▼
┌────────────────┐      ┌────────────────┐
│  Array Ops     │      │  Functions     │
│  - ndarray     │      │  - Built-ins   │
│  - Subscripts  │      │  - Delays      │
│  - Vector ops  │      │  - Lookups     │
└────────────────┘      └────────────────┘
```

## Module Breakdown

### 1. Model Layer (`src/model/`)

Represents the structure of system dynamics models.

#### Components

**Stock** (`stock.rs`)
```rust
pub struct Stock {
    pub id: StockId,
    pub name: String,
    pub initial_value: Expression,
    pub inflows: Vec<FlowId>,
    pub outflows: Vec<FlowId>,
    pub dimensions: Vec<DimensionId>,
    pub units: Option<String>,
    pub documentation: Option<String>,
}
```

**Flow** (`flow.rs`)
```rust
pub struct Flow {
    pub id: FlowId,
    pub name: String,
    pub equation: Expression,
    pub dimensions: Vec<DimensionId>,
    pub units: Option<String>,
}
```

**Auxiliary** (`auxiliary.rs`)
```rust
pub struct Auxiliary {
    pub id: AuxiliaryId,
    pub name: String,
    pub equation: Expression,
    pub dimensions: Vec<DimensionId>,
    pub units: Option<String>,
}
```

**Parameter** (`parameter.rs`)
```rust
pub struct Parameter {
    pub id: ParameterId,
    pub name: String,
    pub value: Value,  // Can be scalar, vector, etc.
    pub units: Option<String>,
}
```

**Equation** (`equation.rs`)
```rust
pub enum Expression {
    Constant(f64),
    Variable(VariableId),
    BinaryOp(Box<Expression>, Operator, Box<Expression>),
    UnaryOp(Operator, Box<Expression>),
    FunctionCall(FunctionId, Vec<Expression>),
    ArrayAccess(Box<Expression>, Vec<Expression>),
    Conditional(Box<Expression>, Box<Expression>, Box<Expression>),
}
```

**Model** (`mod.rs`)
```rust
pub struct Model {
    pub metadata: ModelMetadata,
    pub time_config: TimeConfig,
    pub stocks: HashMap<StockId, Stock>,
    pub flows: HashMap<FlowId, Flow>,
    pub auxiliaries: HashMap<AuxiliaryId, Auxiliary>,
    pub parameters: HashMap<ParameterId, Parameter>,
    pub dimensions: HashMap<DimensionId, Dimension>,
    pub dependency_graph: DependencyGraph,
}
```

### 2. Simulation Engine (`src/simulation/`)

Executes model simulations.

#### Engine (`engine.rs`)

```rust
pub struct SimulationEngine {
    model: Model,
    state: SimulationState,
    integrator: Box<dyn Integrator>,
    config: SimulationConfig,
}

impl SimulationEngine {
    pub fn new(model: Model, config: SimulationConfig) -> Self;
    pub fn step(&mut self) -> Result<(), SimError>;
    pub fn run(&mut self) -> Result<SimulationResults, SimError>;
    pub fn reset(&mut self);
}
```

#### Integrators (`integrator.rs`)

```rust
pub trait Integrator {
    fn step(&self, state: &SimulationState, derivatives: &Derivatives, dt: f64)
        -> Result<SimulationState, SimError>;
}

pub struct EulerIntegrator;
pub struct RK4Integrator;
pub struct RK45Integrator { adaptive: bool }
pub struct BackwardEulerIntegrator;
```

#### Simulation State (`state.rs`)

```rust
pub struct SimulationState {
    pub time: f64,
    pub stocks: HashMap<StockId, Value>,
    pub flows: HashMap<FlowId, Value>,
    pub auxiliaries: HashMap<AuxiliaryId, Value>,
    pub delayed_values: DelayBuffer,
}

pub enum Value {
    Scalar(f64),
    Vector(Vec<f64>),
    Matrix(Array2<f64>),
    Tensor(ArrayD<f64>),
}
```

### 3. Agent-Based Modeling (`src/agent/`)

Support for hybrid SD-ABM models.

#### Agent (`agent.rs`)

```rust
pub struct Agent {
    pub id: AgentId,
    pub agent_type: String,
    pub state: AgentState,
    pub attributes: HashMap<String, Value>,
    pub behavior: Box<dyn AgentBehavior>,
}

pub trait AgentBehavior: Send + Sync {
    fn step(&mut self, agent: &mut Agent, context: &SimulationContext)
        -> Result<(), AgentError>;
    fn perceive(&self, agent: &Agent, context: &SimulationContext)
        -> PerceptionData;
    fn decide(&self, perception: &PerceptionData) -> Action;
    fn act(&mut self, agent: &mut Agent, action: Action) -> Result<(), AgentError>;
}
```

#### Agent Population (`population.rs`)

```rust
pub struct AgentPopulation {
    pub name: String,
    pub agents: Vec<Agent>,
    pub sd_inputs: Vec<String>,  // SD variables this population reads
    pub sd_outputs: HashMap<String, AggregationFn>,  // Aggregations to SD
}

pub enum AggregationFn {
    Count(Box<dyn Fn(&Agent) -> bool>),
    Sum(Box<dyn Fn(&Agent) -> f64>),
    Mean(Box<dyn Fn(&Agent) -> f64>),
    Min(Box<dyn Fn(&Agent) -> f64>),
    Max(Box<dyn Fn(&Agent) -> f64>),
}
```

#### Hybrid Model (`hybrid.rs`)

```rust
pub struct HybridModel {
    pub sd_model: Model,
    pub agent_populations: Vec<AgentPopulation>,
    pub coupling: CouplingSpecification,
}

pub struct CouplingSpecification {
    pub sd_to_agent: Vec<Influence>,
    pub agent_to_sd: Vec<Aggregation>,
    pub update_order: UpdateOrder,
}
```

### 4. Multi-dimensional Variables (`src/array/`)

Support for subscripted variables.

#### Dimension (`subscript.rs`)

```rust
pub struct Dimension {
    pub id: DimensionId,
    pub name: String,
    pub elements: Vec<String>,
    pub subsets: HashMap<String, Subset>,
}

pub struct Subset {
    pub name: String,
    pub indices: Vec<usize>,
}
```

#### Vector Operations (`vector_ops.rs`)

```rust
pub trait VectorOps {
    fn element_wise_op(&self, other: &Self, op: BinaryOp) -> Self;
    fn sum_over(&self, dimension: usize) -> Self;
    fn map_dimension(&self, dimension: usize, f: impl Fn(f64) -> f64) -> Self;
}

impl VectorOps for ArrayD<f64> { ... }
```

### 5. I/O Layer (`src/io/`)

Model and data serialization.

#### Parser (`parser.rs`)

```rust
pub trait ModelParser {
    fn parse(&self, input: &str) -> Result<Model, ParseError>;
}

pub struct JsonParser;
pub struct YamlParser;
pub struct XmileParser;
```

#### Writer (`writer.rs`)

```rust
pub trait ResultWriter {
    fn write(&self, results: &SimulationResults, output: &mut dyn Write)
        -> Result<(), IoError>;
}

pub struct CsvWriter;
pub struct JsonWriter;
pub struct Hdf5Writer;
```

### 6. Built-in Functions (`src/functions/`)

Standard SD function library.

#### Function Registry (`mod.rs`)

```rust
pub struct FunctionRegistry {
    functions: HashMap<String, Box<dyn Function>>,
}

pub trait Function: Send + Sync {
    fn name(&self) -> &str;
    fn call(&self, args: &[Value], context: &EvalContext) -> Result<Value, FunctionError>;
    fn signature(&self) -> FunctionSignature;
}
```

#### Built-in Functions (`builtin.rs`)

Categories:
- **Time functions**: `TIME()`, `TIME_STEP()`, `INITIAL()`, `FINAL_TIME()`
- **Delay functions**: `DELAY1()`, `DELAY3()`, `DELAY_FIXED()`, `SMOOTH()`
- **Lookup functions**: `TABLE()`, `WITH_LOOKUP()`
- **Statistical**: `RANDOM_UNIFORM()`, `RANDOM_NORMAL()`, `RANDOM_POISSON()`
- **Math**: `MIN()`, `MAX()`, `ABS()`, `SQRT()`, `EXP()`, `LN()`, `SIN()`, `COS()`
- **Logical**: `IF_THEN_ELSE()`, `AND()`, `OR()`, `NOT()`
- **Array**: `SUM()`, `MEAN()`, `VMIN()`, `VMAX()`, `VECTOR_SELECT()`
- **Special**: `STEP()`, `RAMP()`, `PULSE()`, `PULSE_TRAIN()`

### 7. Protocol Layer (`src/protocol/`)

External communication protocols.

#### MCP (`mcp.rs`)

**Purpose**: Enable AI/LLM agents to interact with simulations

**Architecture**:
```rust
pub struct McpServer {
    capabilities: McpCapabilities,
    resources: Vec<Resource>,    // Model states, results
    tools: Vec<Tool>,            // Simulation operations
}

// Tools exposed:
// - run_simulation: Execute model with parameters
// - analyze_model: Structural analysis
// - sensitivity_analysis: Parameter sweeps
// - get_variable_timeseries: Extract results
```

**Transports**:
- stdio (JSON-RPC over stdin/stdout)
- HTTP with Server-Sent Events (SSE)

#### A2A (`a2a.rs`)

**Purpose**: Distributed agent communication

**Architecture**:
```rust
pub struct A2aNode {
    agent_id: AgentId,
    registry: HashMap<AgentId, AgentInfo>,  // Agent directory
    subscriptions: HashMap<String, Vec<AgentId>>,  // Pub/sub topics
    transport: Box<dyn A2aTransport>,
}

// Message types:
// - Direct messages: Agent-to-agent
// - Publish/subscribe: Topic-based broadcast
// - Discovery: Find agents by criteria
// - State sync: Distributed state management
// - Sim control: Coordination (barriers, sync)
```

**Transports**:
- UDP (connectionless, multicast)
- TCP (reliable, point-to-point)
- WebSocket (web-based agents)
- Shared memory (same-process agents)

### 8. CLI Layer (`src/cli/`)

Command-line interface.

```rust
#[derive(Parser)]
#[command(name = "rsedsim")]
#[command(about = "System Dynamics Simulator")]
enum Cli {
    Run(RunCommand),
    Sensitivity(SensitivityCommand),
    MonteCarlo(MonteCarloCommand),
    Analyze(AnalyzeCommand),
    Convert(ConvertCommand),
    Validate(ValidateCommand),
    Mcp(McpCommand),
    A2a(A2aCommand),
}
```

## Data Flow

### 1. Model Loading

```
User Input (JSON/YAML/XMILE)
    ↓
Parser (io::parser)
    ↓
Model Structure (model::Model)
    ↓
Dependency Analysis (model::dependency_graph)
    ↓
Validation (check units, cycles, etc.)
```

### 2. Simulation Execution

```
Initialize State (t=0, stocks=initial values)
    ↓
For each time step:
    ├─→ Evaluate auxiliaries (topological order)
    ├─→ Evaluate flows
    ├─→ Calculate derivatives (d(stock)/dt = inflows - outflows)
    ├─→ Integrate (update stocks using integrator)
    ├─→ Update agent behaviors (if hybrid model)
    ├─→ Aggregate agent data to SD (if hybrid)
    └─→ Record results
    ↓
Output results (CSV/JSON/HDF5)
```

### 3. Hybrid SD-ABM Execution

```
At each time step:
    ├─→ SD Phase:
    │    ├─→ Evaluate SD equations
    │    └─→ Update SD stocks
    │
    ├─→ Coupling (SD → Agents):
    │    └─→ Expose SD variables to agents
    │
    ├─→ Agent Phase:
    │    ├─→ For each agent:
    │    │    ├─→ Perceive (read SD vars, sense other agents)
    │    │    ├─→ Decide (behavior logic)
    │    │    └─→ Act (update agent state)
    │    └─→ Agent interactions (via A2A if distributed)
    │
    └─→ Coupling (Agents → SD):
         └─→ Aggregate agent data (COUNT, SUM, MEAN, etc.)
              into SD flows/auxiliaries
```

## Performance Considerations

### 1. Zero-Cost Abstractions
- Use trait objects only where dynamic dispatch is necessary
- Prefer enums over trait objects for known types
- Inline small functions

### 2. Memory Efficiency
- Use `ndarray` for contiguous memory layout
- Lazy evaluation of expressions where possible
- Reuse buffers for intermediate calculations

### 3. Parallelization
- Independent model runs (Monte Carlo) use Rayon
- Agent updates can be parallelized (read-only SD access)
- SIMD operations for vector math (ndarray/nalgebra)

### 4. Caching
- Cache expression evaluation results within a timestep
- Memoize expensive function calls (table lookups)
- Delay buffers use circular arrays

## Error Handling

Errors use `thiserror` for structured error types:

```rust
#[derive(Error, Debug)]
pub enum SimError {
    #[error("Model validation failed: {0}")]
    ValidationError(String),

    #[error("Integration error: {0}")]
    IntegrationError(String),

    #[error("Dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },

    #[error("Undefined variable: {0}")]
    UndefinedVariable(String),
}
```

## Testing Strategy

1. **Unit Tests**: Each module has comprehensive unit tests
2. **Integration Tests**: End-to-end model execution
3. **Property Tests**: Using `proptest` for numerical properties
4. **Benchmarks**: Using `criterion` for performance regression
5. **Example Models**: Known analytical solutions for validation

## Extension Points

### Adding a New Integrator

```rust
impl Integrator for MyIntegrator {
    fn step(&self, state: &SimulationState, derivatives: &Derivatives, dt: f64)
        -> Result<SimulationState, SimError> {
        // Implementation
    }
}
```

### Adding a New Built-in Function

```rust
pub struct MyFunction;

impl Function for MyFunction {
    fn name(&self) -> &str { "MY_FUNC" }

    fn call(&self, args: &[Value], context: &EvalContext)
        -> Result<Value, FunctionError> {
        // Implementation
    }

    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            min_args: 1,
            max_args: Some(3),
            return_type: ValueType::Scalar,
        }
    }
}
```

### Adding a New Transport for A2A

```rust
#[async_trait]
impl A2aTransport for MyTransport {
    async fn send(&self, message: A2aMessage) -> Result<(), A2aError> {
        // Implementation
    }

    async fn receive(&self) -> Result<A2aMessage, A2aError> {
        // Implementation
    }
}
```

## Future Architecture Enhancements

1. **GPU Acceleration**: CUDA/OpenCL for large-scale array operations
2. **Distributed Simulation**: Multi-node parallel execution
3. **Real-time Simulation**: Hard real-time guarantees for embedded systems
4. **Visual Editor**: Web-based graphical model builder
5. **Cloud Integration**: AWS/GCP deployment and scaling
