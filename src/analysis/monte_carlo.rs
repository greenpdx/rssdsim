/// Monte Carlo simulation framework
///
/// Provides tools for:
/// - Running multiple simulations with random parameter values
/// - Aggregating results across runs
/// - Statistical analysis (mean, std dev, percentiles, confidence intervals)
/// - Uncertainty quantification

use std::collections::HashMap;
use rand::prelude::*;
use crate::model::Model;
use crate::simulation::{SimulationEngine, SimulationConfig, SimulationResults};
use crate::analysis::sensitivity::{ParameterRange, ParameterSample};

/// Monte Carlo simulation configuration
#[derive(Debug, Clone)]
pub struct MonteCarloConfig {
    /// Number of simulation runs
    pub n_runs: usize,

    /// Random seed for reproducibility
    pub seed: Option<u64>,

    /// Confidence level for intervals (e.g., 0.95 for 95%)
    pub confidence_level: f64,

    /// Whether to save individual run results or just aggregates
    pub save_individual_runs: bool,
}

impl Default for MonteCarloConfig {
    fn default() -> Self {
        Self {
            n_runs: 100,
            seed: None,
            confidence_level: 0.95,
            save_individual_runs: false,
        }
    }
}

/// Results from Monte Carlo analysis
#[derive(Debug, Clone)]
pub struct MonteCarloResults {
    /// Number of runs completed
    pub n_runs: usize,

    /// Time points (same for all runs)
    pub time: Vec<f64>,

    /// Statistical summaries for each variable
    pub statistics: HashMap<String, TimeSeriesStatistics>,

    /// Individual run results (if saved)
    pub individual_runs: Option<Vec<HashMap<String, Vec<f64>>>>,
}

/// Statistical summary for a time series
#[derive(Debug, Clone)]
pub struct TimeSeriesStatistics {
    pub mean: Vec<f64>,
    pub std_dev: Vec<f64>,
    pub min: Vec<f64>,
    pub max: Vec<f64>,
    pub percentile_5: Vec<f64>,
    pub percentile_25: Vec<f64>,
    pub percentile_50: Vec<f64>,  // median
    pub percentile_75: Vec<f64>,
    pub percentile_95: Vec<f64>,
    pub lower_ci: Vec<f64>,  // Lower confidence interval
    pub upper_ci: Vec<f64>,  // Upper confidence interval
}

impl TimeSeriesStatistics {
    pub fn new(n_points: usize) -> Self {
        Self {
            mean: vec![0.0; n_points],
            std_dev: vec![0.0; n_points],
            min: vec![0.0; n_points],
            max: vec![0.0; n_points],
            percentile_5: vec![0.0; n_points],
            percentile_25: vec![0.0; n_points],
            percentile_50: vec![0.0; n_points],
            percentile_75: vec![0.0; n_points],
            percentile_95: vec![0.0; n_points],
            lower_ci: vec![0.0; n_points],
            upper_ci: vec![0.0; n_points],
        }
    }
}

/// Monte Carlo simulator
pub struct MonteCarloSimulator {
    pub parameter_ranges: Vec<ParameterRange>,
    pub mc_config: MonteCarloConfig,
}

impl MonteCarloSimulator {
    pub fn new(parameter_ranges: Vec<ParameterRange>, mc_config: MonteCarloConfig) -> Self {
        Self {
            parameter_ranges,
            mc_config,
        }
    }

