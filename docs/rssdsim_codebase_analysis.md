# RSSDSIM Codebase Analysis
## Rust System Dynamics Simulator

**Project Name**: rssdsim  
**Language**: Rust (Edition 2024)  
**Version**: 0.1.0  
**Repository**: https://github.com/yourusername/rsedsim

---

## 1. OVERALL PROJECT STRUCTURE

### Directory Layout
```
/home/svvs/rssdsim/
├── src/
│   ├── main.rs                 # CLI interface and orchestration
│   ├── model/                  # System dynamics model definitions
│   │   ├── mod.rs             # Main Model struct
│   │   ├── stock.rs           # Stock (level) variables
│   │   ├── flow.rs            # Flow (rate) variables
│   │   ├── auxiliary.rs       # Auxiliary (converter) variables
│   │   ├── parameter.rs       # Parameter (constant) variables
│   │   └── expression.rs      # Expression parser and evaluator
│   ├── simulation/             # Simulation execution
│   │   ├── mod.rs             # SimulationState, SimulationConfig, SimulationResults
│   │   ├── engine.rs          # SimulationEngine orchestrator
│   │   └── integrator.rs      # Integration methods (Euler, RK4)
│   ├── io/                     # Input/Output and parsing
│   │   ├── mod.rs             # I/O dispatcher
│   │   ├── parser.rs          # JSON/YAML parsers
│   │   ├── writer.rs          # CSV output writer
│   │   ├── xmile.rs           # XMILE XML parser
│   │   └── insightmaker.rs    # InsightMaker JSON parser
│   └── protocol/               # External communication protocols
│       ├── mod.rs             # Protocol module
│       ├── mcp.rs             # Model Context Protocol (stubs)
│       └── a2a.rs             # Agent-to-Agent protocol (stubs)
├── examples/
│   ├── exponential_growth.yaml
│   ├── bank_account.json
│   ├── sir_epidemic.yaml
│   └── insightmaker/
└── Cargo.toml                  # Dependencies and metadata
```

### Key Technologies
- **Core Numerics**: ndarray (0.15), nalgebra (0.32)
- **Parsing**: pest (2.7) for expression parsing
- **Serialization**: serde, serde_json, serde_yaml
- **CLI**: clap (4.5)
- **XML Support**: quick-xml (0.31) for XMILE parsing
- **Async**: tokio (1.35) for protocol support
- **Testing**: criterion (benchmarking), proptest (property-based testing)

---

## 2. INTEGRATION METHODS IMPLEMENTATION

### Location: `/home/svvs/rssdsim/src/simulation/integrator.rs`

#### Current Integration Methods

##### A. **Euler (Forward) Integration**

```rust
pub struct EulerIntegrator;

impl Integrator for EulerIntegrator {
    fn step(&self, model: &Model, state: &SimulationState, dt: f64) 
        -> Result<SimulationState, String>
}
```

**Algorithm**:
1. Evaluate auxiliaries with fixed-point iteration (max 20 passes)
2. Evaluate flows based on current state
3. Compute stock derivatives as: `derivative = sum(inflows) - sum(outflows)`
4. Apply Euler formula: `stock(t+dt) = stock(t) + derivative * dt`
5. Apply constraints (non_negative, max_value)

**Key Features**:
- Simple and fast
- Less accurate for stiff equations
- Default integrator

##### B. **RK4 (Runge-Kutta 4th Order) Integration**

```rust
pub struct RK4Integrator;

impl RK4Integrator {
    fn evaluate_system(...) -> Result<(HashMap<String, f64>, HashMap<String, f64>), String>
    fn compute_derivatives(...) -> Result<HashMap<String, f64>, String>
    fn apply_stock_increments(...) -> SimulationState
}

impl Integrator for RK4Integrator {
    fn step(&self, model: &Model, state: &SimulationState, dt: f64) 
        -> Result<SimulationState, String>
}
```

**Algorithm**:
```
RK4: y_{n+1} = y_n + (k1 + 2*k2 + 2*k3 + k4) * dt / 6

Where:
  k1 = f(t_n, y_n)
  k2 = f(t_n + dt/2, y_n + k1*dt/2)
  k3 = f(t_n + dt/2, y_n + k2*dt/2)
  k4 = f(t_n + dt, y_n + k3*dt)
```

**Key Features**:
- 4th order accuracy
- More computationally expensive (4 function evaluations per step)
- Better for non-linear systems
- Helper methods for auxiliary evaluation and stock increments

