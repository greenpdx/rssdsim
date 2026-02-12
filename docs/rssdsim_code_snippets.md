# RSSDSIM Code Snippets Reference

Quick reference for key code sections in the codebase.

## 1. INTEGRATOR TRAIT DEFINITION

**File**: `src/simulation/integrator.rs:8-10`

```rust
pub trait Integrator {
    fn step(&self, model: &Model, state: &SimulationState, dt: f64) 
        -> Result<SimulationState, String>;
}
```

## 2. EULER INTEGRATION COMPLETE ALGORITHM

**File**: `src/simulation/integrator.rs:15-130`

```rust
impl Integrator for EulerIntegrator {
    fn step(&self, model: &Model, state: &SimulationState, dt: f64) 
        -> Result<SimulationState, String> 
    {
        let mut new_state = state.clone();
        new_state.time += dt;

        // Step 1: Evaluate auxiliaries with fixed-point iteration
        let mut new_auxiliaries = HashMap::new();
        for pass in 0..20 {  // MAX_PASSES = 20
            let mut changed = false;
            let context_with_aux = EvaluationContext::new(model, &new_state, state.time);
            let mut any_errors = false;

            for (name, aux) in &model.auxiliaries {
                match aux.equation.evaluate(&context_with_aux) {
                    Ok(value) => {
                        if let Some(&old_value) = new_auxiliaries.get(name) {
                            if (value - old_value).abs() > 1e-10 {
                                changed = true;
                            }
                        } else {
                            changed = true;
                        }
                        new_auxiliaries.insert(name.clone(), value);
                    }
                    Err(e) => {
                        if pass >= 5 {
                            return Err(format!("Error evaluating auxiliary '{}': {}", name, e));
                        }
                        any_errors = true;
                    }
                }
            }
            new_state.auxiliaries = new_auxiliaries.clone();
            
            if !changed && !any_errors && pass > 0 {
                break;  // Converged
            }
        }

        // Step 2: Evaluate flows
        let context = EvaluationContext::new(model, &new_state, state.time);
        let mut new_flows = HashMap::new();
        for (name, flow) in &model.flows {
            let value = flow.equation.evaluate(&context)?;
            new_flows.insert(name.clone(), value);
        }
        new_state.flows = new_flows;

        // Step 3: Compute derivatives
        let mut stock_derivatives: HashMap<String, f64> = HashMap::new();
        for (stock_name, stock) in &model.stocks {
            let mut derivative = 0.0;
            for inflow_name in &stock.inflows {
                if let Some(flow_value) = new_state.flows.get(inflow_name) {
                    derivative += flow_value;
                }
            }
            for outflow_name in &stock.outflows {
                if let Some(flow_value) = new_state.flows.get(outflow_name) {
                    derivative -= flow_value;
                }
            }
            stock_derivatives.insert(stock_name.clone(), derivative);
        }

        // Step 4: Apply Euler formula
        for (stock_name, current_value) in &state.stocks {
            if let Some(derivative) = stock_derivatives.get(stock_name) {
                let new_value = current_value + derivative * dt;
                
                // Apply constraints
                let constrained_value = if let Some(stock) = model.stocks.get(stock_name) {
                    let mut value = new_value;
                    if stock.non_negative {
                        value = value.max(0.0);
                    }
                    if let Some(max_val) = stock.max_value {
                        value = value.min(max_val);
                    }
                    value
                } else {
                    new_value
                };

                new_state.stocks.insert(stock_name.clone(), constrained_value);
            }
        }

        Ok(new_state)
    }
}
```

## 3. RK4 INTEGRATION CORE LOOP

**File**: `src/simulation/integrator.rs:254-327`

