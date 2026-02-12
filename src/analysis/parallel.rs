/// Parallel Monte Carlo and sensitivity analysis using rayon
///
/// Provides high-performance parallel versions of analysis algorithms
/// with optional ARM NEON optimizations

use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::model::Model;
use crate::simulation::{SimulationEngine, SimulationConfig, SimulationResults};
use crate::analysis::sensitivity::{ParameterRange, ParameterSample, SensitivityResult};
use crate::analysis::monte_carlo::{MonteCarloConfig, MonteCarloResults, TimeSeriesStatistics};
use rand::prelude::*;
use rand::rngs::StdRng;

/// Parallel Monte Carlo simulator
pub struct ParallelMonteCarloSimulator {
    pub parameter_ranges: Vec<ParameterRange>,
    pub mc_config: MonteCarloConfig,
}

impl ParallelMonteCarloSimulator {
    pub fn new(parameter_ranges: Vec<ParameterRange>, mc_config: MonteCarloConfig) -> Self {
        Self {
            parameter_ranges,
            mc_config,
        }
    }

    /// Run parallel Monte Carlo simulation
    pub fn run(
        &self,
        base_model: &Model,
        sim_config: &SimulationConfig,
    ) -> Result<MonteCarloResults, String> {
        let n_runs = self.mc_config.n_runs;

        // Generate all parameter samples upfront
        let samples: Vec<ParameterSample> = self.generate_samples(n_runs);

        // Run simulations in parallel
        let results: Vec<(usize, Result<HashMap<String, Vec<f64>>, String>)> = samples
            .par_iter()
            .enumerate()
            .map(|(run_idx, sample)| {
                let result = self.run_single_simulation(base_model, sim_config, sample);
                (run_idx, result)
            })
            .collect();

        // Aggregate results
        self.aggregate_results(results, base_model)
    }

    /// Generate parameter samples
    fn generate_samples(&self, n_runs: usize) -> Vec<ParameterSample> {
        let mut rng = match self.mc_config.seed {
            Some(seed) => StdRng::seed_from_u64(seed),
            None => StdRng::from_entropy(),
        };

        (0..n_runs)
            .map(|_| self.sample_parameters(&mut rng))
            .collect()
    }

    /// Sample parameters from ranges
    fn sample_parameters(&self, rng: &mut StdRng) -> ParameterSample {
        let mut sample = ParameterSample {
            values: HashMap::new(),
        };

        for param_range in &self.parameter_ranges {
            let value = param_range.sample_uniform(rng);
            sample.values.insert(param_range.name.clone(), value);
        }

        sample
    }

    /// Run a single simulation with given parameters
    fn run_single_simulation(
        &self,
        base_model: &Model,
        sim_config: &SimulationConfig,
        sample: &ParameterSample,
    ) -> Result<HashMap<String, Vec<f64>>, String> {
        // Clone model and update parameters
        let mut model = base_model.clone();
        for (param_name, &param_value) in &sample.values {
            if let Some(param) = model.parameters.get_mut(param_name) {
                param.value = param_value;
            }
        }

        // Run simulation
        let mut engine = SimulationEngine::new(model.clone(), sim_config.clone())?;
        let results = engine.run()?;

        // Extract time series for all variables
        let mut variable_data = HashMap::new();

        // Get stock names
        for stock_name in model.stocks.keys() {
            if let Some(series) = results.get_variable_series(stock_name) {
                variable_data.insert(stock_name.clone(), series);
            }
        }

        Ok(variable_data)
    }

    /// Aggregate results from all runs
    fn aggregate_results(
        &self,
        results: Vec<(usize, Result<HashMap<String, Vec<f64>>, String>)>,
        base_model: &Model,
    ) -> Result<MonteCarloResults, String> {
        // Check for errors
        let mut successful_results = Vec::new();
        for (idx, result) in results {
            match result {
                Ok(data) => successful_results.push(data),
                Err(e) => return Err(format!("Run {} failed: {}", idx, e)),
            }
        }

        if successful_results.is_empty() {
            return Err("No successful runs".to_string());
        }

        // Get time points from first result
        let first_result = &successful_results[0];
        let n_points = first_result.values().next()
            .ok_or("No variables in results")?
            .len();

        // Get variable names
        let var_names: Vec<String> = base_model.stocks.keys().cloned().collect();

        // Compute statistics for each variable
        let statistics: HashMap<String, TimeSeriesStatistics> = var_names
            .par_iter()
            .map(|var_name| {
                let stats = self.compute_statistics(var_name, &successful_results, n_points);
                (var_name.clone(), stats)
            })
            .collect();

        // Extract time points (assuming all runs have same time points)
        let time: Vec<f64> = (0..n_points)
            .map(|i| base_model.time.start + i as f64 * base_model.time.dt)
            .collect();

        // Get n_runs before moving successful_results
        let n_runs = successful_results.len();

        // Save individual runs if requested
        let individual_runs = if self.mc_config.save_individual_runs {
            Some(successful_results)
        } else {
            None
        };

        Ok(MonteCarloResults {
            n_runs,
            time,
            statistics,
            individual_runs,
        })
    }

