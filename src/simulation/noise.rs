/// Noise generators for stochastic modeling
///
/// Implements various noise types:
/// - White noise: Uncorrelated Gaussian noise
/// - Pink noise: 1/f noise with power spectral density inversely proportional to frequency

use rand::prelude::*;
use rand_distr::{Distribution, Normal};

/// White noise generator
/// Generates uncorrelated Gaussian random values
#[derive(Debug, Clone)]
pub struct WhiteNoiseGenerator {
    /// Mean of the distribution
    mean: f64,
    /// Standard deviation
    std_dev: f64,
    /// Sample rate (samples per time unit)
    sample_rate: f64,
}

impl WhiteNoiseGenerator {
    /// Create a new white noise generator
    pub fn new(mean: f64, std_dev: f64, sample_rate: f64) -> Self {
        Self {
            mean,
            std_dev,
            sample_rate,
        }
    }

    /// Generate next sample
    pub fn sample(&self, rng: &mut StdRng) -> f64 {
        let normal = Normal::new(self.mean, self.std_dev)
            .unwrap_or_else(|_| Normal::new(0.0, 1.0).unwrap());
        normal.sample(rng)
    }

    /// Generate samples scaled by time step
    pub fn sample_dt(&self, rng: &mut StdRng, dt: f64) -> f64 {
        // Scale by sqrt(dt) to maintain correct variance
        let scale = (dt * self.sample_rate).sqrt();
        let normal = Normal::new(0.0, self.std_dev * scale)
            .unwrap_or_else(|_| Normal::new(0.0, 1.0).unwrap());
        self.mean + normal.sample(rng)
    }
}

/// Pink noise generator using Voss-McCartney algorithm
/// Generates 1/f noise (power spectral density inversely proportional to frequency)
#[derive(Debug, Clone)]
pub struct PinkNoiseGenerator {
    /// Number of octaves (more = better quality, typically 16)
    num_octaves: usize,
    /// White noise values for each octave
    white_values: Vec<f64>,
    /// Counter for determining which octaves to update
    counter: usize,
    /// Amplitude scaling
    amplitude: f64,
    /// DC offset
    offset: f64,
}

impl PinkNoiseGenerator {
    /// Create a new pink noise generator
    pub fn new(amplitude: f64, offset: f64, num_octaves: usize) -> Self {
        Self {
            num_octaves,
            white_values: vec![0.0; num_octaves],
            counter: 0,
            amplitude,
            offset,
        }
    }

    /// Create with default parameters
    pub fn default_params() -> Self {
        Self::new(1.0, 0.0, 16)
    }

    /// Generate next pink noise sample
    pub fn sample(&mut self, rng: &mut StdRng) -> f64 {
        let mut sum = 0.0;

        // Update white noise values based on counter
        // This creates the 1/f characteristic
        for i in 0..self.num_octaves {
            // Update this octave if counter is divisible by 2^i
            if self.counter % (1 << i) == 0 {
                self.white_values[i] = rng.sample::<f64, _>(rand::distributions::Standard) * 2.0 - 1.0;
            }
            sum += self.white_values[i];
        }

        self.counter = self.counter.wrapping_add(1);

        // Normalize and scale
        self.offset + self.amplitude * (sum / (self.num_octaves as f64))
    }

    /// Reset the generator
    pub fn reset(&mut self) {
        self.white_values.fill(0.0);
        self.counter = 0;
    }
}

/// Improved pink noise generator using Paul Kellet's method
/// Better spectral characteristics than Voss-McCartney
#[derive(Debug, Clone)]
pub struct PinkNoiseKellet {
    b0: f64,
    b1: f64,
    b2: f64,
    b3: f64,
    b4: f64,
    b5: f64,
    b6: f64,
    amplitude: f64,
    offset: f64,
}

impl PinkNoiseKellet {
    pub fn new(amplitude: f64, offset: f64) -> Self {
        Self {
            b0: 0.0,
            b1: 0.0,
            b2: 0.0,
            b3: 0.0,
            b4: 0.0,
            b5: 0.0,
            b6: 0.0,
            amplitude,
            offset,
        }
    }