```rust
impl Integrator for RK4Integrator {
    fn step(&self, model: &Model, state: &SimulationState, dt: f64) 
        -> Result<SimulationState, String> 
    {
        let t = state.time;

        // Stage 1: k1 = f(t, y)
        let (_, flows1) = self.evaluate_system(model, state, t)?;
        let k1 = self.compute_derivatives(model, &flows1)?;

        // Stage 2: k2 = f(t + dt/2, y + k1*dt/2)
        let increments2: HashMap<String, f64> = k1.iter()
            .map(|(name, &deriv)| (name.clone(), deriv * dt / 2.0))
            .collect();
        let state2 = self.apply_stock_increments(state, &increments2);
        let (_, flows2) = self.evaluate_system(model, &state2, t + dt / 2.0)?;
        let k2 = self.compute_derivatives(model, &flows2)?;

        // Stage 3: k3 = f(t + dt/2, y + k2*dt/2)
        let increments3: HashMap<String, f64> = k2.iter()
            .map(|(name, &deriv)| (name.clone(), deriv * dt / 2.0))
            .collect();
        let state3 = self.apply_stock_increments(state, &increments3);
        let (_, flows3) = self.evaluate_system(model, &state3, t + dt / 2.0)?;
        let k3 = self.compute_derivatives(model, &flows3)?;

        // Stage 4: k4 = f(t + dt, y + k3*dt)
        let increments4: HashMap<String, f64> = k3.iter()
            .map(|(name, &deriv)| (name.clone(), deriv * dt))
            .collect();
        let state4 = self.apply_stock_increments(state, &increments4);
        let (_, flows4) = self.evaluate_system(model, &state4, t + dt)?;
        let k4 = self.compute_derivatives(model, &flows4)?;

        // Combine stages with RK4 weights
        let mut new_state = state.clone();
        new_state.time += dt;

        for (stock_name, &current_value) in &state.stocks {
            let d1 = k1.get(stock_name).unwrap_or(&0.0);
            let d2 = k2.get(stock_name).unwrap_or(&0.0);
            let d3 = k3.get(stock_name).unwrap_or(&0.0);
            let d4 = k4.get(stock_name).unwrap_or(&0.0);

            let new_value = current_value + (d1 + 2.0*d2 + 2.0*d3 + d4) * dt / 6.0;
            
            // Apply constraints
            let constrained_value = if let Some(stock) = model.stocks.get(stock_name) {
                let mut value = new_value;
                if stock.non_negative {
                    value = value.max(0.0);
                }
                if let Some(max_val) = stock.max_value {
                    value = value.min(max_val);
                }
                value
            } else {
                new_value
            };

            new_state.stocks.insert(stock_name.clone(), constrained_value);
        }

        new_state.auxiliaries = state4.1.0;  // Use final stage auxiliaries
        new_state.flows = flows4;

        Ok(new_state)
    }
}
```

## 4. EXPRESSION PARSER STRUCTURE

**File**: `src/model/expression.rs:52-133`

```rust
pub fn parse(s: &str) -> Result<Self, String> {
    let s = s.trim();

    // Try to parse as number
    if let Ok(num) = s.parse::<f64>() {
        return Ok(Expression::Constant(num));
    }

    // Try conditionals (highest level)
    if let Some(conditional) = Self::try_parse_conditional(s) {
        return conditional;
    }

    // Try comparisons
    if let Some(expr) = Self::try_parse_comparison(s) {
        return Ok(expr);
    }

    // Try binary operators in precedence order
    if let Some(expr) = Self::try_parse_binary(s, &['+', '-']) {
        return Ok(expr);
    }
    if let Some(expr) = Self::try_parse_binary(s, &['*', '/']) {
        return Ok(expr);
    }
    if let Some(expr) = Self::try_parse_binary(s, &['^']) {
        return Ok(expr);
    }

    // Handle parentheses
    if s.starts_with('(') && s.ends_with(')') {
        return Self::parse(&s[1..s.len() - 1]);
    }

    // Handle function calls
    if let Some(paren_idx) = s.find('(') {
        if s.ends_with(')') && paren_idx > 0 {
            let func_name = s[..paren_idx].trim();
            if !func_name.is_empty() {
                let args_str = &s[paren_idx + 1..s.len() - 1];
                let arg_strings = Self::split_function_args(args_str);
                let args: Result<Vec<_>, _> = arg_strings
                    .iter()
                    .map(|arg| Self::parse(arg.trim()))
                    .collect();
                return Ok(Expression::FunctionCall {
                    name: func_name.to_string(),
                    args: args?,
                });
            }
        }
    }

    // Handle unary minus
    if s.starts_with('-') {
        let inner = Self::parse(&s[1..])?;
        return Ok(Expression::UnaryOp {
            op: UnaryOperator::Negate,
            expr: Box::new(inner),
        });
    }

    // Treat as variable name
    Ok(Expression::Variable(s.to_string()))
}
```

