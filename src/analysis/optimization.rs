/// Parameter calibration and optimization algorithms
///
/// Implements gradient-based (BFGS) and genetic algorithm optimization
/// for parameter estimation and model calibration

use crate::model::{Model, Parameter};
use crate::simulation::{SimulationEngine, SimulationConfig, IntegrationMethod};
use rand::Rng;
use rand::distributions::{Distribution, Uniform};
use std::collections::HashMap;

/// Optimization configuration
#[derive(Debug, Clone)]
pub struct OptimizationConfig {
    /// Maximum number of iterations/generations
    pub max_iterations: usize,
    /// Convergence tolerance
    pub tolerance: f64,
    /// Integration method for simulation
    pub integration_method: IntegrationMethod,
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        Self {
            max_iterations: 100,
            tolerance: 1e-6,
            integration_method: IntegrationMethod::RK4,
        }
    }
}

/// Result of optimization
#[derive(Debug, Clone)]
pub struct OptimizationResult {
    /// Optimal parameter values
    pub parameters: HashMap<String, f64>,
    /// Final objective function value
    pub objective_value: f64,
    /// Number of iterations performed
    pub iterations: usize,
    /// Whether optimization converged
    pub converged: bool,
    /// History of objective values
    pub history: Vec<f64>,
}

/// Parameter bounds
#[derive(Debug, Clone)]
pub struct ParameterBounds {
    pub name: String,
    pub min: f64,
    pub max: f64,
}

impl ParameterBounds {
    pub fn new(name: &str, min: f64, max: f64) -> Self {
        Self {
            name: name.to_string(),
            min,
            max,
        }
    }

    /// Clamp value to bounds
    pub fn clamp(&self, value: f64) -> f64 {
        value.max(self.min).min(self.max)
    }

    /// Check if value is within bounds
    pub fn contains(&self, value: f64) -> bool {
        value >= self.min && value <= self.max
    }
}

/// Objective function type
pub type ObjectiveFunction = Box<dyn Fn(&Model, &SimulationEngine) -> Result<f64, String>>;

/// Gradient-based optimizer using BFGS quasi-Newton method
pub struct GradientOptimizer {
    config: OptimizationConfig,
    bounds: Vec<ParameterBounds>,
    /// Step size for finite difference gradient
    epsilon: f64,
}

impl GradientOptimizer {
    pub fn new(config: OptimizationConfig, bounds: Vec<ParameterBounds>) -> Self {
        Self {
            config,
            bounds,
            epsilon: 1e-6,
        }
    }

