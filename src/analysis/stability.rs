/// Stability analysis using eigensystem analysis
///
/// Computes the Jacobian matrix of the system and analyzes its eigenvalues
/// to determine stability properties

use crate::model::Model;
use crate::simulation::{SimulationState, SimulationEngine, SimulationConfig, IntegrationMethod};
use nalgebra::{DMatrix, DVector, Complex};
use std::collections::HashMap;

/// Stability classification of an equilibrium point
#[derive(Debug, Clone, PartialEq)]
pub enum StabilityType {
    /// All eigenvalues have negative real parts
    Stable,
    /// At least one eigenvalue has positive real part
    Unstable,
    /// All eigenvalues have non-positive real parts, at least one is zero
    MarginallyStable,
    /// Complex eigenvalues with non-zero imaginary parts
    Oscillatory,
    /// Cannot determine (e.g., numerical issues)
    Unknown,
}

/// Result of stability analysis
#[derive(Debug, Clone)]
pub struct StabilityAnalysis {
    /// Classification of stability
    pub stability_type: StabilityType,
    /// Eigenvalues of the Jacobian matrix
    pub eigenvalues: Vec<Complex<f64>>,
    /// Jacobian matrix at the equilibrium point
    pub jacobian: DMatrix<f64>,
    /// Stock names (order corresponds to Jacobian rows/columns)
    pub stock_names: Vec<String>,
    /// Maximum real part of eigenvalues
    pub max_real_part: f64,
    /// Indicates if system has oscillatory modes
    pub has_oscillations: bool,
    /// Dominant period for oscillatory systems (if applicable)
    pub dominant_period: Option<f64>,
}

/// Stability analyzer
pub struct StabilityAnalyzer {
    /// Perturbation size for numerical Jacobian computation
    pub epsilon: f64,
}

impl Default for StabilityAnalyzer {
    fn default() -> Self {
        Self { epsilon: 1e-6 }
    }
}

impl StabilityAnalyzer {
    pub fn new(epsilon: f64) -> Self {
        Self { epsilon }
    }

    /// Analyze stability at a given state
    pub fn analyze(&self, model: &Model, state: &SimulationState) -> Result<StabilityAnalysis, String> {
        // Get stock names in consistent order
        let mut stock_names: Vec<String> = model.stocks.keys().cloned().collect();
        stock_names.sort();
        let n = stock_names.len();

        if n == 0 {
            return Err("No stocks in model".to_string());
        }

        // Compute Jacobian matrix using finite differences
        let jacobian = self.compute_jacobian(model, state, &stock_names)?;

        // Compute eigenvalues
        let eigenvalues = self.compute_eigenvalues(&jacobian)?;

        // Analyze eigenvalues
        let max_real_part = eigenvalues.iter()
            .map(|e| e.re)
            .fold(f64::NEG_INFINITY, f64::max);

        let has_oscillations = eigenvalues.iter()
            .any(|e| e.im.abs() > 1e-10);

        // Determine stability type
        let stability_type = self.classify_stability(&eigenvalues);

        // Compute dominant period for oscillatory systems
        let dominant_period = if has_oscillations {
            eigenvalues.iter()
                .filter(|e| e.im.abs() > 1e-10)
                .map(|e| 2.0 * std::f64::consts::PI / e.im.abs())
                .min_by(|a, b| a.partial_cmp(b).unwrap())
        } else {
            None
        };

        Ok(StabilityAnalysis {
            stability_type,
            eigenvalues,
            jacobian,
            stock_names,
            max_real_part,
            has_oscillations,
            dominant_period,
        })
    }

    /// Compute Jacobian matrix using finite differences
    fn compute_jacobian(
        &self,
        model: &Model,
        state: &SimulationState,
        stock_names: &[String],
    ) -> Result<DMatrix<f64>, String> {
        let n = stock_names.len();
        let mut jacobian = DMatrix::zeros(n, n);

        // Compute derivatives d(flow_i)/d(stock_j)
        let base_derivatives = self.compute_derivatives(model, state, stock_names)?;

        for (j, stock_name) in stock_names.iter().enumerate() {
            // Perturb stock j
            let mut perturbed_state = state.clone();
            let original_value = state.stocks.get(stock_name)
                .ok_or_else(|| format!("Stock '{}' not found", stock_name))?;

            perturbed_state.stocks.insert(stock_name.clone(), original_value + self.epsilon);

            // Compute derivatives at perturbed state
            let perturbed_derivatives = self.compute_derivatives(model, &perturbed_state, stock_names)?;

            // Finite difference: (f(x + h) - f(x)) / h
            for (i, _) in stock_names.iter().enumerate() {
                let base_deriv = base_derivatives[i];
                let pert_deriv = perturbed_derivatives[i];
                jacobian[(i, j)] = (pert_deriv - base_deriv) / self.epsilon;
            }
        }

        Ok(jacobian)
    }