    /// Run Monte Carlo simulation
    pub fn run(
        &self,
        base_model: &Model,
        sim_config: &SimulationConfig,
    ) -> Result<MonteCarloResults, String> {
        let mut rng = if let Some(seed) = self.mc_config.seed {
            StdRng::seed_from_u64(seed)
        } else {
            StdRng::from_entropy()
        };

        // Storage for all runs
        let mut all_runs: Vec<HashMap<String, Vec<f64>>> = Vec::new();
        let mut time_vec: Option<Vec<f64>> = None;

        // Run simulations
        for run_idx in 0..self.mc_config.n_runs {
            // Sample parameters
            let sample = self.sample_parameters(&mut rng);

            // Run simulation
            let run_results = self.run_single_simulation(base_model, sim_config, &sample)?;

            // Extract time series
            let mut run_data = HashMap::new();

            // Extract time if first run
            if time_vec.is_none() {
                time_vec = Some(run_results.states.iter().map(|s| s.time).collect());
            }

            // Extract all variable time series
            for state in &run_results.states {
                for (name, &value) in &state.stocks {
                    run_data.entry(name.clone()).or_insert_with(Vec::new).push(value);
                }
                for (name, &value) in &state.flows {
                    run_data.entry(name.clone()).or_insert_with(Vec::new).push(value);
                }
                for (name, &value) in &state.auxiliaries {
                    run_data.entry(name.clone()).or_insert_with(Vec::new).push(value);
                }
            }

            all_runs.push(run_data);

            if (run_idx + 1) % 10 == 0 {
                eprintln!("Completed {}/{} Monte Carlo runs", run_idx + 1, self.mc_config.n_runs);
            }
        }

        let time = time_vec.ok_or("No simulation results generated")?;

        // Calculate statistics
        let statistics = self.calculate_statistics(&all_runs, time.len())?;

        // Prepare results
        let individual_runs = if self.mc_config.save_individual_runs {
            Some(all_runs)
        } else {
            None
        };

        Ok(MonteCarloResults {
            n_runs: self.mc_config.n_runs,
            time,
            statistics,
            individual_runs,
        })
    }

    /// Sample parameters uniformly from ranges
    fn sample_parameters(&self, rng: &mut impl Rng) -> ParameterSample {
        let mut sample = ParameterSample::new();

        for param_range in &self.parameter_ranges {
            let value = param_range.sample_uniform(rng);
            sample.set(param_range.name.clone(), value);
        }

        sample
    }

    /// Run single simulation with parameter sample
    fn run_single_simulation(
        &self,
        base_model: &Model,
        config: &SimulationConfig,
        sample: &ParameterSample,
    ) -> Result<SimulationResults, String> {
        let mut model = base_model.clone();

        // Apply parameter values
        for (param_name, &value) in &sample.values {
            model.set_parameter(param_name, value)?;
        }

        // Run simulation
        let mut engine = SimulationEngine::new(model, config.clone())?;
        engine.run()
    }

    /// Calculate statistics across all runs
    fn calculate_statistics(
        &self,
        all_runs: &[HashMap<String, Vec<f64>>],
        n_points: usize,
    ) -> Result<HashMap<String, TimeSeriesStatistics>, String> {
        let mut statistics = HashMap::new();

        if all_runs.is_empty() {
            return Ok(statistics);
        }

        // Get list of all variables
        let variables: Vec<String> = all_runs[0].keys().cloned().collect();

        for var_name in variables {
            let mut stats = TimeSeriesStatistics::new(n_points);

            // For each time point
            for t in 0..n_points {
                // Collect values across all runs
                let mut values: Vec<f64> = all_runs
                    .iter()
                    .filter_map(|run| run.get(&var_name).and_then(|ts| ts.get(t)))
                    .copied()
                    .collect();

                if values.is_empty() {
                    continue;
                }

                values.sort_by(|a, b| a.partial_cmp(b).unwrap());

                // Calculate statistics
                stats.mean[t] = values.iter().sum::<f64>() / values.len() as f64;
                stats.min[t] = values[0];
                stats.max[t] = values[values.len() - 1];

                // Standard deviation
                let mean = stats.mean[t];
                let variance = values.iter()
                    .map(|v| (v - mean).powi(2))
                    .sum::<f64>() / values.len() as f64;
                stats.std_dev[t] = variance.sqrt();

                // Percentiles
                stats.percentile_5[t] = Self::percentile(&values, 0.05);
                stats.percentile_25[t] = Self::percentile(&values, 0.25);
                stats.percentile_50[t] = Self::percentile(&values, 0.50);
                stats.percentile_75[t] = Self::percentile(&values, 0.75);
                stats.percentile_95[t] = Self::percentile(&values, 0.95);

                // Confidence intervals (using percentiles)
                let alpha = 1.0 - self.mc_config.confidence_level;
                stats.lower_ci[t] = Self::percentile(&values, alpha / 2.0);
                stats.upper_ci[t] = Self::percentile(&values, 1.0 - alpha / 2.0);
            }

            statistics.insert(var_name, stats);
        }

        Ok(statistics)
    }

