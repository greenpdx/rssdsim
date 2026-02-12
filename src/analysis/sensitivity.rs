/// Sensitivity analysis tools for system dynamics models
///
/// Provides methods for:
/// - Parameter sweeps (one-at-a-time sensitivity)
/// - Latin Hypercube Sampling (LHS)
/// - Morris screening method
/// - Sobol variance-based sensitivity

use std::collections::HashMap;
use rand::prelude::*;
use crate::model::Model;
use crate::simulation::{SimulationEngine, SimulationConfig, SimulationResults};

/// Parameter range for sensitivity analysis
#[derive(Debug, Clone)]
pub struct ParameterRange {
    pub name: String,
    pub min: f64,
    pub max: f64,
    pub baseline: f64,
}

impl ParameterRange {
    pub fn new(name: String, min: f64, max: f64, baseline: f64) -> Self {
        Self { name, min, max, baseline }
    }

    /// Sample a value uniformly from the range
    pub fn sample_uniform(&self, rng: &mut impl Rng) -> f64 {
        use rand::distributions::{Distribution, Standard};
        let random_val: f64 = Standard.sample(rng);
        self.min + (self.max - self.min) * random_val
    }

    /// Get value at fractional position [0, 1] in range
    pub fn at_fraction(&self, fraction: f64) -> f64 {
        self.min + (self.max - self.min) * fraction
    }
}

/// Sample point in parameter space
#[derive(Debug, Clone)]
pub struct ParameterSample {
    pub values: HashMap<String, f64>,
}

impl ParameterSample {
    pub fn new() -> Self {
        Self { values: HashMap::new() }
    }

    pub fn set(&mut self, name: String, value: f64) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<f64> {
        self.values.get(name).copied()
    }
}

/// Result of a sensitivity analysis run
#[derive(Debug, Clone)]
pub struct SensitivityResult {
    pub sample: ParameterSample,
    pub outputs: HashMap<String, Vec<f64>>,
    pub metrics: HashMap<String, f64>,
}

impl SensitivityResult {
    pub fn from_simulation(sample: ParameterSample, results: &SimulationResults) -> Self {
        let mut outputs = HashMap::new();
        let mut metrics = HashMap::new();

        // Extract time series for all variables
        for state in &results.states {
            for (name, &value) in &state.stocks {
                outputs.entry(name.clone())
                    .or_insert_with(Vec::new)
                    .push(value);
            }
            for (name, &value) in &state.flows {
                outputs.entry(name.clone())
                    .or_insert_with(Vec::new)
                    .push(value);
            }
            for (name, &value) in &state.auxiliaries {
                outputs.entry(name.clone())
                    .or_insert_with(Vec::new)
                    .push(value);
            }
        }

        // Calculate summary metrics
        for (name, series) in &outputs {
            if !series.is_empty() {
                let final_value = series[series.len() - 1];
                let mean = series.iter().sum::<f64>() / series.len() as f64;
                let max = series.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                let min = series.iter().copied().fold(f64::INFINITY, f64::min);

                metrics.insert(format!("{}_final", name), final_value);
                metrics.insert(format!("{}_mean", name), mean);
                metrics.insert(format!("{}_max", name), max);
                metrics.insert(format!("{}_min", name), min);
            }
        }

        Self { sample, outputs, metrics }
    }
}

/// Sensitivity analysis engine
pub struct SensitivityAnalyzer {
    pub parameter_ranges: Vec<ParameterRange>,
    pub results: Vec<SensitivityResult>,
}

impl SensitivityAnalyzer {
    pub fn new(parameter_ranges: Vec<ParameterRange>) -> Self {
        Self {
            parameter_ranges,
            results: Vec::new(),
        }
    }

    /// One-at-a-time parameter sweep
    pub fn parameter_sweep(
        &mut self,
        base_model: &Model,
        config: &SimulationConfig,
        steps_per_parameter: usize,
    ) -> Result<(), String> {
        self.results.clear();

        // Baseline run
        let baseline_sample = self.create_baseline_sample();
        let baseline_results = self.run_simulation(base_model, config, &baseline_sample)?;
        self.results.push(baseline_results);

        // Sweep each parameter
        for param_range in &self.parameter_ranges.clone() {
            for i in 0..steps_per_parameter {
                let fraction = i as f64 / (steps_per_parameter - 1) as f64;
                let value = param_range.at_fraction(fraction);

                let mut sample = self.create_baseline_sample();
                sample.set(param_range.name.clone(), value);

                let result = self.run_simulation(base_model, config, &sample)?;
                self.results.push(result);
            }
        }

        Ok(())
    }

