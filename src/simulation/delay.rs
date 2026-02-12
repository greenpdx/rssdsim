/// Delay function support for system dynamics models
///
/// This module provides infrastructure for:
/// - DELAY1: First-order exponential delay
/// - DELAY3: Third-order delay (smoother)
/// - SMOOTH: Alias for DELAY1
/// - DELAYP: Pipeline (pure time) delay

use std::collections::{HashMap, VecDeque};

/// Represents a single delay instance (for DELAY1/DELAY3/SMOOTH)
#[derive(Debug, Clone)]
pub struct ExponentialDelay {
    /// Current delayed value
    pub value: f64,
    /// Delay time constant
    pub delay_time: f64,
    /// Order of the delay (1 for DELAY1/SMOOTH, 3 for DELAY3)
    pub order: usize,
    /// For DELAY3, we need intermediate stages
    pub stages: Vec<f64>,
}

impl ExponentialDelay {
    pub fn new(initial_value: f64, delay_time: f64, order: usize) -> Self {
        let stages = if order > 1 {
            vec![initial_value; order]
        } else {
            vec![initial_value]
        };

        Self {
            value: initial_value,
            delay_time,
            order,
            stages,
        }
    }

    /// Update the delay using Euler integration
    /// For DELAY1: d(output)/dt = (input - output) / delay_time
    /// For DELAY3: Chain of 3 first-order delays
    pub fn update(&mut self, input: f64, dt: f64) {
        if self.order == 1 {
            // First-order delay
            let derivative = (input - self.value) / self.delay_time;
            self.value += derivative * dt;
            self.stages[0] = self.value;
        } else {
            // Higher-order delay (cascade of first-order delays)
            let stage_delay = self.delay_time / self.order as f64;

            // First stage gets input
            let deriv0 = (input - self.stages[0]) / stage_delay;
            let new_stage0 = self.stages[0] + deriv0 * dt;

            // Subsequent stages cascade
            let mut new_stages = vec![new_stage0];
            for i in 1..self.order {
                let deriv = (self.stages[i - 1] - self.stages[i]) / stage_delay;
                new_stages.push(self.stages[i] + deriv * dt);
            }

            self.stages = new_stages;
            self.value = self.stages[self.order - 1];
        }
    }

    /// Get current delayed value
    pub fn get_value(&self) -> f64 {
        self.value
    }
}

/// Represents a pipeline delay (fixed time delay with history buffer)
#[derive(Debug, Clone)]
pub struct PipelineDelay {
    /// History buffer storing (time, value) pairs
    history: VecDeque<(f64, f64)>,
    /// Delay time
    delay_time: f64,
    /// Initial value used for times before simulation start
    initial_value: f64,
}

impl PipelineDelay {
    pub fn new(initial_value: f64, delay_time: f64) -> Self {
        Self {
            history: VecDeque::new(),
            delay_time,
            initial_value,
        }
    }

    /// Record a new value at the current time
    pub fn push(&mut self, time: f64, value: f64) {
        self.history.push_back((time, value));

        // Remove old values that are too far in the past
        // Keep values for at least delay_time * 2 for safety
        let cutoff_time = time - self.delay_time * 2.0;
        while self.history.len() > 2 {
            if let Some(&(t, _)) = self.history.front() {
                if t < cutoff_time {
                    self.history.pop_front();
                } else {
                    break;
                }
            }
        }
    }

    /// Get the delayed value at the current time
    /// Uses linear interpolation between history points
    pub fn get_delayed_value(&self, current_time: f64) -> f64 {
        let target_time = current_time - self.delay_time;

        // If target time is before any history, return initial value
        if self.history.is_empty() {
            return self.initial_value;
        }

        if let Some(&(first_time, _)) = self.history.front() {
            if target_time <= first_time {
                return self.initial_value;
            }
        }

        // Find the two points to interpolate between
        for i in 0..self.history.len() {
            let (t, v) = self.history[i];

            if t >= target_time {
                if i == 0 {
                    // Target is before first recorded point, use initial
                    return self.initial_value;
                }

                // Interpolate between i-1 and i
                let (t_prev, v_prev) = self.history[i - 1];
                let alpha = (target_time - t_prev) / (t - t_prev);
                return v_prev + alpha * (v - v_prev);
            }
        }

        // Target is after all recorded points, use last value
        if let Some(&(_, v)) = self.history.back() {
            v
        } else {
            self.initial_value
        }
    }
}

/// Manager for all delays in a simulation
#[derive(Debug, Clone)]
pub struct DelayManager {
    /// Exponential delays (DELAY1, DELAY3, SMOOTH) indexed by unique key
    pub exponential_delays: HashMap<String, ExponentialDelay>,
    /// Pipeline delays (DELAYP) indexed by unique key
    pub pipeline_delays: HashMap<String, PipelineDelay>,
}

impl DelayManager {
    pub fn new() -> Self {
        Self {
            exponential_delays: HashMap::new(),
            pipeline_delays: HashMap::new(),
        }
    }

    /// Get or create an exponential delay
    pub fn get_or_create_exponential(
        &mut self,
        key: &str,
        initial_value: f64,
        delay_time: f64,
        order: usize,
    ) -> &mut ExponentialDelay {
        self.exponential_delays
            .entry(key.to_string())
            .or_insert_with(|| ExponentialDelay::new(initial_value, delay_time, order))
    }

    /// Get or create a pipeline delay
    pub fn get_or_create_pipeline(
        &mut self,
        key: &str,
        initial_value: f64,
        delay_time: f64,
    ) -> &mut PipelineDelay {
        self.pipeline_delays
            .entry(key.to_string())
            .or_insert_with(|| PipelineDelay::new(initial_value, delay_time))
    }

    /// Update all exponential delays
    pub fn update_all_exponential(&mut self, inputs: &HashMap<String, f64>, dt: f64) {
        for (key, delay) in &mut self.exponential_delays {
            if let Some(&input) = inputs.get(key) {
                delay.update(input, dt);
            }
        }
    }
}

impl Default for DelayManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delay1() {
        let mut delay = ExponentialDelay::new(0.0, 10.0, 1);

        // Step input of 1.0
        // After t=10 (one time constant), should reach ~63% of final value
        // After t=50 (5 time constants), should be >99% of final value
        for _ in 0..500 {
            delay.update(1.0, 0.1);
        }

        // After many time constants (t=50), should be very close to 1.0
        assert!((delay.get_value() - 1.0).abs() < 0.1, "Got: {}", delay.get_value());
    }

    #[test]
    fn test_pipeline_delay() {
        let mut delay = PipelineDelay::new(0.0, 5.0);

        // Record some history
        delay.push(0.0, 0.0);
        delay.push(1.0, 1.0);
        delay.push(2.0, 2.0);
        delay.push(10.0, 10.0);

        // At time 10, delayed value (5 time units ago) should be 5.0
        let delayed = delay.get_delayed_value(10.0);
        assert!((delayed - 5.0).abs() < 0.1);
    }
}