#### Auxiliary Variable Evaluation (Fixed-Point Iteration)

Both integrators use the same approach for auxiliaries:
```rust
// Iterative evaluation with convergence check
for pass in 0..MAX_PASSES {
    for (name, aux) in &model.auxiliaries {
        match aux.equation.evaluate(&context) {
            Ok(value) => {
                if diff.abs() > 1e-10 {
                    changed = true;
                }
                auxiliaries.insert(name.clone(), value);
            }
            Err(e) => {
                if pass >= 5 {
                    return Err(...);  // After 5 passes, fail on error
                }
                any_errors = true;
            }
        }
    }
    if !changed && !any_errors && pass > 0 {
        break;  // Converged
    }
}
```

#### Integration Method Selection

**Location**: `/home/svvs/rssdsim/src/simulation/engine.rs` (line 35-38)

```rust
let integrator: Box<dyn Integrator> = match self.config.integration_method {
    IntegrationMethod::Euler => Box::new(EulerIntegrator),
    IntegrationMethod::RK4 => Box::new(RK4Integrator),
};
```

**CLI Usage** (from main.rs):
```bash
rsedsim run model.yaml --integrator euler|rk4
```

**Default**: Euler

---

## 3. FUNCTION/BUILTIN SUPPORT

### Location: `/home/svvs/rssdsim/src/model/expression.rs` (lines 493-548)

#### Implemented Built-in Functions

All functions are **case-insensitive** and return floating-point numbers.

| Function | Arity | Description | Implementation |
|----------|-------|-------------|-----------------|
| `MIN` | 2+ | Minimum of arguments | `iter().fold(f64::INFINITY, f64::min)` |
| `MAX` | 2+ | Maximum of arguments | `iter().fold(f64::NEG_INFINITY, f64::max)` |
| `ABS` | 1 | Absolute value | `arg.abs()` |
| `SQRT` | 1 | Square root | `arg.sqrt()` |
| `EXP` | 1 | Exponential (e^x) | `arg.exp()` |
| `LN` | 1 | Natural logarithm | `arg.ln()` |
| `SIN` | 1 | Sine (radians) | `arg.sin()` |
| `COS` | 1 | Cosine (radians) | `arg.cos()` |
| `TIME` | 0 | Current simulation time | Returns context.time |

#### Function Evaluation Process

```rust
fn evaluate_function(name: &str, args: &[Expression], context: &EvaluationContext) 
    -> Result<f64, String>
{
    // 1. Evaluate all argument expressions first
    let arg_values: Result<Vec<f64>, String> = args
        .iter()
        .map(|arg| arg.evaluate(context))
        .collect();
    
    // 2. Pattern match on function name (case-insensitive)
    match name.to_uppercase().as_str() {
        "MIN" => {...}
        "MAX" => {...}
        // ... etc
        _ => Err(format!("Unknown function: '{}'", name))
    }
}
```

#### Variadic Functions
- `MIN()` and `MAX()` accept 2 or more arguments
- Example: `MIN(a, b, c, d)` → returns minimum value

#### Type System
- All functions work with f64 (64-bit floating-point numbers)
- Comparison operators return 1.0 (true) or 0.0 (false)

---

## 4. ARRAY/SUBSET IMPLEMENTATION

### Current Status: **NOT YET IMPLEMENTED**

The codebase currently has **NO support** for:
- Multi-dimensional variables/arrays
- Subscripting/indexing
- Vector operations
- Subset operations

#### Future Plans (from main.rs line 222):
```rust
println!("  ○ Multi-dimensional variables (planned)");
```

#### Dependencies Prepared for Arrays
The Cargo.toml includes these libraries which could support arrays:
- `ndarray = "0.15"` - Multi-dimensional array library
- `nalgebra = "0.32"` - Linear algebra library

#### Where Array Support Would Be Needed
1. **expression.rs**: Extend Expression enum with subscript/index variants
2. **SimulationState**: Extend to store multi-dimensional data
3. **Parser**: Add syntax for `variable[dimension]` subscripts
4. **Integrator**: Handle array derivatives in RK4/Euler

---

## 5. KEY DATA STRUCTURES

### A. Model Representation

**Location**: `/home/svvs/rssdsim/src/model/mod.rs`