    /// Compute statistics for a single variable across all runs
    #[cfg(not(all(target_arch = "aarch64", feature = "neon")))]
    fn compute_statistics(
        &self,
        var_name: &str,
        results: &[HashMap<String, Vec<f64>>],
        n_points: usize,
    ) -> TimeSeriesStatistics {
        self.compute_statistics_scalar(var_name, results, n_points)
    }

    /// Compute statistics using ARM NEON optimizations
    #[cfg(all(target_arch = "aarch64", feature = "neon"))]
    fn compute_statistics(
        &self,
        var_name: &str,
        results: &[HashMap<String, Vec<f64>>],
        n_points: usize,
    ) -> TimeSeriesStatistics {
        self.compute_statistics_neon(var_name, results, n_points)
    }

    /// Scalar implementation of statistics computation
    fn compute_statistics_scalar(
        &self,
        var_name: &str,
        results: &[HashMap<String, Vec<f64>>],
        n_points: usize,
    ) -> TimeSeriesStatistics {
        let mut stats = TimeSeriesStatistics::new(n_points);
        let n_runs = results.len() as f64;

        for t in 0..n_points {
            // Collect values at this time point
            let mut values: Vec<f64> = results.iter()
                .filter_map(|run| run.get(var_name).and_then(|series| series.get(t)))
                .copied()
                .collect();

            if values.is_empty() {
                continue;
            }

            // Sort for percentiles
            values.sort_by(|a, b| a.partial_cmp(b).unwrap());

            // Mean
            let mean: f64 = values.iter().sum::<f64>() / values.len() as f64;
            stats.mean[t] = mean;

            // Variance and std dev
            let variance: f64 = values.iter()
                .map(|x| (x - mean).powi(2))
                .sum::<f64>() / values.len() as f64;
            stats.std_dev[t] = variance.sqrt();

            // Min/max
            stats.min[t] = values[0];
            stats.max[t] = values[values.len() - 1];

            // Percentiles
            stats.percentile_5[t] = Self::percentile(&values, 0.05);
            stats.percentile_25[t] = Self::percentile(&values, 0.25);
            stats.percentile_50[t] = Self::percentile(&values, 0.50);
            stats.percentile_75[t] = Self::percentile(&values, 0.75);
            stats.percentile_95[t] = Self::percentile(&values, 0.95);

            // Confidence intervals (assuming normal distribution)
            let se = stats.std_dev[t] / n_runs.sqrt();
            let z = Self::z_score(self.mc_config.confidence_level);
            stats.lower_ci[t] = mean - z * se;
            stats.upper_ci[t] = mean + z * se;
        }

        stats
    }

    /// ARM NEON-optimized statistics computation
    #[cfg(all(target_arch = "aarch64", feature = "neon"))]
    fn compute_statistics_neon(
        &self,
        var_name: &str,
        results: &[HashMap<String, Vec<f64>>],
        n_points: usize,
    ) -> TimeSeriesStatistics {
        use std::arch::aarch64::*;

        let mut stats = TimeSeriesStatistics::new(n_points);
        let n_runs = results.len() as f64;

        for t in 0..n_points {
            // Collect values at this time point
            let mut values: Vec<f64> = results.iter()
                .filter_map(|run| run.get(var_name).and_then(|series| series.get(t)))
                .copied()
                .collect();

            if values.is_empty() {
                continue;
            }

            // Use NEON for mean calculation if we have enough values
            let mean = if values.len() >= 4 {
                unsafe { Self::neon_mean(&values) }
            } else {
                values.iter().sum::<f64>() / values.len() as f64
            };

            stats.mean[t] = mean;

            // Sort for percentiles
            values.sort_by(|a, b| a.partial_cmp(b).unwrap());

            // Use NEON for variance if we have enough values
            let variance = if values.len() >= 4 {
                unsafe { Self::neon_variance(&values, mean) }
            } else {
                values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / values.len() as f64
            };

            stats.std_dev[t] = variance.sqrt();

            // Min/max
            stats.min[t] = values[0];
            stats.max[t] = values[values.len() - 1];

            // Percentiles
            stats.percentile_5[t] = Self::percentile(&values, 0.05);
            stats.percentile_25[t] = Self::percentile(&values, 0.25);
            stats.percentile_50[t] = Self::percentile(&values, 0.50);
            stats.percentile_75[t] = Self::percentile(&values, 0.75);
            stats.percentile_95[t] = Self::percentile(&values, 0.95);

            // Confidence intervals
            let se = stats.std_dev[t] / n_runs.sqrt();
            let z = Self::z_score(self.mc_config.confidence_level);
            stats.lower_ci[t] = mean - z * se;
            stats.upper_ci[t] = mean + z * se;
        }

        stats
    }