    /// Generate next sample using Paul Kellet's algorithm
    pub fn sample(&mut self, rng: &mut StdRng) -> f64 {
        let white = rng.sample::<f64, _>(rand::distributions::Standard) * 2.0 - 1.0;

        self.b0 = 0.99886 * self.b0 + white * 0.0555179;
        self.b1 = 0.99332 * self.b1 + white * 0.0750759;
        self.b2 = 0.96900 * self.b2 + white * 0.1538520;
        self.b3 = 0.86650 * self.b3 + white * 0.3104856;
        self.b4 = 0.55000 * self.b4 + white * 0.5329522;
        self.b5 = -0.7616 * self.b5 - white * 0.0168980;

        let pink = self.b0 + self.b1 + self.b2 + self.b3 + self.b4 + self.b5 + self.b6 + white * 0.5362;
        self.b6 = white * 0.115926;

        self.offset + self.amplitude * (pink / 7.0)
    }

    /// Reset the generator
    pub fn reset(&mut self) {
        self.b0 = 0.0;
        self.b1 = 0.0;
        self.b2 = 0.0;
        self.b3 = 0.0;
        self.b4 = 0.0;
        self.b5 = 0.0;
        self.b6 = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_white_noise() {
        let mut rng = StdRng::seed_from_u64(42);
        let generator = WhiteNoiseGenerator::new(0.0, 1.0, 1.0);

        let samples: Vec<f64> = (0..1000).map(|_| generator.sample(&mut rng)).collect();

        // Check mean is close to 0
        let mean: f64 = samples.iter().sum::<f64>() / samples.len() as f64;
        assert!((mean - 0.0).abs() < 0.1);

        // Check std dev is close to 1
        let variance: f64 = samples.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / samples.len() as f64;
        let std_dev = variance.sqrt();
        assert!((std_dev - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_pink_noise_voss() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut generator = PinkNoiseGenerator::new(1.0, 0.0, 16);

        let samples: Vec<f64> = (0..1000).map(|_| generator.sample(&mut rng)).collect();

        // Pink noise should be bounded roughly in [-1, 1] range
        for &sample in &samples {
            assert!(sample.abs() < 2.0);
        }

        // Check that samples vary (not all the same)
        let variance: f64 = samples.iter()
            .map(|x| x.powi(2))
            .sum::<f64>() / samples.len() as f64;
        assert!(variance > 0.01);
    }

    #[test]
    fn test_pink_noise_kellet() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut generator = PinkNoiseKellet::new(1.0, 0.0);

        let samples: Vec<f64> = (0..1000).map(|_| generator.sample(&mut rng)).collect();

        // Pink noise should be bounded
        for &sample in &samples {
            assert!(sample.abs() < 3.0);
        }

        // Check variance
        let variance: f64 = samples.iter()
            .map(|x| x.powi(2))
            .sum::<f64>() / samples.len() as f64;
        assert!(variance > 0.01);
    }

    #[test]
    fn test_pink_noise_reset() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut generator1 = PinkNoiseGenerator::new(1.0, 0.0, 16);
        let mut generator2 = PinkNoiseGenerator::new(1.0, 0.0, 16);

        // Generate some samples
        for _ in 0..100 {
            generator1.sample(&mut rng);
        }

        // Reset and compare with fresh generator
        generator1.reset();

        let mut rng1 = StdRng::seed_from_u64(42);
        let mut rng2 = StdRng::seed_from_u64(42);

        for _ in 0..10 {
            let s1 = generator1.sample(&mut rng1);
            let s2 = generator2.sample(&mut rng2);
            assert_eq!(s1, s2);
        }
    }

    #[test]
    fn test_white_noise_dt_scaling() {
        let mut rng = StdRng::seed_from_u64(42);
        let generator = WhiteNoiseGenerator::new(0.0, 1.0, 1.0);

        let dt = 0.1;
        let samples: Vec<f64> = (0..1000).map(|_| generator.sample_dt(&mut rng, dt)).collect();

        // With smaller dt, variance should be scaled
        let variance: f64 = samples.iter()
            .map(|x| x.powi(2))
            .sum::<f64>() / samples.len() as f64;

        // Variance should be approximately dt
        assert!((variance - dt).abs() < 0.05);
    }
}