```rust
pub struct Model {
    pub metadata: ModelMetadata,
    pub time: TimeConfig,
    pub stocks: HashMap<String, Stock>,
    pub flows: HashMap<String, Flow>,
    pub auxiliaries: HashMap<String, Auxiliary>,
    pub parameters: HashMap<String, Parameter>,
}

pub struct TimeConfig {
    pub start: f64,
    pub stop: f64,
    pub dt: f64,
    pub units: Option<String>,
}

pub struct ModelMetadata {
    pub name: String,
    pub description: Option<String>,
    pub author: Option<String>,
}
```

### B. Variable Definitions

**Stock** (`stock.rs`):
```rust
pub struct Stock {
    pub name: String,
    pub initial: Expression,          // Initial value (can be expression)
    pub inflows: Vec<String>,         // Names of inflow variables
    pub outflows: Vec<String>,        // Names of outflow variables
    pub units: Option<String>,
    pub non_negative: bool,           // Constraint: >= 0
    pub max_value: Option<f64>,       // Constraint: <= max
}
```

**Flow** (`flow.rs`):
```rust
pub struct Flow {
    pub name: String,
    pub equation: Expression,
    pub units: Option<String>,
}
```

**Auxiliary** (`auxiliary.rs`):
```rust
pub struct Auxiliary {
    pub name: String,
    pub equation: Expression,
    pub units: Option<String>,
}
```

**Parameter** (`parameter.rs`):
```rust
pub struct Parameter {
    pub name: String,
    pub value: f64,
    pub units: Option<String>,
    pub description: Option<String>,
}
```

### C. Expression Tree

**Location**: `/home/svvs/rssdsim/src/model/expression.rs`

```rust
pub enum Expression {
    Constant(f64),
    Variable(String),
    BinaryOp {
        op: Operator,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    UnaryOp {
        op: UnaryOperator,
        expr: Box<Expression>,
    },
    FunctionCall {
        name: String,
        args: Vec<Expression>,
    },
    Conditional {
        condition: Box<Expression>,
        true_expr: Box<Expression>,
        false_expr: Box<Expression>,
    },
}

pub enum Operator {
    Add, Subtract, Multiply, Divide, Power,
    GreaterThan, LessThan, GreaterEqual, LessEqual,
    Equal, NotEqual,
}

pub enum UnaryOperator {
    Negate,
}
```

### D. Simulation State

**Location**: `/home/svvs/rssdsim/src/simulation/mod.rs`

```rust
pub struct SimulationState {
    pub time: f64,
    pub stocks: HashMap<String, f64>,      // Current stock values
    pub flows: HashMap<String, f64>,       // Current flow rates
    pub auxiliaries: HashMap<String, f64>, // Current auxiliary values
}

pub struct SimulationConfig {
    pub integration_method: IntegrationMethod,
    pub output_interval: Option<f64>,      // Recording interval
}

pub enum IntegrationMethod {
    Euler,
    RK4,
}

pub struct SimulationResults {
    pub times: Vec<f64>,
    pub states: Vec<SimulationState>,
}
```

### E. Evaluation Context

**Location**: `/home/svvs/rssdsim/src/model/expression.rs` (lines 552-571)

```rust
pub struct EvaluationContext<'a> {
    pub model: &'a Model,
    pub state: &'a SimulationState,
    pub time: f64,
}

impl<'a> EvaluationContext<'a> {
    pub fn get_variable(&self, name: &str) -> Result<f64, String> {
        // Priority: TIME (special), parameters, stocks, flows, auxiliaries
        if name.to_uppercase() == "TIME" {
            return Ok(self.time);
        }
        self.model.get_variable(name, self.state)
    }
}
```

---

## 6. EXPRESSION EVALUATION FLOW

### Parsing Phase

**Location**: `/home/svvs/rssdsim/src/model/expression.rs` (lines 55-133)

The parser uses **recursive descent** with **operator precedence**:

```
1. Parse conditional (IF THEN ELSE)
2. Parse comparison (>, <, >=, <=, ==, !=)
3. Parse addition/subtraction (+, -)
4. Parse multiplication/division (*, /)
5. Parse exponentiation (^)
6. Handle parentheses and function calls
7. Handle unary minus
8. Fall back to variable names
```

**Example**: `IF x > 5 THEN Population * growth_rate ELSE 0`

### Evaluation Phase

**Location**: `/home/svvs/rssdsim/src/model/expression.rs` (lines 437-491)

