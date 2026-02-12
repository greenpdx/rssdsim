/// Integration methods for numerical simulation

use std::collections::HashMap;
use crate::model::Model;
use crate::model::expression::EvaluationContext;
use super::SimulationState;

pub trait Integrator {
    fn step(&self, model: &Model, state: &SimulationState, dt: f64) -> Result<SimulationState, String>;
}

/// Euler (forward) integration method
pub struct EulerIntegrator;

impl Integrator for EulerIntegrator {
    fn step(&self, model: &Model, state: &SimulationState, dt: f64) -> Result<SimulationState, String> {
        let mut new_state = state.clone();
        new_state.time += dt;

        // 1. Evaluate auxiliaries (in dependency order - use fixed-point iteration)
        // Since we don't have dependency sorting, evaluate multiple passes until stable
        let mut new_auxiliaries = HashMap::new();
        const MAX_PASSES: usize = 20;  // Prevent infinite loops (complex models need more passes)

        for pass in 0..MAX_PASSES {
            let mut changed = false;
            let mut any_errors = false;

            for (name, aux) in &model.auxiliaries {
                let mut temp_state = new_state.clone();
                let mut context_with_aux = EvaluationContext::new(model, &mut temp_state, state.time);

                match aux.equation.evaluate(&mut context_with_aux) {
                    Ok(value) => {
                        // Check if value changed
                        if let Some(&old_value) = new_auxiliaries.get(name) {
                            let diff: f64 = value - old_value;
                            if diff.abs() > 1e-10 {
                                changed = true;
                            }
                        } else {
                            changed = true; // New value added
                        }
                        new_auxiliaries.insert(name.clone(), value);

                        // Merge back state changes
                        new_state.delays = temp_state.delays;
                        new_state.stochastic = temp_state.stochastic;
                        new_state.agents = temp_state.agents;
                    }
                    Err(e) => {
                        // On first few passes, errors are expected (missing dependencies)
                        // On later passes, if we still have errors, fail
                        if pass >= 5 {
                            return Err(format!("Error evaluating auxiliary '{}' (pass {}): {}", name, pass + 1, e));
                        }
                        any_errors = true;
                    }
                }
            }
            new_state.auxiliaries = new_auxiliaries.clone();

            // Converged if nothing changed and no errors
            if !changed && !any_errors && pass > 0 {
                break;
            }
        }
        new_state.auxiliaries = new_auxiliaries;

        // 2. Evaluate flows
        let mut new_flows = HashMap::new();
        for (name, flow) in &model.flows {
            let mut temp_state = new_state.clone();
            let mut context = EvaluationContext::new(model, &mut temp_state, state.time);

            let value = flow.equation.evaluate(&mut context)
                .map_err(|e| format!("Error evaluating flow '{}': {}", name, e))?;
            new_flows.insert(name.clone(), value);

            // Merge back delay, stochastic, and agent state
            new_state.delays = temp_state.delays;
            new_state.stochastic = temp_state.stochastic;
            new_state.agents = temp_state.agents;
        }
        new_state.flows = new_flows;

        // 3. Update stocks using d(stock)/dt = inflows - outflows
        let mut stock_derivatives: HashMap<String, f64> = HashMap::new();

        for (stock_name, stock) in &model.stocks {
            let mut derivative = 0.0;

            // Add inflows
            for inflow_name in &stock.inflows {
                if let Some(flow_value) = new_state.flows.get(inflow_name) {
                    derivative += flow_value;
                } else {
                    return Err(format!("Inflow '{}' not found for stock '{}'", inflow_name, stock_name));
                }
            }

            // Subtract outflows
            for outflow_name in &stock.outflows {
                if let Some(flow_value) = new_state.flows.get(outflow_name) {
                    derivative -= flow_value;
                } else {
                    return Err(format!("Outflow '{}' not found for stock '{}'", outflow_name, stock_name));
                }
            }

            stock_derivatives.insert(stock_name.clone(), derivative);
        }

        // Apply Euler integration: stock(t+dt) = stock(t) + derivative * dt
        for (stock_name, current_value) in &state.stocks {
            if let Some(derivative) = stock_derivatives.get(stock_name) {
                let new_value = current_value + derivative * dt;

                // Enforce non-negative and max_value constraints if specified
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

        // Update exponential delays
        let mut delay_inputs = HashMap::new();
        for (key, delay) in &new_state.delays.exponential_delays {
            // The input is stored in the delay itself from evaluation
            // For now, we'll update delays based on their current values
            delay_inputs.insert(key.clone(), delay.value);
        }
        new_state.delays.update_all_exponential(&delay_inputs, dt);

        // Update pipeline delays
        for pipeline in new_state.delays.pipeline_delays.values_mut() {
            pipeline.push(new_state.time, 0.0); // Placeholder - need to track actual inputs
        }

        Ok(new_state)
    }
}

/// RK4 (Runge-Kutta 4th order) integration method
pub struct RK4Integrator;

impl RK4Integrator {
    /// Evaluate auxiliaries and flows at a given state
    fn evaluate_system(
        &self,
        model: &Model,
        state: &SimulationState,
        time: f64,
    ) -> Result<(HashMap<String, f64>, HashMap<String, f64>), String> {
        // 1. Evaluate auxiliaries with fixed-point iteration
        // Start with existing auxiliary values from state
        let mut auxiliaries = state.auxiliaries.clone();
        const MAX_PASSES: usize = 20;

        for pass in 0..MAX_PASSES {
            let mut changed = false;
            let mut temp_state = state.clone();
            temp_state.auxiliaries = auxiliaries.clone();
            let mut any_errors = false;

            for (name, aux) in &model.auxiliaries {
                let mut eval_state = temp_state.clone();
                let mut context = EvaluationContext::new(model, &mut eval_state, time);

                match aux.equation.evaluate(&mut context) {
                    Ok(value) => {
                        if let Some(&old_value) = auxiliaries.get(name) {
                            let diff: f64 = value - old_value;
                            if diff.abs() > 1e-10 {
                                changed = true;
                            }
                        } else {
                            changed = true;
                        }
                        auxiliaries.insert(name.clone(), value);
                    }
                    Err(e) => {
                        if pass >= 5 {
                            return Err(format!("Error evaluating auxiliary '{}' (pass {}): {}", name, pass + 1, e));
                        }
                        any_errors = true;
                    }
                }
            }

            if !changed && !any_errors && pass > 0 {
                break;
            }
        }

        // 2. Evaluate flows
        let mut eval_state = state.clone();
        eval_state.auxiliaries = auxiliaries.clone();

        let mut flows = HashMap::new();
        for (name, flow) in &model.flows {
            let mut temp_state = eval_state.clone();
            let mut context = EvaluationContext::new(model, &mut temp_state, time);

            let value = flow.equation.evaluate(&mut context)
                .map_err(|e| format!("Error evaluating flow '{}': {}", name, e))?;
            flows.insert(name.clone(), value);
        }

        Ok((auxiliaries, flows))
    }

    /// Compute derivatives (inflows - outflows) for all stocks
    fn compute_derivatives(
        &self,
        model: &Model,
        flows: &HashMap<String, f64>,
    ) -> Result<HashMap<String, f64>, String> {
        let mut derivatives = HashMap::new();

        for (stock_name, stock) in &model.stocks {
            let mut derivative = 0.0;

            // Add inflows
            for inflow_name in &stock.inflows {
                if let Some(flow_value) = flows.get(inflow_name) {
                    derivative += flow_value;
                } else {
                    return Err(format!("Inflow '{}' not found for stock '{}'", inflow_name, stock_name));
                }
            }

            // Subtract outflows
            for outflow_name in &stock.outflows {
                if let Some(flow_value) = flows.get(outflow_name) {
                    derivative -= flow_value;
                } else {
                    return Err(format!("Outflow '{}' not found for stock '{}'", outflow_name, stock_name));
                }
            }

            derivatives.insert(stock_name.clone(), derivative);
        }

        Ok(derivatives)
    }

    /// Create a new state with modified stock values
    /// Preserves auxiliaries and flows from base state as starting point
    fn apply_stock_increments(
        &self,
        base_state: &SimulationState,
        increments: &HashMap<String, f64>,
    ) -> SimulationState {
        let mut new_state = base_state.clone();
        // Update stocks with increments
        for (stock_name, increment) in increments {
            if let Some(&current_value) = base_state.stocks.get(stock_name) {
                new_state.stocks.insert(stock_name.clone(), current_value + increment);
            }
        }
        // Preserve auxiliaries and flows from base state
        // They will be updated by evaluate_system
        new_state
    }
}

impl Integrator for RK4Integrator {
    fn step(&self, model: &Model, state: &SimulationState, dt: f64) -> Result<SimulationState, String> {
        // RK4 algorithm: y_{n+1} = y_n + (k1 + 2*k2 + 2*k3 + k4) * dt / 6
        // where:
        //   k1 = f(t_n, y_n)
        //   k2 = f(t_n + dt/2, y_n + k1*dt/2)
        //   k3 = f(t_n + dt/2, y_n + k2*dt/2)
        //   k4 = f(t_n + dt, y_n + k3*dt)

        let t = state.time;

        // Stage 1: k1 = f(t, y)
        let (aux1, flows1) = self.evaluate_system(model, state, t)?;
        let k1 = self.compute_derivatives(model, &flows1)?;

        // Stage 2: k2 = f(t + dt/2, y + k1*dt/2)
        let increments2: HashMap<String, f64> = k1.iter()
            .map(|(name, &deriv)| (name.clone(), deriv * dt / 2.0))
            .collect();
        let state2 = self.apply_stock_increments(state, &increments2);
        let (aux2, flows2) = self.evaluate_system(model, &state2, t + dt / 2.0)?;
        let k2 = self.compute_derivatives(model, &flows2)?;

        // Stage 3: k3 = f(t + dt/2, y + k2*dt/2)
        let increments3: HashMap<String, f64> = k2.iter()
            .map(|(name, &deriv)| (name.clone(), deriv * dt / 2.0))
            .collect();
        let state3 = self.apply_stock_increments(state, &increments3);
        let (aux3, flows3) = self.evaluate_system(model, &state3, t + dt / 2.0)?;
        let k3 = self.compute_derivatives(model, &flows3)?;

        // Stage 4: k4 = f(t + dt, y + k3*dt)
        let increments4: HashMap<String, f64> = k3.iter()
            .map(|(name, &deriv)| (name.clone(), deriv * dt))
            .collect();
        let state4 = self.apply_stock_increments(state, &increments4);
        let (aux4, flows4) = self.evaluate_system(model, &state4, t + dt)?;
        let k4 = self.compute_derivatives(model, &flows4)?;

        // Combine stages with RK4 weights
        let mut new_state = state.clone();
        new_state.time += dt;

        // Update stocks: y_new = y + (k1 + 2*k2 + 2*k3 + k4) * dt / 6
        for (stock_name, &current_value) in &state.stocks {
            let d1 = k1.get(stock_name).unwrap_or(&0.0);
            let d2 = k2.get(stock_name).unwrap_or(&0.0);
            let d3 = k3.get(stock_name).unwrap_or(&0.0);
            let d4 = k4.get(stock_name).unwrap_or(&0.0);

            let new_value = current_value + (d1 + 2.0 * d2 + 2.0 * d3 + d4) * dt / 6.0;

            // Enforce non-negative and max_value constraints if specified
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

        // Use final stage values for auxiliaries and flows
        new_state.auxiliaries = aux4;
        new_state.flows = flows4;

        Ok(new_state)
    }
}

/// Heun's method (Improved Euler / RK2)
/// A second-order predictor-corrector method
/// More accurate than Euler but less expensive than RK4
pub struct HeunIntegrator;

impl HeunIntegrator {
    /// Evaluate auxiliaries and flows at a given state
    fn evaluate_system(
        &self,
        model: &Model,
        state: &SimulationState,
        time: f64,
    ) -> Result<(HashMap<String, f64>, HashMap<String, f64>), String> {
        // Evaluate auxiliaries with fixed-point iteration
        let mut auxiliaries = state.auxiliaries.clone();
        const MAX_PASSES: usize = 20;

        for pass in 0..MAX_PASSES {
            let mut changed = false;
            let mut temp_state = state.clone();
            temp_state.auxiliaries = auxiliaries.clone();
            let mut any_errors = false;

            for (name, aux) in &model.auxiliaries {
                let mut eval_state = temp_state.clone();
                let mut context = EvaluationContext::new(model, &mut eval_state, time);

                match aux.equation.evaluate(&mut context) {
                    Ok(value) => {
                        if let Some(&old_value) = auxiliaries.get(name) {
                            let diff: f64 = value - old_value;
                            if diff.abs() > 1e-10 {
                                changed = true;
                            }
                        } else {
                            changed = true;
                        }
                        auxiliaries.insert(name.clone(), value);
                    }
                    Err(e) => {
                        if pass >= 5 {
                            return Err(format!("Error evaluating auxiliary '{}' (pass {}): {}", name, pass + 1, e));
                        }
                        any_errors = true;
                    }
                }
            }

            if !changed && !any_errors && pass > 0 {
                break;
            }
        }

        // Evaluate flows
        let mut eval_state = state.clone();
        eval_state.auxiliaries = auxiliaries.clone();

        let mut flows = HashMap::new();
        for (name, flow) in &model.flows {
            let mut temp_state = eval_state.clone();
            let mut context = EvaluationContext::new(model, &mut temp_state, time);

            let value = flow.equation.evaluate(&mut context)
                .map_err(|e| format!("Error evaluating flow '{}': {}", name, e))?;
            flows.insert(name.clone(), value);
        }

        Ok((auxiliaries, flows))
    }

    fn compute_derivatives(
        &self,
        model: &Model,
        flows: &HashMap<String, f64>,
    ) -> Result<HashMap<String, f64>, String> {
        let mut derivatives = HashMap::new();

        for (stock_name, stock) in &model.stocks {
            let mut derivative = 0.0;

            for inflow_name in &stock.inflows {
                if let Some(flow_value) = flows.get(inflow_name) {
                    derivative += flow_value;
                } else {
                    return Err(format!("Inflow '{}' not found for stock '{}'", inflow_name, stock_name));
                }
            }

            for outflow_name in &stock.outflows {
                if let Some(flow_value) = flows.get(outflow_name) {
                    derivative -= flow_value;
                } else {
                    return Err(format!("Outflow '{}' not found for stock '{}'", outflow_name, stock_name));
                }
            }

            derivatives.insert(stock_name.clone(), derivative);
        }

        Ok(derivatives)
    }

    fn apply_stock_increments(
        &self,
        base_state: &SimulationState,
        increments: &HashMap<String, f64>,
    ) -> SimulationState {
        let mut new_state = base_state.clone();
        for (stock_name, increment) in increments {
            if let Some(&current_value) = base_state.stocks.get(stock_name) {
                new_state.stocks.insert(stock_name.clone(), current_value + increment);
            }
        }
        new_state
    }
}

impl Integrator for HeunIntegrator {
    fn step(&self, model: &Model, state: &SimulationState, dt: f64) -> Result<SimulationState, String> {
        // Heun's method (predictor-corrector):
        // 1. Predictor: y_pred = y_n + f(t_n, y_n) * dt
        // 2. Corrector: y_{n+1} = y_n + [f(t_n, y_n) + f(t_{n+1}, y_pred)] * dt / 2

        let t = state.time;

        // Predictor step: evaluate at current state
        let (aux1, flows1) = self.evaluate_system(model, state, t)?;
        let k1 = self.compute_derivatives(model, &flows1)?;

        // Predicted state: y_pred = y + k1 * dt
        let increments_pred: HashMap<String, f64> = k1.iter()
            .map(|(name, &deriv)| (name.clone(), deriv * dt))
            .collect();
        let state_pred = self.apply_stock_increments(state, &increments_pred);

        // Corrector step: evaluate at predicted state
        let (aux2, flows2) = self.evaluate_system(model, &state_pred, t + dt)?;
        let k2 = self.compute_derivatives(model, &flows2)?;

        // Final update: y_new = y + (k1 + k2) * dt / 2
        let mut new_state = state.clone();
        new_state.time += dt;

        for (stock_name, &current_value) in &state.stocks {
            let d1 = k1.get(stock_name).unwrap_or(&0.0);
            let d2 = k2.get(stock_name).unwrap_or(&0.0);

            let new_value = current_value + (d1 + d2) * dt / 2.0;

            // Enforce constraints
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

        // Use corrector values for auxiliaries and flows
        new_state.auxiliaries = aux2;
        new_state.flows = flows2;

        Ok(new_state)
    }
}

/// Backward Euler (Implicit Euler)
/// An implicit method that is more stable for stiff systems
/// Uses fixed-point iteration to solve the implicit equation
pub struct BackwardEulerIntegrator {
    /// Maximum iterations for implicit solve
    pub max_iterations: usize,
    /// Convergence tolerance
    pub tolerance: f64,
}

impl Default for BackwardEulerIntegrator {
    fn default() -> Self {
        Self {
            max_iterations: 20,
            tolerance: 1e-6,
        }
    }
}

impl BackwardEulerIntegrator {
    pub fn new(max_iterations: usize, tolerance: f64) -> Self {
        Self {
            max_iterations,
            tolerance,
        }
    }

    fn evaluate_system(
        &self,
        model: &Model,
        state: &SimulationState,
        time: f64,
    ) -> Result<(HashMap<String, f64>, HashMap<String, f64>), String> {
        let mut auxiliaries = state.auxiliaries.clone();
        const MAX_PASSES: usize = 20;

        for pass in 0..MAX_PASSES {
            let mut changed = false;
            let mut temp_state = state.clone();
            temp_state.auxiliaries = auxiliaries.clone();
            let mut any_errors = false;

            for (name, aux) in &model.auxiliaries {
                let mut eval_state = temp_state.clone();
                let mut context = EvaluationContext::new(model, &mut eval_state, time);

                match aux.equation.evaluate(&mut context) {
                    Ok(value) => {
                        if let Some(&old_value) = auxiliaries.get(name) {
                            let diff: f64 = value - old_value;
                            if diff.abs() > 1e-10 {
                                changed = true;
                            }
                        } else {
                            changed = true;
                        }
                        auxiliaries.insert(name.clone(), value);
                    }
                    Err(e) => {
                        if pass >= 5 {
                            return Err(format!("Error evaluating auxiliary '{}' (pass {}): {}", name, pass + 1, e));
                        }
                        any_errors = true;
                    }
                }
            }

            if !changed && !any_errors && pass > 0 {
                break;
            }
        }

        let mut eval_state = state.clone();
        eval_state.auxiliaries = auxiliaries.clone();

        let mut flows = HashMap::new();
        for (name, flow) in &model.flows {
            let mut temp_state = eval_state.clone();
            let mut context = EvaluationContext::new(model, &mut temp_state, time);

            let value = flow.equation.evaluate(&mut context)
                .map_err(|e| format!("Error evaluating flow '{}': {}", name, e))?;
            flows.insert(name.clone(), value);
        }

        Ok((auxiliaries, flows))
    }

    fn compute_derivatives(
        &self,
        model: &Model,
        flows: &HashMap<String, f64>,
    ) -> Result<HashMap<String, f64>, String> {
        let mut derivatives = HashMap::new();

        for (stock_name, stock) in &model.stocks {
            let mut derivative = 0.0;

            for inflow_name in &stock.inflows {
                if let Some(flow_value) = flows.get(inflow_name) {
                    derivative += flow_value;
                } else {
                    return Err(format!("Inflow '{}' not found for stock '{}'", inflow_name, stock_name));
                }
            }

            for outflow_name in &stock.outflows {
                if let Some(flow_value) = flows.get(outflow_name) {
                    derivative -= flow_value;
                } else {
                    return Err(format!("Outflow '{}' not found for stock '{}'", outflow_name, stock_name));
                }
            }

            derivatives.insert(stock_name.clone(), derivative);
        }

        Ok(derivatives)
    }
}

impl Integrator for BackwardEulerIntegrator {
    fn step(&self, model: &Model, state: &SimulationState, dt: f64) -> Result<SimulationState, String> {
        // Backward Euler: y_{n+1} = y_n + f(t_{n+1}, y_{n+1}) * dt
        // This is implicit, so we solve using fixed-point iteration:
        // y^{k+1} = y_n + f(t_{n+1}, y^k) * dt

        let t_next = state.time + dt;

        // Initial guess: use forward Euler
        let (_, flows0) = self.evaluate_system(model, state, state.time)?;
        let deriv0 = self.compute_derivatives(model, &flows0)?;

        let mut current_state = state.clone();
        current_state.time = t_next;

        // Apply initial Euler step as starting guess
        for (stock_name, &current_value) in &state.stocks {
            if let Some(&derivative) = deriv0.get(stock_name) {
                current_state.stocks.insert(stock_name.clone(), current_value + derivative * dt);
            }
        }

        // Fixed-point iteration
        for iteration in 0..self.max_iterations {
            // Evaluate system at current estimate
            let (auxiliaries, flows) = self.evaluate_system(model, &current_state, t_next)?;
            let derivatives = self.compute_derivatives(model, &flows)?;

            // Compute new estimate: y_new = y_old + f(t_next, y_current) * dt
            let mut max_change: f64 = 0.0;
            let mut next_state = current_state.clone();

            for (stock_name, &old_value) in &state.stocks {
                if let Some(&derivative) = derivatives.get(stock_name) {
                    let new_value = old_value + derivative * dt;

                    // Enforce constraints
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

                    // Track convergence
                    if let Some(&current_val) = current_state.stocks.get(stock_name) {
                        let change = (constrained_value - current_val).abs();
                        max_change = max_change.max(change);
                    }

                    next_state.stocks.insert(stock_name.clone(), constrained_value);
                }
            }

            next_state.auxiliaries = auxiliaries;
            next_state.flows = flows;

            // Check convergence
            if max_change < self.tolerance && iteration > 0 {
                return Ok(next_state);
            }

            current_state = next_state;
        }

        // If we didn't converge, return best estimate with warning
        eprintln!("Warning: Backward Euler did not converge after {} iterations", self.max_iterations);
        Ok(current_state)
    }
}

/// Dormand-Prince RK45 adaptive integrator
/// A 5th order Runge-Kutta method with 4th order error estimation
/// Automatically adjusts step size based on error tolerance
pub struct RK45Integrator {
    /// Relative error tolerance
    pub rtol: f64,
    /// Absolute error tolerance
    pub atol: f64,
    /// Minimum allowed step size
    pub min_step: f64,
    /// Maximum allowed step size
    pub max_step: f64,
    /// Safety factor for step size adjustment
    pub safety_factor: f64,
}

impl Default for RK45Integrator {
    fn default() -> Self {
        Self {
            rtol: 1e-6,
            atol: 1e-8,
            min_step: 1e-10,
            max_step: 1.0,
            safety_factor: 0.9,
        }
    }
}

impl RK45Integrator {
    pub fn new(rtol: f64, atol: f64) -> Self {
        Self {
            rtol,
            atol,
            ..Default::default()
        }
    }

    pub fn with_step_limits(mut self, min_step: f64, max_step: f64) -> Self {
        self.min_step = min_step;
        self.max_step = max_step;
        self
    }

    /// Evaluate system at a given state
    fn evaluate_system(
        &self,
        model: &Model,
        state: &SimulationState,
        time: f64,
    ) -> Result<(HashMap<String, f64>, HashMap<String, f64>), String> {
        // Evaluate auxiliaries with fixed-point iteration
        let mut auxiliaries = state.auxiliaries.clone();
        const MAX_PASSES: usize = 20;

        for pass in 0..MAX_PASSES {
            let mut changed = false;
            let mut temp_state = state.clone();
            temp_state.auxiliaries = auxiliaries.clone();
            let mut any_errors = false;

            for (name, aux) in &model.auxiliaries {
                let mut eval_state = temp_state.clone();
                let mut context = EvaluationContext::new(model, &mut eval_state, time);

                match aux.equation.evaluate(&mut context) {
                    Ok(value) => {
                        if let Some(&old_value) = auxiliaries.get(name) {
                            let diff: f64 = value - old_value;
                            if diff.abs() > 1e-10 {
                                changed = true;
                            }
                        } else {
                            changed = true;
                        }
                        auxiliaries.insert(name.clone(), value);
                    }
                    Err(e) => {
                        if pass >= 5 {
                            return Err(format!("Error evaluating auxiliary '{}' (pass {}): {}", name, pass + 1, e));
                        }
                        any_errors = true;
                    }
                }
            }

            if !changed && !any_errors && pass > 0 {
                break;
            }
        }

        // Evaluate flows
        let mut eval_state = state.clone();
        eval_state.auxiliaries = auxiliaries.clone();

        let mut flows = HashMap::new();
        for (name, flow) in &model.flows {
            let mut temp_state = eval_state.clone();
            let mut context = EvaluationContext::new(model, &mut temp_state, time);

            let value = flow.equation.evaluate(&mut context)
                .map_err(|e| format!("Error evaluating flow '{}': {}", name, e))?;
            flows.insert(name.clone(), value);
        }

        Ok((auxiliaries, flows))
    }

    /// Compute derivatives (inflows - outflows) for all stocks
    fn compute_derivatives(
        &self,
        model: &Model,
        flows: &HashMap<String, f64>,
    ) -> Result<HashMap<String, f64>, String> {
        let mut derivatives = HashMap::new();

        for (stock_name, stock) in &model.stocks {
            let mut derivative = 0.0;

            for inflow_name in &stock.inflows {
                if let Some(flow_value) = flows.get(inflow_name) {
                    derivative += flow_value;
                } else {
                    return Err(format!("Inflow '{}' not found for stock '{}'", inflow_name, stock_name));
                }
            }

            for outflow_name in &stock.outflows {
                if let Some(flow_value) = flows.get(outflow_name) {
                    derivative -= flow_value;
                } else {
                    return Err(format!("Outflow '{}' not found for stock '{}'", outflow_name, stock_name));
                }
            }

            derivatives.insert(stock_name.clone(), derivative);
        }

        Ok(derivatives)
    }

    /// Apply stock increments to create new state
    fn apply_stock_increments(
        &self,
        base_state: &SimulationState,
        increments: &HashMap<String, f64>,
    ) -> SimulationState {
        let mut new_state = base_state.clone();
        for (stock_name, increment) in increments {
            if let Some(&current_value) = base_state.stocks.get(stock_name) {
                new_state.stocks.insert(stock_name.clone(), current_value + increment);
            }
        }
        new_state
    }

    /// Compute error estimate and optimal step size
    fn compute_error_and_step(
        &self,
        y4: &HashMap<String, f64>,
        y5: &HashMap<String, f64>,
        current_step: f64,
    ) -> (f64, f64) {
        let mut max_error: f64 = 0.0;

        for (name, &val4) in y4 {
            if let Some(&val5) = y5.get(name) {
                let error = (val5 - val4).abs();
                let scale = self.atol + self.rtol * val5.abs().max(val4.abs());
                let normalized_error = error / scale;
                max_error = max_error.max(normalized_error);
            }
        }

        // Compute new step size
        let new_step: f64 = if max_error > 0.0 {
            self.safety_factor * current_step * (1.0 / max_error).powf(0.2)
        } else {
            current_step * 2.0
        };

        let new_step = new_step.max(self.min_step).min(self.max_step);

        (max_error, new_step)
    }
}

impl Integrator for RK45Integrator {
    fn step(&self, model: &Model, state: &SimulationState, dt: f64) -> Result<SimulationState, String> {
        // Dormand-Prince coefficients
        // Butcher tableau for DOPRI5
        let a21 = 1.0 / 5.0;
        let a31 = 3.0 / 40.0;
        let a32 = 9.0 / 40.0;
        let a41 = 44.0 / 45.0;
        let a42 = -56.0 / 15.0;
        let a43 = 32.0 / 9.0;
        let a51 = 19372.0 / 6561.0;
        let a52 = -25360.0 / 2187.0;
        let a53 = 64448.0 / 6561.0;
        let a54 = -212.0 / 729.0;
        let a61 = 9017.0 / 3168.0;
        let a62 = -355.0 / 33.0;
        let a63 = 46732.0 / 5247.0;
        let a64 = 49.0 / 176.0;
        let a65 = -5103.0 / 18656.0;
        let a71 = 35.0 / 384.0;
        let a73 = 500.0 / 1113.0;
        let a74 = 125.0 / 192.0;
        let a75 = -2187.0 / 6784.0;
        let a76 = 11.0 / 84.0;

        // 5th order weights
        let b1 = 35.0 / 384.0;
        let b3 = 500.0 / 1113.0;
        let b4 = 125.0 / 192.0;
        let b5 = -2187.0 / 6784.0;
        let b6 = 11.0 / 84.0;

        // 4th order weights for error estimation
        let b1_star = 5179.0 / 57600.0;
        let b3_star = 7571.0 / 16695.0;
        let b4_star = 393.0 / 640.0;
        let b5_star = -92097.0 / 339200.0;
        let b6_star = 187.0 / 2100.0;
        let b7_star = 1.0 / 40.0;

        let t = state.time;
        let mut h = dt.min(self.max_step);
        const MAX_ATTEMPTS: usize = 10;

        for _attempt in 0..MAX_ATTEMPTS {
            // Stage 1: k1 = f(t, y)
            let (_, flows1) = self.evaluate_system(model, state, t)?;
            let k1 = self.compute_derivatives(model, &flows1)?;

            // Stage 2
            let inc2: HashMap<String, f64> = k1.iter()
                .map(|(name, &d)| (name.clone(), a21 * d * h))
                .collect();
            let state2 = self.apply_stock_increments(state, &inc2);
            let (_, flows2) = self.evaluate_system(model, &state2, t + h / 5.0)?;
            let k2 = self.compute_derivatives(model, &flows2)?;

            // Stage 3
            let inc3: HashMap<String, f64> = k1.iter()
                .map(|(name, &d1)| {
                    let d2 = k2.get(name).unwrap_or(&0.0);
                    (name.clone(), (a31 * d1 + a32 * d2) * h)
                })
                .collect();
            let state3 = self.apply_stock_increments(state, &inc3);
            let (_, flows3) = self.evaluate_system(model, &state3, t + 3.0 * h / 10.0)?;
            let k3 = self.compute_derivatives(model, &flows3)?;

            // Stage 4
            let inc4: HashMap<String, f64> = k1.iter()
                .map(|(name, &d1)| {
                    let d2 = k2.get(name).unwrap_or(&0.0);
                    let d3 = k3.get(name).unwrap_or(&0.0);
                    (name.clone(), (a41 * d1 + a42 * d2 + a43 * d3) * h)
                })
                .collect();
            let state4 = self.apply_stock_increments(state, &inc4);
            let (_, flows4) = self.evaluate_system(model, &state4, t + 4.0 * h / 5.0)?;
            let k4 = self.compute_derivatives(model, &flows4)?;

            // Stage 5
            let inc5: HashMap<String, f64> = k1.iter()
                .map(|(name, &d1)| {
                    let d2 = k2.get(name).unwrap_or(&0.0);
                    let d3 = k3.get(name).unwrap_or(&0.0);
                    let d4 = k4.get(name).unwrap_or(&0.0);
                    (name.clone(), (a51 * d1 + a52 * d2 + a53 * d3 + a54 * d4) * h)
                })
                .collect();
            let state5 = self.apply_stock_increments(state, &inc5);
            let (_, flows5) = self.evaluate_system(model, &state5, t + 8.0 * h / 9.0)?;
            let k5 = self.compute_derivatives(model, &flows5)?;

            // Stage 6
            let inc6: HashMap<String, f64> = k1.iter()
                .map(|(name, &d1)| {
                    let d2 = k2.get(name).unwrap_or(&0.0);
                    let d3 = k3.get(name).unwrap_or(&0.0);
                    let d4 = k4.get(name).unwrap_or(&0.0);
                    let d5 = k5.get(name).unwrap_or(&0.0);
                    (name.clone(), (a61 * d1 + a62 * d2 + a63 * d3 + a64 * d4 + a65 * d5) * h)
                })
                .collect();
            let state6 = self.apply_stock_increments(state, &inc6);
            let (_, flows6) = self.evaluate_system(model, &state6, t + h)?;
            let k6 = self.compute_derivatives(model, &flows6)?;

            // Stage 7
            let inc7: HashMap<String, f64> = k1.iter()
                .map(|(name, &d1)| {
                    let d3 = k3.get(name).unwrap_or(&0.0);
                    let d4 = k4.get(name).unwrap_or(&0.0);
                    let d5 = k5.get(name).unwrap_or(&0.0);
                    let d6 = k6.get(name).unwrap_or(&0.0);
                    (name.clone(), (a71 * d1 + a73 * d3 + a74 * d4 + a75 * d5 + a76 * d6) * h)
                })
                .collect();
            let state7 = self.apply_stock_increments(state, &inc7);
            let (aux7, flows7) = self.evaluate_system(model, &state7, t + h)?;
            let k7 = self.compute_derivatives(model, &flows7)?;

            // Compute 5th order solution
            let mut y5 = HashMap::new();
            for (name, &y0) in &state.stocks {
                let d1 = k1.get(name).unwrap_or(&0.0);
                let d3 = k3.get(name).unwrap_or(&0.0);
                let d4 = k4.get(name).unwrap_or(&0.0);
                let d5 = k5.get(name).unwrap_or(&0.0);
                let d6 = k6.get(name).unwrap_or(&0.0);
                let val = y0 + (b1 * d1 + b3 * d3 + b4 * d4 + b5 * d5 + b6 * d6) * h;
                y5.insert(name.clone(), val);
            }

            // Compute 4th order solution for error estimation
            let mut y4 = HashMap::new();
            for (name, &y0) in &state.stocks {
                let d1 = k1.get(name).unwrap_or(&0.0);
                let d3 = k3.get(name).unwrap_or(&0.0);
                let d4 = k4.get(name).unwrap_or(&0.0);
                let d5 = k5.get(name).unwrap_or(&0.0);
                let d6 = k6.get(name).unwrap_or(&0.0);
                let d7 = k7.get(name).unwrap_or(&0.0);
                let val = y0 + (b1_star * d1 + b3_star * d3 + b4_star * d4 + b5_star * d5 + b6_star * d6 + b7_star * d7) * h;
                y4.insert(name.clone(), val);
            }

            // Check error and adjust step size
            let (error, new_h) = self.compute_error_and_step(&y4, &y5, h);

            if error <= 1.0 {
                // Accept the step
                let mut new_state = state.clone();
                new_state.time = t + h;

                // Apply constraints and use 5th order solution
                for (stock_name, &new_value) in &y5 {
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

                new_state.auxiliaries = aux7;
                new_state.flows = flows7;

                return Ok(new_state);
            } else {
                // Reject and retry with smaller step
                h = new_h;
                if h < self.min_step {
                    return Err(format!(
                        "Step size ({}) below minimum ({}). Error: {}",
                        h, self.min_step, error
                    ));
                }
            }
        }

        Err("RK45 failed to converge after maximum attempts".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Model, Stock, Flow, Parameter, Expression};

    #[test]
    fn test_euler_simple_growth() {
        let mut model = Model::new("Growth");
        model.time.dt = 1.0;

        // Stock: Population = 100
        let stock = Stock::new("Population", "100");
        model.add_stock(stock).unwrap();

        // Flow: growth = Population * 0.1
        model.add_parameter(Parameter::new("growth_rate", 0.1)).unwrap();
        let flow = Flow::new("growth", "Population * growth_rate");
        model.add_flow(flow).unwrap();

        // Connect flow
        model.stocks.get_mut("Population").unwrap().inflows.push("growth".to_string());

        let state = SimulationState::initialize_from_model(&model).unwrap();
        assert_eq!(state.stocks.get("Population"), Some(&100.0));

        let integrator = EulerIntegrator;
        let new_state = integrator.step(&model, &state, 1.0).unwrap();

        // After 1 step: Population = 100 + (100 * 0.1) * 1 = 110
        assert_eq!(new_state.stocks.get("Population"), Some(&110.0));
    }

    #[test]
    fn test_heun_growth() {
        let mut model = Model::new("Growth");
        model.time.dt = 1.0;

        let stock = Stock::new("Population", "100");
        model.add_stock(stock).unwrap();

        model.add_parameter(Parameter::new("growth_rate", 0.1)).unwrap();
        let flow = Flow::new("growth", "Population * growth_rate");
        model.add_flow(flow).unwrap();

        model.stocks.get_mut("Population").unwrap().inflows.push("growth".to_string());

        let state = SimulationState::initialize_from_model(&model).unwrap();
        let integrator = HeunIntegrator;
        let new_state = integrator.step(&model, &state, 1.0).unwrap();

        // Heun should give a more accurate result than Euler
        // Predictor: 100 + 10 = 110
        // Corrector: 100 + (10 + 11)/2 = 110.5
        assert!(new_state.stocks.get("Population").unwrap() > &110.0);
    }

    #[test]
    fn test_backward_euler_growth() {
        let mut model = Model::new("Growth");
        model.time.dt = 1.0;

        let stock = Stock::new("Population", "100");
        model.add_stock(stock).unwrap();

        model.add_parameter(Parameter::new("growth_rate", 0.1)).unwrap();
        let flow = Flow::new("growth", "Population * growth_rate");
        model.add_flow(flow).unwrap();

        model.stocks.get_mut("Population").unwrap().inflows.push("growth".to_string());

        let state = SimulationState::initialize_from_model(&model).unwrap();
        let integrator = BackwardEulerIntegrator::default();
        let new_state = integrator.step(&model, &state, 1.0).unwrap();

        // Backward Euler should be stable and give reasonable results
        assert!(new_state.stocks.get("Population").unwrap() > &100.0);
        assert!(new_state.stocks.get("Population").unwrap() < &150.0);
    }

    #[test]
    fn test_integrator_comparison() {
        // Compare all integrators on the same simple problem
        let mut model = Model::new("Test");
        model.time.dt = 0.1;

        let stock = Stock::new("X", "1.0");
        model.add_stock(stock).unwrap();
        model.add_parameter(Parameter::new("k", 1.0)).unwrap();
        let flow = Flow::new("f", "X * k");
        model.add_flow(flow).unwrap();
        model.stocks.get_mut("X").unwrap().inflows.push("f".to_string());

        let state = SimulationState::initialize_from_model(&model).unwrap();
        let dt = 0.1;

        // Euler
        let euler = EulerIntegrator;
        let state_euler = euler.step(&model, &state, dt).unwrap();

        // Heun
        let heun = HeunIntegrator;
        let state_heun = heun.step(&model, &state, dt).unwrap();

        // RK4
        let rk4 = RK4Integrator;
        let state_rk4 = rk4.step(&model, &state, dt).unwrap();

        // Backward Euler
        let backward = BackwardEulerIntegrator::default();
        let state_backward = backward.step(&model, &state, dt).unwrap();

        // All should give results in reasonable range
        // Exact solution: X(0.1) = exp(0.1) ≈ 1.10517
        let exact = 1.0_f64.exp().powf(0.1);

        println!("Exact: {}", exact);
        println!("Euler: {}", state_euler.stocks.get("X").unwrap());
        println!("Heun: {}", state_heun.stocks.get("X").unwrap());
        println!("RK4: {}", state_rk4.stocks.get("X").unwrap());
        println!("Backward Euler: {}", state_backward.stocks.get("X").unwrap());

        // RK4 should be most accurate
        assert!(state_rk4.stocks.get("X").unwrap() > &1.0);
    }
}