    /// Calculate percentile from sorted values
    fn percentile(sorted_values: &[f64], p: f64) -> f64 {
        let n = sorted_values.len();
        if n == 0 {
            return 0.0;
        }

        let index = p * (n - 1) as f64;
        let lower = index.floor() as usize;
        let upper = index.ceil() as usize;

        if lower == upper {
            sorted_values[lower]
        } else {
            let weight = index - lower as f64;
            sorted_values[lower] * (1.0 - weight) + sorted_values[upper] * weight
        }
    }

    /// Export results to CSV
    pub fn export_csv(
        &self,
        results: &MonteCarloResults,
        variable_name: &str,
    ) -> Result<String, String> {
        let stats = results.statistics.get(variable_name)
            .ok_or_else(|| format!("Variable '{}' not found in results", variable_name))?;

        let mut csv = String::new();

        // Header
        csv.push_str("time,mean,std_dev,min,max,p5,p25,median,p75,p95,lower_ci,upper_ci\n");

        // Data rows
        for i in 0..results.time.len() {
            csv.push_str(&format!(
                "{},{},{},{},{},{},{},{},{},{},{},{}\n",
                results.time[i],
                stats.mean[i],
                stats.std_dev[i],
                stats.min[i],
                stats.max[i],
                stats.percentile_5[i],
                stats.percentile_25[i],
                stats.percentile_50[i],
                stats.percentile_75[i],
                stats.percentile_95[i],
                stats.lower_ci[i],
                stats.upper_ci[i],
            ));
        }

        Ok(csv)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Stock, Flow, Parameter};

    #[test]
    fn test_monte_carlo_basic() {
        let mut model = Model::new("Test");
        model.time.start = 0.0;
        model.time.stop = 10.0;
        model.time.dt = 1.0;

        model.add_stock(Stock::new("Population", "100")).unwrap();
        model.add_parameter(Parameter::new("growth_rate", 0.1)).unwrap();
        model.add_flow(Flow::new("growth", "Population * growth_rate")).unwrap();
        model.stocks.get_mut("Population").unwrap().inflows.push("growth".to_string());

        let param_ranges = vec![
            ParameterRange::new("growth_rate".to_string(), 0.05, 0.15, 0.1),
        ];

        let mc_config = MonteCarloConfig {
            n_runs: 10,
            seed: Some(42),
            confidence_level: 0.95,
            save_individual_runs: false,
        };

        let simulator = MonteCarloSimulator::new(param_ranges, mc_config);
        let sim_config = SimulationConfig::default();

        let results = simulator.run(&model, &sim_config).unwrap();

        assert_eq!(results.n_runs, 10);
        assert!(results.statistics.contains_key("Population"));
    }

    #[test]
    fn test_percentile_calculation() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];

        assert_eq!(MonteCarloSimulator::percentile(&values, 0.0), 1.0);
        assert_eq!(MonteCarloSimulator::percentile(&values, 0.5), 3.0);
        assert_eq!(MonteCarloSimulator::percentile(&values, 1.0), 5.0);
    }

    #[test]
    fn test_csv_export() {
        let mut model = Model::new("Test");
        model.time.start = 0.0;
        model.time.stop = 5.0;
        model.time.dt = 1.0;

        model.add_stock(Stock::new("S", "100")).unwrap();
        model.add_parameter(Parameter::new("r", 0.1)).unwrap();
        model.add_flow(Flow::new("f", "S * r")).unwrap();
        model.stocks.get_mut("S").unwrap().inflows.push("f".to_string());

        let param_ranges = vec![
            ParameterRange::new("r".to_string(), 0.05, 0.15, 0.1),
        ];

        let mc_config = MonteCarloConfig {
            n_runs: 5,
            seed: Some(42),
            ..Default::default()
        };

        let simulator = MonteCarloSimulator::new(param_ranges, mc_config);
        let sim_config = SimulationConfig::default();

        let results = simulator.run(&model, &sim_config).unwrap();
        let csv = simulator.export_csv(&results, "S").unwrap();

        assert!(csv.contains("time,mean,std_dev"));
        assert!(csv.lines().count() > 1);
    }
}