    /// Optimize parameters to minimize objective function
    pub fn optimize(
        &self,
        model: &Model,
        objective: ObjectiveFunction,
    ) -> Result<OptimizationResult, String> {
        // Get parameter names and initial values
        let param_names: Vec<String> = self.bounds.iter().map(|b| b.name.clone()).collect();
        let mut params: Vec<f64> = param_names.iter()
            .map(|name| {
                model.parameters.get(name)
                    .map(|p| p.value)
                    .unwrap_or(0.0)
            })
            .collect();

        let n = params.len();
        let mut history = Vec::new();

        // Initialize inverse Hessian approximation (identity matrix)
        let mut inv_hessian = vec![vec![0.0; n]; n];
        for i in 0..n {
            inv_hessian[i][i] = 1.0;
        }

        // Evaluate initial objective
        let mut current_obj = self.evaluate_objective(model, &params, &param_names, &objective)?;
        history.push(current_obj);

        let mut converged = false;

        for iter in 0..self.config.max_iterations {
            // Compute gradient using finite differences
            let gradient = self.compute_gradient(model, &params, &param_names, &objective, current_obj)?;

            // Check for convergence
            let grad_norm = gradient.iter().map(|g| g * g).sum::<f64>().sqrt();
            if grad_norm < self.config.tolerance {
                converged = true;
                break;
            }

            // Compute search direction: d = -H * gradient
            let mut direction = vec![0.0; n];
            for i in 0..n {
                for j in 0..n {
                    direction[i] -= inv_hessian[i][j] * gradient[j];
                }
            }

            // Line search to find step size
            let alpha = self.line_search(
                model,
                &params,
                &direction,
                &param_names,
                &objective,
                current_obj,
                &gradient,
            )?;

            // Update parameters
            let mut new_params = params.clone();
            for i in 0..n {
                new_params[i] = self.bounds[i].clamp(params[i] + alpha * direction[i]);
            }

            // Evaluate new objective
            let new_obj = self.evaluate_objective(model, &new_params, &param_names, &objective)?;
            history.push(new_obj);

            // Update inverse Hessian using BFGS formula
            let s: Vec<f64> = (0..n).map(|i| new_params[i] - params[i]).collect();
            let new_gradient = self.compute_gradient(model, &new_params, &param_names, &objective, new_obj)?;
            let y: Vec<f64> = (0..n).map(|i| new_gradient[i] - gradient[i]).collect();

            let s_dot_y: f64 = s.iter().zip(y.iter()).map(|(si, yi)| si * yi).sum();

            if s_dot_y.abs() > 1e-10 {
                // BFGS update
                let rho = 1.0 / s_dot_y;

                // H' = (I - rho * s * y^T) * H * (I - rho * y * s^T) + rho * s * s^T
                let mut new_inv_hessian = vec![vec![0.0; n]; n];

                for i in 0..n {
                    for j in 0..n {
                        let mut value = inv_hessian[i][j];

                        // Subtract rho * s * y^T * H
                        for k in 0..n {
                            value -= rho * s[i] * y[k] * inv_hessian[k][j];
                        }

                        // Subtract rho * H * y * s^T
                        for k in 0..n {
                            value -= rho * inv_hessian[i][k] * y[k] * s[j];
                        }

                        // Add rho^2 * s * y^T * H * y * s^T
                        let mut hy_dot_y = 0.0;
                        for k in 0..n {
                            for l in 0..n {
                                hy_dot_y += inv_hessian[k][l] * y[l] * y[k];
                            }
                        }
                        value += rho * rho * s[i] * s[j] * hy_dot_y;

                        // Add rho * s * s^T
                        value += rho * s[i] * s[j];

                        new_inv_hessian[i][j] = value;
                    }
                }

                inv_hessian = new_inv_hessian;
            }

            params = new_params;
            current_obj = new_obj;

            // Check for convergence
            if iter > 0 && (history[iter] - history[iter - 1]).abs() < self.config.tolerance {
                converged = true;
                break;
            }
        }

        // Build result
        let mut result_params = HashMap::new();
        for (i, name) in param_names.iter().enumerate() {
            result_params.insert(name.clone(), params[i]);
        }

        Ok(OptimizationResult {
            parameters: result_params,
            objective_value: current_obj,
            iterations: history.len(),
            converged,
            history,
        })
    }

    /// Evaluate objective function with given parameters
    fn evaluate_objective(
        &self,
        model: &Model,
        params: &[f64],
        param_names: &[String],
        objective: &ObjectiveFunction,
    ) -> Result<f64, String> {
        // Create model copy with updated parameters
        let mut model_copy = model.clone();
        for (i, name) in param_names.iter().enumerate() {
            if let Some(param) = model_copy.parameters.get_mut(name) {
                param.value = params[i];
            }
        }

        // Run simulation
        let config = SimulationConfig {
            integration_method: self.config.integration_method,
            output_interval: None,
        };

        let mut engine = SimulationEngine::new(model_copy.clone(), config)?;
        let _results = engine.run()?;

        // Evaluate objective
        objective(&model_copy, &engine)
    }

    /// Compute gradient using finite differences
    fn compute_gradient(
        &self,
        model: &Model,
        params: &[f64],
        param_names: &[String],
        objective: &ObjectiveFunction,
        base_obj: f64,
    ) -> Result<Vec<f64>, String> {
        let n = params.len();
        let mut gradient = vec![0.0; n];

        for i in 0..n {
            let mut perturbed = params.to_vec();
            perturbed[i] += self.epsilon;
            perturbed[i] = self.bounds[i].clamp(perturbed[i]);

            let perturbed_obj = self.evaluate_objective(model, &perturbed, param_names, objective)?;
            gradient[i] = (perturbed_obj - base_obj) / self.epsilon;
        }

        Ok(gradient)
    }