    /// NEON-optimized mean calculation
    #[cfg(all(target_arch = "aarch64", feature = "neon"))]
    unsafe fn neon_mean(values: &[f64]) -> f64 {
        use std::arch::aarch64::*;

        let mut sum = vdupq_n_f64(0.0);
        let chunks = values.chunks_exact(2);
        let remainder = chunks.remainder();

        for chunk in chunks {
            let v = vld1q_f64(chunk.as_ptr());
            sum = vaddq_f64(sum, v);
        }

        // Sum the vector
        let mut total = vgetq_lane_f64(sum, 0) + vgetq_lane_f64(sum, 1);

        // Add remainder
        total += remainder.iter().sum::<f64>();

        total / values.len() as f64
    }

    /// NEON-optimized variance calculation
    #[cfg(all(target_arch = "aarch64", feature = "neon"))]
    unsafe fn neon_variance(values: &[f64], mean: f64) -> f64 {
        use std::arch::aarch64::*;

        let mean_vec = vdupq_n_f64(mean);
        let mut sum_sq = vdupq_n_f64(0.0);

        let chunks = values.chunks_exact(2);
        let remainder = chunks.remainder();

        for chunk in chunks {
            let v = vld1q_f64(chunk.as_ptr());
            let diff = vsubq_f64(v, mean_vec);
            sum_sq = vfmaq_f64(sum_sq, diff, diff);  // sum_sq += diff * diff
        }

        // Sum the vector
        let mut total = vgetq_lane_f64(sum_sq, 0) + vgetq_lane_f64(sum_sq, 1);

        // Add remainder
        total += remainder.iter().map(|x| (x - mean).powi(2)).sum::<f64>();

        total / values.len() as f64
    }

    /// Calculate percentile from sorted data
    fn percentile(sorted_data: &[f64], p: f64) -> f64 {
        if sorted_data.is_empty() {
            return 0.0;
        }

        let idx = (p * (sorted_data.len() - 1) as f64).floor() as usize;
        let frac = p * (sorted_data.len() - 1) as f64 - idx as f64;

        if idx + 1 < sorted_data.len() {
            sorted_data[idx] * (1.0 - frac) + sorted_data[idx + 1] * frac
        } else {
            sorted_data[idx]
        }
    }

    /// Get z-score for confidence level
    fn z_score(confidence: f64) -> f64 {
        // Approximate z-scores for common confidence levels
        match confidence {
            c if (c - 0.90).abs() < 0.01 => 1.645,
            c if (c - 0.95).abs() < 0.01 => 1.96,
            c if (c - 0.99).abs() < 0.01 => 2.576,
            _ => 1.96,  // Default to 95%
        }
    }
}

/// Parallel sensitivity analyzer
pub struct ParallelSensitivityAnalyzer {
    pub parameter_ranges: Vec<ParameterRange>,
    pub n_samples: usize,
}

impl ParallelSensitivityAnalyzer {
    pub fn new(parameter_ranges: Vec<ParameterRange>, n_samples: usize) -> Self {
        Self {
            parameter_ranges,
            n_samples,
        }
    }

    /// Run parallel sensitivity analysis
    pub fn run(
        &self,
        base_model: &Model,
        sim_config: &SimulationConfig,
        output_variable: &str,
    ) -> Result<HashMap<String, SensitivityResult>, String> {
        // Analyze each parameter in parallel
        let results: Vec<(String, SensitivityResult)> = self.parameter_ranges
            .par_iter()
            .map(|param_range| {
                let result = self.analyze_parameter(base_model, sim_config, param_range, output_variable)?;
                Ok((param_range.name.clone(), result))
            })
            .collect::<Result<Vec<_>, String>>()?;

        Ok(results.into_iter().collect())
    }