## 5. EXPRESSION EVALUATION CORE

**File**: `src/model/expression.rs:437-491`

```rust
pub fn evaluate(&self, context: &EvaluationContext) -> Result<f64, String> {
    match self {
        Expression::Constant(val) => Ok(*val),

        Expression::Variable(name) => {
            context.get_variable(name)
        }

        Expression::BinaryOp { op, left, right } => {
            let left_val = left.evaluate(context)?;
            let right_val = right.evaluate(context)?;

            Ok(match op {
                Operator::Add => left_val + right_val,
                Operator::Subtract => left_val - right_val,
                Operator::Multiply => left_val * right_val,
                Operator::Divide => {
                    if right_val == 0.0 {
                        return Err("Division by zero".to_string());
                    }
                    left_val / right_val
                }
                Operator::Power => left_val.powf(right_val),
                Operator::GreaterThan => if left_val > right_val { 1.0 } else { 0.0 },
                Operator::LessThan => if left_val < right_val { 1.0 } else { 0.0 },
                Operator::GreaterEqual => if left_val >= right_val { 1.0 } else { 0.0 },
                Operator::LessEqual => if left_val <= right_val { 1.0 } else { 0.0 },
                Operator::Equal => if (left_val - right_val).abs() < 1e-10 { 1.0 } else { 0.0 },
                Operator::NotEqual => if (left_val - right_val).abs() >= 1e-10 { 1.0 } else { 0.0 },
            })
        }

        Expression::UnaryOp { op, expr } => {
            let val = expr.evaluate(context)?;
            Ok(match op {
                UnaryOperator::Negate => -val,
            })
        }

        Expression::FunctionCall { name, args } => {
            Self::evaluate_function(name, args, context)
        }

        Expression::Conditional { condition, true_expr, false_expr } => {
            let cond_val = condition.evaluate(context)?;
            if cond_val > 0.5 {
                true_expr.evaluate(context)
            } else {
                false_expr.evaluate(context)
            }
        }
    }
}

fn evaluate_function(
    name: &str, 
    args: &[Expression], 
    context: &EvaluationContext
) -> Result<f64, String> 
{
    let arg_values: Result<Vec<f64>, String> = args
        .iter()
        .map(|arg| arg.evaluate(context))
        .collect();
    let arg_values = arg_values?;

    match name.to_uppercase().as_str() {
        "MIN" => Ok(arg_values.iter().copied().fold(f64::INFINITY, f64::min)),
        "MAX" => Ok(arg_values.iter().copied().fold(f64::NEG_INFINITY, f64::max)),
        "ABS" => {
            if arg_values.len() != 1 {
                return Err(format!("ABS expects 1 argument, got {}", arg_values.len()));
            }
            Ok(arg_values[0].abs())
        }
        "SQRT" => {
            if arg_values.len() != 1 {
                return Err(format!("SQRT expects 1 argument, got {}", arg_values.len()));
            }
            Ok(arg_values[0].sqrt())
        }
        "EXP" => {
            if arg_values.len() != 1 {
                return Err(format!("EXP expects 1 argument, got {}", arg_values.len()));
            }
            Ok(arg_values[0].exp())
        }
        "LN" => {
            if arg_values.len() != 1 {
                return Err(format!("LN expects 1 argument, got {}", arg_values.len()));
            }
            Ok(arg_values[0].ln())
        }
        "SIN" => {
            if arg_values.len() != 1 {
                return Err(format!("SIN expects 1 argument, got {}", arg_values.len()));
            }
            Ok(arg_values[0].sin())
        }
        "COS" => {
            if arg_values.len() != 1 {
                return Err(format!("COS expects 1 argument, got {}", arg_values.len()));
            }
            Ok(arg_values[0].cos())
        }
        "TIME" => Ok(context.time),
        _ => Err(format!("Unknown function: '{}'", name))
    }
}
```