```rust
impl Expression {
    pub fn evaluate(&self, context: &EvaluationContext) -> Result<f64, String> {
        match self {
            Expression::Constant(val) => Ok(*val),
            
            Expression::Variable(name) => {
                context.get_variable(name)
            },
            
            Expression::BinaryOp { op, left, right } => {
                let left_val = left.evaluate(context)?;
                let right_val = right.evaluate(context)?;
                Ok(match op {
                    Operator::Add => left_val + right_val,
                    Operator::Multiply => left_val * right_val,
                    // ... etc
                })
            },
            
            Expression::FunctionCall { name, args } => {
                Self::evaluate_function(name, args, context)
            },
            
            Expression::Conditional { condition, true_expr, false_expr } => {
                let cond_val = condition.evaluate(context)?;
                if cond_val > 0.5 {  // Lazy evaluation!
                    true_expr.evaluate(context)
                } else {
                    false_expr.evaluate(context)
                }
            },
        }
    }
}
```

### Variable Resolution Order

**Location**: `/home/svvs/rssdsim/src/model/mod.rs` (lines 109-132)

When evaluating a variable reference:
```
1. Check if it's a parameter → return param.value
2. Check current stocks → return stock value
3. Check current flows → return flow value
4. Check current auxiliaries → return auxiliary value
5. If not found → return error
```

---

## 7. SIMULATION EXECUTION FLOW

### Simulation Engine

**Location**: `/home/svvs/rssdsim/src/simulation/engine.rs`

```rust
pub struct SimulationEngine {
    model: Model,
    config: SimulationConfig,
    state: SimulationState,
}

impl SimulationEngine {
    pub fn run(&mut self) -> Result<SimulationResults, String> {
        let mut results = SimulationResults::new();
        results.add_point(self.state.time, self.state.clone());
        
        let integrator: Box<dyn Integrator> = match self.config.integration_method {
            IntegrationMethod::Euler => Box::new(EulerIntegrator),
            IntegrationMethod::RK4 => Box::new(RK4Integrator),
        };
        
        while self.state.time < stop_time {
            self.state = integrator.step(&self.model, &self.state, dt)?;
            if self.state.time > stop_time {
                self.state.time = stop_time;  // Don't overshoot
            }
            results.add_point(self.state.time, self.state.clone());
        }
        
        Ok(results)
    }
}
```

### Complete Simulation Workflow

```
1. Load Model (JSON/YAML/XMILE/InsightMaker)
   └─ Parse expressions in flows, auxiliaries, stocks

2. Initialize SimulationState
   └─ Evaluate initial values for stocks
   └─ Initialize flows and auxiliaries to 0.0

3. Main Loop (time < stop_time):
   
   a. Execute Integration Step:
      i. Evaluate auxiliaries (fixed-point iteration)
      ii. Evaluate flows
      iii. Compute stock derivatives
      iv. Apply integration formula (Euler/RK4)
      v. Apply constraints (non_negative, max_value)
   
   b. Record Results
      └─ Add state to SimulationResults if output interval reached
   
4. Write Output
   └─ CSV format with columns: time, stock1, stock2, ..., flow1, ..., aux1, ...
```

---

## 8. WORKFLOW: HOW DERIVATIVES ARE COMPUTED

### Derivative Computation (Both Integrators)

```rust
fn compute_derivatives(
    model: &Model,
    flows: &HashMap<String, f64>,
) -> Result<HashMap<String, f64>, String> {
    for (stock_name, stock) in &model.stocks {
        let mut derivative = 0.0;
        
        // Sum inflows
        for inflow_name in &stock.inflows {
            if let Some(flow_value) = flows.get(inflow_name) {
                derivative += flow_value;
            }
        }
        
        // Subtract outflows
        for outflow_name in &stock.outflows {
            if let Some(flow_value) = flows.get(outflow_name) {
                derivative -= flow_value;
            }
        }
        
        derivatives.insert(stock_name.clone(), derivative);
    }
    Ok(derivatives)
}
```

### Constraint Handling

After integration step:
```rust
let constrained_value = if let Some(stock) = model.stocks.get(stock_name) {
    let mut value = new_value;
    
    if stock.non_negative {
        value = value.max(0.0);  // Clamp to >= 0
    }
    
    if let Some(max_val) = stock.max_value {
        value = value.min(max_val);  // Clamp to <= max
    }
    
    value
} else {
    new_value
};
```

---

## 9. INPUT/OUTPUT FORMATS

### Supported Input Formats