    /// Analyze a single parameter
    fn analyze_parameter(
        &self,
        base_model: &Model,
        sim_config: &SimulationConfig,
        param_range: &ParameterRange,
        output_variable: &str,
    ) -> Result<SensitivityResult, String> {
        let mut rng = StdRng::from_entropy();

        // Generate samples for this parameter
        let samples: Vec<f64> = (0..self.n_samples)
            .map(|_| rng.gen_range(param_range.min..=param_range.max))
            .collect();

        // Run simulations for each sample value
        let outputs: Vec<f64> = samples
            .par_iter()
            .map(|&param_value| {
                let mut model = base_model.clone();
                if let Some(param) = model.parameters.get_mut(&param_range.name) {
                    param.value = param_value;
                }

                let mut engine = SimulationEngine::new(model, sim_config.clone())?;
                let results = engine.run()?;

                // Get final value of output variable
                let final_value = results.get_variable_series(output_variable)
                    .and_then(|series| series.last().copied())
                    .ok_or_else(|| format!("Output variable '{}' not found", output_variable))?;

                Ok(final_value)
            })
            .collect::<Result<Vec<_>, String>>()?;

        // Compute sensitivity metrics
        let correlation = Self::compute_correlation(&samples, &outputs);
        let partial_correlation = correlation;  // Simplified

        // Compute elasticity at mean
        let mean_input = samples.iter().sum::<f64>() / samples.len() as f64;
        let mean_output = outputs.iter().sum::<f64>() / outputs.len() as f64;
        let elasticity = if mean_output.abs() > 1e-10 {
            (correlation * samples.iter().map(|x| (x - mean_input).powi(2)).sum::<f64>().sqrt() / samples.len() as f64)
                / mean_output
        } else {
            0.0
        };

        // Create a sample with all parameter values
        let mut sample = ParameterSample { values: HashMap::new() };
        sample.values.insert(param_range.name.clone(), param_range.baseline);

        // Create output map with time series
        let mut output_map = HashMap::new();
        output_map.insert(output_variable.to_string(), outputs.clone());

        // Create metrics map
        let mut metrics = HashMap::new();
        metrics.insert("correlation".to_string(), correlation);
        metrics.insert("partial_correlation".to_string(), partial_correlation);
        metrics.insert("elasticity".to_string(), elasticity);

        Ok(SensitivityResult {
            sample,
            outputs: output_map,
            metrics,
        })
    }

    /// Compute Pearson correlation coefficient
    fn compute_correlation(x: &[f64], y: &[f64]) -> f64 {
        let n = x.len() as f64;
        let mean_x = x.iter().sum::<f64>() / n;
        let mean_y = y.iter().sum::<f64>() / n;

        let cov: f64 = x.iter().zip(y.iter())
            .map(|(&xi, &yi)| (xi - mean_x) * (yi - mean_y))
            .sum::<f64>() / n;

        let var_x = x.iter().map(|&xi| (xi - mean_x).powi(2)).sum::<f64>() / n;
        let var_y = y.iter().map(|&yi| (yi - mean_y).powi(2)).sum::<f64>() / n;

        if var_x > 0.0 && var_y > 0.0 {
            cov / (var_x.sqrt() * var_y.sqrt())
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Model, Stock, Flow, Parameter};
    use crate::simulation::IntegrationMethod;

    #[test]
    fn test_parallel_monte_carlo() {
        let mut model = Model::new("Test");
        model.time.start = 0.0;
        model.time.stop = 10.0;
        model.time.dt = 1.0;

        model.add_stock(Stock::new("X", "10.0")).unwrap();
        model.add_parameter(Parameter::new("k", 0.1)).unwrap();
        model.add_flow(Flow::new("growth", "k * X")).unwrap();
        model.stocks.get_mut("X").unwrap().inflows.push("growth".to_string());

        let param_ranges = vec![
            ParameterRange::new("k".to_string(), 0.05, 0.15, 0.1),
        ];

        let mc_config = MonteCarloConfig {
            n_runs: 10,
            seed: Some(42),
            confidence_level: 0.95,
            save_individual_runs: false,
        };

        let simulator = ParallelMonteCarloSimulator::new(param_ranges, mc_config);
        let sim_config = SimulationConfig {
            integration_method: IntegrationMethod::Euler,
            output_interval: None,
        };

        let results = simulator.run(&model, &sim_config).unwrap();

        assert_eq!(results.n_runs, 10);
        assert!(results.statistics.contains_key("X"));
    }
}