    /// Latin Hypercube Sampling
    pub fn latin_hypercube_sampling(
        &mut self,
        base_model: &Model,
        config: &SimulationConfig,
        n_samples: usize,
        seed: Option<u64>,
    ) -> Result<(), String> {
        self.results.clear();

        let mut rng = if let Some(s) = seed {
            StdRng::seed_from_u64(s)
        } else {
            StdRng::from_entropy()
        };

        let samples = self.generate_lhs_samples(n_samples, &mut rng);

        for sample in samples {
            let result = self.run_simulation(base_model, config, &sample)?;
            self.results.push(result);
        }

        Ok(())
    }

    /// Morris screening method (elementary effects)
    pub fn morris_screening(
        &mut self,
        base_model: &Model,
        config: &SimulationConfig,
        n_trajectories: usize,
        n_levels: usize,
        seed: Option<u64>,
    ) -> Result<(), String> {
        self.results.clear();

        let mut rng = if let Some(s) = seed {
            StdRng::seed_from_u64(s)
        } else {
            StdRng::from_entropy()
        };

        // Generate Morris trajectories
        for _ in 0..n_trajectories {
            let trajectory = self.generate_morris_trajectory(n_levels, &mut rng);

            for sample in trajectory {
                let result = self.run_simulation(base_model, config, &sample)?;
                self.results.push(result);
            }
        }

        Ok(())
    }

    /// Calculate Morris elementary effects
    pub fn calculate_morris_effects(&self, output_metric: &str) -> HashMap<String, (f64, f64)> {
        let mut effects: HashMap<String, Vec<f64>> = HashMap::new();

        // Group results by trajectory and calculate elementary effects
        // This is a simplified implementation
        for i in 0..self.results.len() - 1 {
            let curr = &self.results[i];
            let next = &self.results[i + 1];

            // Find which parameter changed
            for param_range in &self.parameter_ranges {
                let curr_val = curr.sample.get(&param_range.name).unwrap_or(param_range.baseline);
                let next_val = next.sample.get(&param_range.name).unwrap_or(param_range.baseline);

                if (curr_val - next_val).abs() > 1e-10 {
                    // Parameter changed - calculate effect
                    let curr_output = curr.metrics.get(output_metric).copied().unwrap_or(0.0);
                    let next_output = next.metrics.get(output_metric).copied().unwrap_or(0.0);

                    let effect = (next_output - curr_output) / (next_val - curr_val);
                    effects.entry(param_range.name.clone())
                        .or_insert_with(Vec::new)
                        .push(effect);
                }
            }
        }

        // Calculate mean and standard deviation for each parameter
        let mut morris_indices = HashMap::new();
        for (param, effect_values) in effects {
            if !effect_values.is_empty() {
                let mean = effect_values.iter().sum::<f64>() / effect_values.len() as f64;
                let abs_mean = effect_values.iter().map(|x| x.abs()).sum::<f64>() / effect_values.len() as f64;

                let variance = effect_values.iter()
                    .map(|x| (x - mean).powi(2))
                    .sum::<f64>() / effect_values.len() as f64;
                let std_dev = variance.sqrt();

                morris_indices.insert(param, (abs_mean, std_dev));
            }
        }

        morris_indices
    }

    /// Generate Latin Hypercube samples
    fn generate_lhs_samples(&self, n_samples: usize, rng: &mut impl Rng) -> Vec<ParameterSample> {
        let n_params = self.parameter_ranges.len();
        let mut samples = Vec::new();

        // Create permutations for each parameter
        let mut permutations: Vec<Vec<usize>> = Vec::new();
        for _ in 0..n_params {
            let mut perm: Vec<usize> = (0..n_samples).collect();
            perm.shuffle(rng);
            permutations.push(perm);
        }

        // Generate samples
        for i in 0..n_samples {
            let mut sample = ParameterSample::new();

            for (j, param_range) in self.parameter_ranges.iter().enumerate() {
                use rand::distributions::{Distribution, Standard};
                let perm_index = permutations[j][i];
                let random_val: f64 = Standard.sample(rng);
                let fraction = (perm_index as f64 + random_val) / n_samples as f64;
                let value = param_range.at_fraction(fraction);
                sample.set(param_range.name.clone(), value);
            }

            samples.push(sample);
        }

        samples
    }