## 6. SIMULATION STATE INITIALIZATION

**File**: `src/simulation/mod.rs:31-56`

```rust
pub fn initialize_from_model(model: &Model) -> Result<Self, String> {
    let mut state = Self::new();
    state.time = model.time.start;

    // Initialize stocks with their initial values
    for (name, stock) in &model.stocks {
        let initial_value = stock.initial.evaluate(&crate::model::expression::EvaluationContext {
            model,
            state: &state,
            time: model.time.start,
        })?;
        state.stocks.insert(name.clone(), initial_value);
    }

    // Initialize flows to zero
    for name in model.flows.keys() {
        state.flows.insert(name.clone(), 0.0);
    }

    // Initialize auxiliaries
    for name in model.auxiliaries.keys() {
        state.auxiliaries.insert(name.clone(), 0.0);
    }

    Ok(state)
}
```

## 7. SIMULATION ENGINE RUN LOOP

**File**: `src/simulation/engine.rs:25-67`

```rust
pub fn run(&mut self) -> Result<SimulationResults, String> {
    let mut results = SimulationResults::new();
    results.add_point(self.state.time, self.state.clone());

    let dt = self.model.time.dt;
    let stop_time = self.model.time.stop;

    // Create integrator based on config
    let integrator: Box<dyn Integrator> = match self.config.integration_method {
        IntegrationMethod::Euler => Box::new(EulerIntegrator),
        IntegrationMethod::RK4 => Box::new(RK4Integrator),
    };

    // Main simulation loop
    while self.state.time < stop_time {
        // Take a step
        self.state = integrator.step(&self.model, &self.state, dt)?;

        // Ensure we don't overshoot
        if self.state.time > stop_time {
            self.state.time = stop_time;
        }

        // Record state based on output interval
        let should_record = if let Some(interval) = self.config.output_interval {
            let current_interval = (self.state.time / interval).floor();
            let prev_interval = ((self.state.time - dt) / interval).floor();
            current_interval > prev_interval
        } else {
            true  // Record every step
        };

        if should_record {
            results.add_point(self.state.time, self.state.clone());
        }
    }

    Ok(results)
}
```

## 8. VARIABLE RESOLUTION PRIORITY

**File**: `src/model/mod.rs:109-132`

```rust
pub fn get_variable(&self, name: &str, state: &crate::simulation::SimulationState) 
    -> Result<f64, String> 
{
    // Try parameter first
    if let Some(param) = self.parameters.get(name) {
        return Ok(param.value);
    }

    // Try stock
    if let Some(value) = state.stocks.get(name) {
        return Ok(*value);
    }

    // Try flow
    if let Some(value) = state.flows.get(name) {
        return Ok(*value);
    }

    // Try auxiliary
    if let Some(value) = state.auxiliaries.get(name) {
        return Ok(*value);
    }

    Err(format!("Variable '{}' not found", name))
}
```

## 9. CLI COMMAND PARSING

**File**: `src/main.rs:19-60`

```rust
#[derive(Parser)]
#[command(name = "rsedsim")]
#[command(about = "Rust System Dynamics Simulator", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a simulation
    Run {
        /// Model file (JSON or YAML)
        model: PathBuf,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Override parameters (format: "param1=value1,param2=value2")
        #[arg(short, long)]
        params: Option<String>,

        /// Integration method (euler or rk4)
        #[arg(long, default_value = "euler")]
        integrator: String,

        /// Override timestep (dt)
        #[arg(long)]
        dt: Option<f64>,
    },

    /// Validate a model file
    Validate {
        /// Model file to validate
        model: PathBuf,
    },

    /// Show version and info
    Info,
}
```

## 10. MODEL STRUCTURE DEFINITIONS

**File**: `src/model/mod.rs:18-59`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeConfig {
    pub start: f64,
    pub stop: f64,
    pub dt: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub units: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    #[serde(default)]
    pub metadata: ModelMetadata,
    pub time: TimeConfig,
    pub stocks: HashMap<String, Stock>,
    pub flows: HashMap<String, Flow>,
    pub auxiliaries: HashMap<String, Auxiliary>,
    pub parameters: HashMap<String, Parameter>,
}
```