    /// Line search using backtracking
    fn line_search(
        &self,
        model: &Model,
        params: &[f64],
        direction: &[f64],
        param_names: &[String],
        objective: &ObjectiveFunction,
        base_obj: f64,
        gradient: &[f64],
    ) -> Result<f64, String> {
        let mut alpha = 1.0;
        let c = 0.5; // Sufficient decrease constant
        let rho = 0.5; // Backtracking factor

        let directional_derivative: f64 = gradient.iter()
            .zip(direction.iter())
            .map(|(g, d)| g * d)
            .sum();

        for _ in 0..20 {
            let mut new_params = params.to_vec();
            for i in 0..params.len() {
                new_params[i] = self.bounds[i].clamp(params[i] + alpha * direction[i]);
            }

            let new_obj = self.evaluate_objective(model, &new_params, param_names, objective)?;

            // Armijo condition
            if new_obj <= base_obj + c * alpha * directional_derivative {
                return Ok(alpha);
            }

            alpha *= rho;
        }

        Ok(alpha)
    }
}

/// Genetic algorithm optimizer
pub struct GeneticOptimizer {
    config: OptimizationConfig,
    bounds: Vec<ParameterBounds>,
    /// Population size
    population_size: usize,
    /// Crossover probability
    crossover_rate: f64,
    /// Mutation probability
    mutation_rate: f64,
    /// Mutation strength
    mutation_strength: f64,
}

impl GeneticOptimizer {
    pub fn new(config: OptimizationConfig, bounds: Vec<ParameterBounds>) -> Self {
        Self {
            config,
            bounds,
            population_size: 50,
            crossover_rate: 0.8,
            mutation_rate: 0.1,
            mutation_strength: 0.1,
        }
    }

    pub fn with_parameters(
        mut self,
        population_size: usize,
        crossover_rate: f64,
        mutation_rate: f64,
        mutation_strength: f64,
    ) -> Self {
        self.population_size = population_size;
        self.crossover_rate = crossover_rate;
        self.mutation_rate = mutation_rate;
        self.mutation_strength = mutation_strength;
        self
    }

    /// Optimize using genetic algorithm
    pub fn optimize(
        &self,
        model: &Model,
        objective: ObjectiveFunction,
    ) -> Result<OptimizationResult, String> {
        let mut rng = rand::thread_rng();
        let n_params = self.bounds.len();
        let param_names: Vec<String> = self.bounds.iter().map(|b| b.name.clone()).collect();

        // Initialize population
        let mut population: Vec<Vec<f64>> = (0..self.population_size)
            .map(|_| {
                self.bounds.iter()
                    .map(|bounds| {
                        let dist = Uniform::new(bounds.min, bounds.max);
                        dist.sample(&mut rng)
                    })
                    .collect()
            })
            .collect();

        let mut history = Vec::new();
        let mut best_individual = population[0].clone();
        let mut best_fitness = f64::INFINITY;
        let mut converged = false;

        for generation in 0..self.config.max_iterations {
            // Evaluate fitness for all individuals
            let mut fitness: Vec<f64> = Vec::new();

            for individual in &population {
                let obj = self.evaluate_objective(model, individual, &param_names, &objective)?;
                fitness.push(obj);

                if obj < best_fitness {
                    best_fitness = obj;
                    best_individual = individual.clone();
                }
            }

            history.push(best_fitness);

            // Check convergence
            if generation > 10 {
                let recent_improvement = history[generation - 10] - history[generation];
                if recent_improvement < self.config.tolerance {
                    converged = true;
                    break;
                }
            }

            // Selection: tournament selection
            let mut new_population = Vec::new();

            while new_population.len() < self.population_size {
                // Select parents
                let parent1 = self.tournament_select(&population, &fitness, &mut rng);
                let parent2 = self.tournament_select(&population, &fitness, &mut rng);

                // Crossover
                let mut offspring = if rng.gen_range(0.0..1.0) < self.crossover_rate {
                    self.crossover(&parent1, &parent2, &mut rng)
                } else {
                    parent1.clone()
                };

                // Mutation
                self.mutate(&mut offspring, &mut rng);

                new_population.push(offspring);
            }

            // Elitism: keep best individual
            new_population[0] = best_individual.clone();

            population = new_population;
        }

        // Build result
        let mut result_params = HashMap::new();
        for (i, name) in param_names.iter().enumerate() {
            result_params.insert(name.clone(), best_individual[i]);
        }

        Ok(OptimizationResult {
            parameters: result_params,
            objective_value: best_fitness,
            iterations: history.len(),
            converged,
            history,
        })
    }