#### JSON Model Format
**Location**: `/home/svvs/rssdsim/src/io/parser.rs`

```json
{
  "model": {
    "name": "Model Name",
    "description": "Optional description",
    "time": {
      "start": 0.0,
      "stop": 100.0,
      "dt": 0.1,
      "units": "years"
    },
    "stocks": [
      {
        "name": "Population",
        "initial": 100,
        "inflows": ["births"],
        "outflows": ["deaths"],
        "units": "people"
      }
    ],
    "flows": [
      {
        "name": "births",
        "equation": "Population * birth_rate",
        "units": "people/year"
      }
    ],
    "auxiliaries": [
      {
        "name": "net_rate",
        "equation": "birth_rate - death_rate"
      }
    ],
    "parameters": [
      {
        "name": "birth_rate",
        "value": 0.1,
        "units": "1/year",
        "description": "Birth rate"
      }
    ]
  }
}
```

#### YAML Model Format
**Location**: `/home/svvs/rssdsim/src/io/parser.rs`

Uses same structure as JSON but in YAML syntax:

```yaml
model:
  name: Population Growth
  time:
    start: 0
    stop: 10
    dt: 0.1
  stocks:
    - name: Population
      initial: 100
      inflows: [births]
  flows:
    - name: births
      equation: Population * birth_rate
  parameters:
    - name: birth_rate
      value: 0.1
```

#### XMILE Format
**Location**: `/home/svvs/rssdsim/src/io/xmile.rs`

XML-based System Dynamics Interchange format. Supports:
- Stella
- Vensim
- InsightMaker XMILE exports

#### InsightMaker Format
**Location**: `/home/svvs/rssdsim/src/io/insightmaker.rs`

JSON format specific to InsightMaker platform with:
- Primitives (stocks, flows, auxiliaries, parameters)
- Variable types mapping

### Output Format

**Location**: `/home/svvs/rssdsim/src/io/writer.rs`

CSV output with structure:
```
time,Population,births,growth_rate
0.0,100.0,10.0,0.1
0.1,100.1,10.01,0.1
0.2,100.2,10.02,0.1
...
```

---

## 10. CLI INTERFACE

**Location**: `/home/svvs/rssdsim/src/main.rs`

### Available Commands

#### Run Simulation
```bash
rsedsim run <model.yaml|json|xmile>
  --output <output.csv>              # Output file (default: results.csv)
  --params "param1=10,param2=0.5"    # Parameter overrides
  --integrator <euler|rk4>           # Integration method (default: euler)
  --dt <0.1>                         # Override timestep
```

#### Validate Model
```bash
rsedsim validate <model.yaml|json|xmile>
```

Checks:
- All flows referenced by stocks exist
- Model structure validity

#### Show Info
```bash
rsedsim info
```

---

## 11. TESTING

### Existing Tests

#### Expression Parser Tests
**Location**: `/home/svvs/rssdsim/src/model/expression.rs` (lines 617-655)

- `test_parse_constant()`
- `test_parse_variable()`
- `test_parse_simple_conditional()`
- `test_parse_nested_conditional()`
- `test_parse_addition()`
- `test_parse_multiplication()`

#### Integration Tests
**Location**: `/home/svvs/rssdsim/src/simulation/integrator.rs` (lines 331-361)

- `test_euler_simple_growth()` - Tests basic exponential growth

#### Parser Tests
**Location**: `/home/svvs/rssdsim/src/io/parser.rs` (lines 154-189)

- `test_parse_json_simple()`

#### Engine Tests
**Location**: `/home/svvs/rssdsim/src/simulation/engine.rs` (lines 97-123)

- `test_simulation_engine_simple()`

### Running Tests
```bash
cargo test
cargo test --release
cargo test --doc
```

---

## 12. ARCHITECTURE PATTERNS

### Design Patterns Used

1. **Trait-based Polymorphism**: `Integrator` trait for swappable integrators
2. **Recursive Descent Parsing**: Expression parser with precedence handling
3. **Lazy Evaluation**: Conditional expressions only evaluate needed branches
4. **Fixed-Point Iteration**: Auxiliary variable convergence
5. **Builder Pattern**: Stock/Flow/Auxiliary/Parameter with builder methods
6. **State Machine**: Simulation engine lifecycle

### Dependency Injection
- SimulationEngine accepts Model and SimulationConfig
- Integrator trait abstraction
- Parser trait for model loading

---

## 13. CURRENT LIMITATIONS