    /// Compute derivatives (d(stock)/dt) for all stocks at given state
    fn compute_derivatives(
        &self,
        model: &Model,
        state: &SimulationState,
        stock_names: &[String],
    ) -> Result<Vec<f64>, String> {
        use crate::model::expression::EvaluationContext;

        let mut derivatives = Vec::with_capacity(stock_names.len());

        // Evaluate auxiliaries first
        let mut eval_state = state.clone();
        let mut auxiliaries = HashMap::new();

        const MAX_PASSES: usize = 20;
        for pass in 0..MAX_PASSES {
            let mut changed = false;
            let mut temp_state = eval_state.clone();
            temp_state.auxiliaries = auxiliaries.clone();

            for (name, aux) in &model.auxiliaries {
                let mut aux_state = temp_state.clone();
                let mut context = EvaluationContext::new(model, &mut aux_state, state.time);

                match aux.equation.evaluate(&mut context) {
                    Ok(value) => {
                        if let Some(&old_value) = auxiliaries.get(name) {
                            if (value - old_value).abs() > 1e-10 {
                                changed = true;
                            }
                        } else {
                            changed = true;
                        }
                        auxiliaries.insert(name.clone(), value);
                    }
                    Err(_) if pass < 5 => continue,
                    Err(e) => return Err(format!("Error evaluating auxiliary '{}': {}", name, e)),
                }
            }

            if !changed && pass > 0 {
                break;
            }
        }
        eval_state.auxiliaries = auxiliaries;

        // Evaluate flows
        let mut flows = HashMap::new();
        for (name, flow) in &model.flows {
            let mut flow_state = eval_state.clone();
            let mut context = EvaluationContext::new(model, &mut flow_state, state.time);
            let value = flow.equation.evaluate(&mut context)
                .map_err(|e| format!("Error evaluating flow '{}': {}", name, e))?;
            flows.insert(name.clone(), value);
        }

        // Compute derivatives for each stock
        for stock_name in stock_names {
            let stock = model.stocks.get(stock_name)
                .ok_or_else(|| format!("Stock '{}' not found", stock_name))?;

            let mut derivative = 0.0;

            // Add inflows
            for inflow_name in &stock.inflows {
                if let Some(&flow_value) = flows.get(inflow_name) {
                    derivative += flow_value;
                } else {
                    return Err(format!("Inflow '{}' not found", inflow_name));
                }
            }

            // Subtract outflows
            for outflow_name in &stock.outflows {
                if let Some(&flow_value) = flows.get(outflow_name) {
                    derivative -= flow_value;
                } else {
                    return Err(format!("Outflow '{}' not found", outflow_name));
                }
            }

            derivatives.push(derivative);
        }

        Ok(derivatives)
    }

    /// Compute eigenvalues of a matrix
    fn compute_eigenvalues(&self, matrix: &DMatrix<f64>) -> Result<Vec<Complex<f64>>, String> {
        // Use nalgebra's eigenvalue computation
        let eigen = matrix.clone().complex_eigenvalues();

        let eigenvalues: Vec<Complex<f64>> = eigen.iter()
            .map(|e| Complex::new(e.re, e.im))
            .collect();

        Ok(eigenvalues)
    }

