/// Stochastic elements for system dynamics models
///
/// Provides random number generation for:
/// - RANDOM: Uniform random [0, 1)
/// - UNIFORM: Uniform random [min, max]
/// - NORMAL: Normal distribution (mean, std_dev)
/// - LOGNORMAL: Log-normal distribution
/// - POISSON: Poisson distribution

use rand::prelude::*;
use rand_distr::{Distribution, Normal, Poisson, LogNormal};

/// Manager for stochastic elements in simulation
#[derive(Debug, Clone)]
pub struct StochasticManager {
    /// Random number generator
    rng: StdRng,
    /// Seed for reproducibility
    seed: Option<u64>,
}

impl StochasticManager {
    pub fn new() -> Self {
        Self {
            rng: StdRng::from_entropy(),
            seed: None,
        }
    }

    pub fn with_seed(seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
            seed: Some(seed),
        }
    }

    /// Generate uniform random [0, 1)
    pub fn random(&mut self) -> f64 {
        self.rng.sample(rand::distributions::Standard)
    }

    /// Generate uniform random [min, max]
    pub fn uniform(&mut self, min: f64, max: f64) -> f64 {
        min + (max - min) * self.random()
    }

    /// Generate normal random with mean and standard deviation
    pub fn normal(&mut self, mean: f64, std_dev: f64) -> Result<f64, String> {
        let normal = Normal::new(mean, std_dev)
            .map_err(|e| format!("Invalid normal distribution parameters: {}", e))?;
        Ok(normal.sample(&mut self.rng))
    }

    /// Generate log-normal random
    pub fn lognormal(&mut self, mean: f64, std_dev: f64) -> Result<f64, String> {
        let lognormal = LogNormal::new(mean, std_dev)
            .map_err(|e| format!("Invalid log-normal distribution parameters: {}", e))?;
        Ok(lognormal.sample(&mut self.rng))
    }

    /// Generate Poisson random variable
    pub fn poisson(&mut self, lambda: f64) -> Result<f64, String> {
        if lambda <= 0.0 {
            return Err("Poisson lambda must be positive".to_string());
        }
        let poisson = Poisson::new(lambda)
            .map_err(|e| format!("Invalid Poisson parameter: {}", e))?;
        Ok(poisson.sample(&mut self.rng) as f64)
    }

    /// Reset RNG with a new seed
    pub fn reseed(&mut self, seed: u64) {
        self.rng = StdRng::seed_from_u64(seed);
        self.seed = Some(seed);
    }
}

impl Default for StochasticManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random() {
        let mut mgr = StochasticManager::with_seed(42);
        let val = mgr.random();
        assert!(val >= 0.0 && val < 1.0);
    }

    #[test]
    fn test_uniform() {
        let mut mgr = StochasticManager::with_seed(42);
        let val = mgr.uniform(10.0, 20.0);
        assert!(val >= 10.0 && val <= 20.0);
    }

    #[test]
    fn test_normal() {
        let mut mgr = StochasticManager::with_seed(42);
        let val = mgr.normal(0.0, 1.0).unwrap();
        // Just check it returns something reasonable
        assert!(val.is_finite());
    }

    #[test]
    fn test_poisson() {
        let mut mgr = StochasticManager::with_seed(42);
        let val = mgr.poisson(5.0).unwrap();
        assert!(val >= 0.0);
    }

    #[test]
    fn test_reproducibility() {
        let mut mgr1 = StochasticManager::with_seed(123);
        let mut mgr2 = StochasticManager::with_seed(123);

        // Same seed should produce same sequence
        for _ in 0..10 {
            assert_eq!(mgr1.random(), mgr2.random());
        }
    }
}