1. **No array/multidimensional support** - Listed as "planned"
2. **No delays** - No delay functions like DELAY1, DELAY3, etc.
3. **No lookup tables** - Cannot reference external data tables
4. **Limited built-ins** - Only 9 functions (MIN, MAX, ABS, SQRT, EXP, LN, SIN, COS, TIME)
5. **No random number generation** - Though rand/rand_distr are in dependencies
6. **Limited protocol support** - MCP and A2A are stubs only
7. **Simple fixed-point iteration** - No sophisticated convergence detection for auxiliaries
8. **Case-sensitive variable names** - But function names are case-insensitive

---

## 14. PLANNED FEATURES (From Code Comments)

From `main.rs` (lines 220-226):
```
✓ Stock-flow models (DONE)
✓ Expression evaluation (DONE)
✓ Multiple integrators (Euler, RK4) (DONE)
✓ JSON/YAML model format (DONE)
✓ CSV output (DONE)
○ Agent-based modeling (PLANNED)
○ Hybrid models (PLANNED)
○ Multi-dimensional variables (PLANNED)
○ MCP (Model Context Protocol) integration (STUBS)
○ A2A (Agent-to-Agent) protocol (STUBS)
```

---

## 15. EXAMPLE WALKTHROUGH

### Simple Growth Model

**Model** (`examples/exponential_growth.yaml`):
```yaml
model:
  name: Exponential Growth
  time: {start: 0, stop: 10, dt: 0.1}
  stocks:
    - name: Population
      initial: 100
      inflows: [births]
  flows:
    - name: births
      equation: Population * birth_rate
  parameters:
    - name: birth_rate
      value: 0.1
```

**Execution**:
1. Parse model → Stock "Population" with initial=100, Flow "births" with equation
2. Initialize state → Population=100, births=0, time=0
3. Step 1 (t=0 → 0.1):
   - births = 100 * 0.1 = 10
   - d(Population)/dt = 10
   - Population(0.1) = 100 + 10 * 0.1 = 101
4. Step 2 (t=0.1 → 0.2):
   - births = 101 * 0.1 = 10.1
   - d(Population)/dt = 10.1
   - Population(0.2) = 101 + 10.1 * 0.1 ≈ 102.01
5. Continue until t > 10

**Mathematical Model**:
```
dP/dt = P * r
P(t) = P(0) * e^(rt) = 100 * e^(0.1t)
```

---

## 16. KEY FILES SUMMARY

| File | Lines | Purpose |
|------|-------|---------|
| `src/main.rs` | 238 | CLI interface, orchestration |
| `src/model/expression.rs` | 656 | Expression parsing & evaluation |
| `src/simulation/integrator.rs` | 362 | Euler & RK4 implementations |
| `src/simulation/engine.rs` | 125 | Simulation orchestration |
| `src/simulation/mod.rs` | 131 | State and configuration types |
| `src/io/parser.rs` | 191 | JSON/YAML parsing |
| `src/io/xmile.rs` | 300+ | XMILE XML parsing |
| `src/io/insightmaker.rs` | 200+ | InsightMaker format support |
| `src/model/mod.rs` | 144 | Model data structures |
| `Cargo.toml` | 73 | Dependencies |

---

## 17. COMPARISON: EULER vs RK4

| Aspect | Euler | RK4 |
|--------|-------|-----|
| Accuracy | 1st order | 4th order |
| Function Evals/Step | 1 (eval auxiliaries once) | 4 (eval at 4 stages) |
| Speed | Fast | ~4x slower |
| Stability | Less stable | More stable |
| Best For | Simple models | Non-linear, stiff systems |
| Error Per Step | O(dt²) | O(dt⁵) |
| Global Error | O(dt) | O(dt⁴) |

---

## CONCLUSION

The rssdsim codebase is a well-structured, production-ready System Dynamics simulator written in Rust. It provides:

- **Solid foundation**: Stock-flow dynamics with multiple integration methods
- **Flexible I/O**: Support for multiple model formats (JSON, YAML, XMILE, InsightMaker)
- **Expression evaluation**: Comprehensive parser with built-in functions
- **Extensibility**: Trait-based design for integrators and parsers
- **Future-ready**: Dependencies prepared for arrays (ndarray), linear algebra (nalgebra), and async protocols (tokio)

The main limitations are the absence of multi-dimensional array support, limited built-in functions, and incomplete protocol stubs.