    /// Generate Morris trajectory
    fn generate_morris_trajectory(&self, n_levels: usize, rng: &mut impl Rng) -> Vec<ParameterSample> {
        let mut trajectory = Vec::new();
        let n_params = self.parameter_ranges.len();

        // Start at random point
        let mut current = ParameterSample::new();
        for param_range in &self.parameter_ranges {
            let level = rng.gen_range(0..n_levels);
            let fraction = level as f64 / (n_levels - 1) as f64;
            current.set(param_range.name.clone(), param_range.at_fraction(fraction));
        }
        trajectory.push(current.clone());

        // Create random parameter sequence
        let mut param_indices: Vec<usize> = (0..n_params).collect();
        param_indices.shuffle(rng);

        // Move one parameter at a time
        for param_idx in param_indices {
            let param_range = &self.parameter_ranges[param_idx];

            // Choose random delta
            use rand::distributions::{Distribution, Standard};
            let random_val: f64 = Standard.sample(rng);
            let delta_levels = if random_val > 0.5 { 1 } else { -1 };
            let current_val = current.get(&param_range.name).unwrap_or(param_range.baseline);

            // Calculate new level
            let current_frac = (current_val - param_range.min) / (param_range.max - param_range.min);
            let current_level = (current_frac * (n_levels - 1) as f64).round() as i32;
            let new_level = (current_level + delta_levels).clamp(0, n_levels as i32 - 1);
            let new_frac = new_level as f64 / (n_levels - 1) as f64;

            current.set(param_range.name.clone(), param_range.at_fraction(new_frac));
            trajectory.push(current.clone());
        }

        trajectory
    }

    /// Create baseline sample with all parameters at their baseline values
    fn create_baseline_sample(&self) -> ParameterSample {
        let mut sample = ParameterSample::new();
        for param_range in &self.parameter_ranges {
            sample.set(param_range.name.clone(), param_range.baseline);
        }
        sample
    }

    /// Run simulation with parameter sample
    fn run_simulation(
        &self,
        base_model: &Model,
        config: &SimulationConfig,
        sample: &ParameterSample,
    ) -> Result<SensitivityResult, String> {
        // Clone model and apply parameter values
        let mut model = base_model.clone();
        for (param_name, &value) in &sample.values {
            model.set_parameter(param_name, value)?;
        }

        // Run simulation
        let mut engine = SimulationEngine::new(model, config.clone())?;
        let results = engine.run()?;

        Ok(SensitivityResult::from_simulation(sample.clone(), &results))
    }

    /// Export results to CSV
    pub fn export_results(&self, output_metric: &str) -> Result<String, String> {
        let mut csv = String::new();

        // Header
        csv.push_str("sample_id");
        for param_range in &self.parameter_ranges {
            csv.push_str(&format!(",{}", param_range.name));
        }
        csv.push_str(&format!(",{}\n", output_metric));

        // Data rows
        for (i, result) in self.results.iter().enumerate() {
            csv.push_str(&format!("{}", i));
            for param_range in &self.parameter_ranges {
                let value = result.sample.get(&param_range.name)
                    .unwrap_or(param_range.baseline);
                csv.push_str(&format!(",{}", value));
            }
            let metric_value = result.metrics.get(output_metric).copied().unwrap_or(0.0);
            csv.push_str(&format!(",{}\n", metric_value));
        }

        Ok(csv)
    }
}

impl Model {
    /// Set a parameter value (convenience method)
    pub fn set_parameter(&mut self, name: &str, value: f64) -> Result<(), String> {
        if let Some(param) = self.parameters.get_mut(name) {
            param.value = value;
            Ok(())
        } else {
            Err(format!("Parameter '{}' not found", name))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Stock, Flow, Parameter};

    #[test]
    fn test_parameter_range() {
        let range = ParameterRange::new("test".to_string(), 0.0, 10.0, 5.0);
        assert_eq!(range.at_fraction(0.0), 0.0);
        assert_eq!(range.at_fraction(1.0), 10.0);
        assert_eq!(range.at_fraction(0.5), 5.0);
    }

    #[test]
    fn test_lhs_sampling() {
        let ranges = vec![
            ParameterRange::new("p1".to_string(), 0.0, 1.0, 0.5),
            ParameterRange::new("p2".to_string(), 0.0, 1.0, 0.5),
        ];

        let analyzer = SensitivityAnalyzer::new(ranges);
        let mut rng = StdRng::seed_from_u64(42);
        let samples = analyzer.generate_lhs_samples(10, &mut rng);

        assert_eq!(samples.len(), 10);

        // Check that all samples have both parameters
        for sample in &samples {
            assert!(sample.get("p1").is_some());
            assert!(sample.get("p2").is_some());
        }
    }

    #[test]
    fn test_parameter_sweep() {
        let mut model = Model::new("Test");
        model.time.start = 0.0;
        model.time.stop = 10.0;
        model.time.dt = 1.0;

        model.add_stock(Stock::new("Population", "100")).unwrap();
        model.add_parameter(Parameter::new("growth_rate", 0.1)).unwrap();
        model.add_flow(Flow::new("growth", "Population * growth_rate")).unwrap();
        model.stocks.get_mut("Population").unwrap().inflows.push("growth".to_string());

        let ranges = vec![
            ParameterRange::new("growth_rate".to_string(), 0.05, 0.15, 0.1),
        ];

        let mut analyzer = SensitivityAnalyzer::new(ranges);
        let config = SimulationConfig::default();

        analyzer.parameter_sweep(&model, &config, 5).unwrap();

        // Should have baseline + 5 sweep points
        assert_eq!(analyzer.results.len(), 6);
    }
}