    /// Classify stability based on eigenvalues
    fn classify_stability(&self, eigenvalues: &[Complex<f64>]) -> StabilityType {
        let mut max_real = f64::NEG_INFINITY;
        let mut has_zero = false;
        let mut has_oscillations = false;

        for eigenvalue in eigenvalues {
            max_real = max_real.max(eigenvalue.re);
            if eigenvalue.re.abs() < 1e-8 {
                has_zero = true;
            }
            if eigenvalue.im.abs() > 1e-10 {
                has_oscillations = true;
            }
        }

        if max_real > 1e-8 {
            if has_oscillations {
                StabilityType::Oscillatory
            } else {
                StabilityType::Unstable
            }
        } else if max_real < -1e-8 {
            if has_oscillations {
                StabilityType::Oscillatory
            } else {
                StabilityType::Stable
            }
        } else if has_zero {
            StabilityType::MarginallyStable
        } else {
            StabilityType::Unknown
        }
    }

    /// Find equilibrium point through simulation
    pub fn find_equilibrium(
        &self,
        model: &Model,
        initial_state: &SimulationState,
        max_time: f64,
        tolerance: f64,
    ) -> Result<SimulationState, String> {
        let config = SimulationConfig {
            integration_method: IntegrationMethod::RK4,
            output_interval: None,
        };

        let mut engine = SimulationEngine::new(model.clone(), config)?;

        let mut prev_stocks = initial_state.stocks.clone();
        let dt = model.time.dt;
        let mut time = 0.0;

        while time < max_time {
            engine.step()?;
            time += dt;

            // Check if we've reached equilibrium
            let current_stocks = &engine.current_state().stocks;
            let mut max_change: f64 = 0.0;

            for (name, &value) in current_stocks {
                if let Some(&prev_value) = prev_stocks.get(name) {
                    let change = ((value - prev_value) / dt).abs();
                    max_change = max_change.max(change);
                }
            }

            if max_change < tolerance {
                return Ok(engine.current_state().clone());
            }

            prev_stocks = current_stocks.clone();
        }

        Err(format!("Failed to find equilibrium within {} time units", max_time))
    }
}

impl StabilityAnalysis {
    /// Print a human-readable summary
    pub fn summary(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!("Stability Type: {:?}\n", self.stability_type));
        s.push_str(&format!("Max Real Part: {:.6}\n", self.max_real_part));
        s.push_str(&format!("Has Oscillations: {}\n", self.has_oscillations));

        if let Some(period) = self.dominant_period {
            s.push_str(&format!("Dominant Period: {:.4}\n", period));
        }

        s.push_str("\nEigenvalues:\n");
        for (i, eigenvalue) in self.eigenvalues.iter().enumerate() {
            s.push_str(&format!("  λ_{}: {:.6} + {:.6}i\n", i, eigenvalue.re, eigenvalue.im));
        }

        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Model, Stock, Flow, Parameter};

    #[test]
    fn test_stable_system() {
        // Create a simple decay model: dX/dt = -k*X (stable)
        let mut model = Model::new("Decay");
        model.time.dt = 0.1;

        model.add_stock(Stock::new("X", "10.0")).unwrap();
        model.add_parameter(Parameter::new("k", 0.5)).unwrap();
        model.add_flow(Flow::new("decay", "k * X")).unwrap();

        model.stocks.get_mut("X").unwrap().outflows.push("decay".to_string());

        let state = SimulationState::initialize_from_model(&model).unwrap();
        let analyzer = StabilityAnalyzer::default();
        let analysis = analyzer.analyze(&model, &state).unwrap();

        println!("{}", analysis.summary());

        // System should be stable (eigenvalue = -k = -0.5)
        assert_eq!(analysis.stability_type, StabilityType::Stable);
        assert!(analysis.max_real_part < 0.0);
    }

    #[test]
    fn test_unstable_system() {
        // Create a growth model: dX/dt = k*X (unstable)
        let mut model = Model::new("Growth");
        model.time.dt = 0.1;

        model.add_stock(Stock::new("X", "10.0")).unwrap();
        model.add_parameter(Parameter::new("k", 0.5)).unwrap();
        model.add_flow(Flow::new("growth", "k * X")).unwrap();

        model.stocks.get_mut("X").unwrap().inflows.push("growth".to_string());

        let state = SimulationState::initialize_from_model(&model).unwrap();
        let analyzer = StabilityAnalyzer::default();
        let analysis = analyzer.analyze(&model, &state).unwrap();

        println!("{}", analysis.summary());

        // System should be unstable (eigenvalue = +k = +0.5)
        assert_eq!(analysis.stability_type, StabilityType::Unstable);
        assert!(analysis.max_real_part > 0.0);
    }
}