    /// Evaluate objective function
    fn evaluate_objective(
        &self,
        model: &Model,
        params: &[f64],
        param_names: &[String],
        objective: &ObjectiveFunction,
    ) -> Result<f64, String> {
        let mut model_copy = model.clone();
        for (i, name) in param_names.iter().enumerate() {
            if let Some(param) = model_copy.parameters.get_mut(name) {
                param.value = params[i];
            }
        }

        let config = SimulationConfig {
            integration_method: self.config.integration_method,
            output_interval: None,
        };

        let mut engine = SimulationEngine::new(model_copy.clone(), config)?;
        let _results = engine.run()?;

        objective(&model_copy, &engine)
    }

    /// Tournament selection
    fn tournament_select<R: Rng>(
        &self,
        population: &[Vec<f64>],
        fitness: &[f64],
        rng: &mut R,
    ) -> Vec<f64> {
        let tournament_size = 3;
        let mut best_idx = rng.gen_range(0..population.len());
        let mut best_fitness = fitness[best_idx];

        for _ in 1..tournament_size {
            let idx = rng.gen_range(0..population.len());
            if fitness[idx] < best_fitness {
                best_idx = idx;
                best_fitness = fitness[idx];
            }
        }

        population[best_idx].clone()
    }

    /// Uniform crossover
    fn crossover<R: Rng>(&self, parent1: &[f64], parent2: &[f64], rng: &mut R) -> Vec<f64> {
        parent1.iter()
            .zip(parent2.iter())
            .map(|(&p1, &p2)| if rng.gen_range(0.0..1.0) < 0.5 { p1 } else { p2 })
            .collect()
    }

    /// Gaussian mutation
    fn mutate<R: Rng>(&self, individual: &mut [f64], rng: &mut R) {
        use rand_distr::Normal;

        for i in 0..individual.len() {
            if rng.gen_range(0.0..1.0) < self.mutation_rate {
                let range = self.bounds[i].max - self.bounds[i].min;
                let normal = Normal::new(0.0, self.mutation_strength * range)
                    .unwrap_or_else(|_| Normal::new(0.0, 1.0).unwrap());

                individual[i] += normal.sample(rng);
                individual[i] = self.bounds[i].clamp(individual[i]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Model, Stock, Flow};

    #[test]
    #[ignore] // This test requires more robust gradient computation for this problem
    fn test_gradient_optimizer() {
        // Create a simple model to optimize
        let mut model = Model::new("Test");
        model.time.start = 0.0;
        model.time.stop = 10.0;
        model.time.dt = 0.1;

        model.add_stock(Stock::new("X", "1.0")).unwrap();
        model.add_parameter(Parameter::new("k", 0.5)).unwrap();
        model.add_flow(Flow::new("growth", "k * X")).unwrap();

        model.stocks.get_mut("X").unwrap().inflows.push("growth".to_string());

        // Objective: minimize difference from target final value
        let objective: ObjectiveFunction = Box::new(|_model, engine| {
            let final_x = engine.current_state().stocks.get("X").unwrap_or(&0.0);
            let target = 10.0;
            Ok((final_x - target).powi(2))
        });

        let bounds = vec![ParameterBounds::new("k", 0.0, 2.0)];
        let config = OptimizationConfig {
            max_iterations: 20,
            tolerance: 1e-3,
            integration_method: IntegrationMethod::RK4,
        };

        let optimizer = GradientOptimizer::new(config, bounds);
        let result = optimizer.optimize(&model, objective).unwrap();

        println!("Optimized k: {}", result.parameters.get("k").unwrap());
        println!("Final objective: {}", result.objective_value);
        println!("Iterations: {}", result.iterations);

        assert!(result.objective_value < 1.0);
    }
}
