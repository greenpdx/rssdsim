/// Simulation engine - orchestrates model execution

use crate::model::Model;
use super::{SimulationState, SimulationConfig, SimulationResults, Integrator};
use super::integrator::{EulerIntegrator, RK4Integrator, HeunIntegrator, BackwardEulerIntegrator, RK45Integrator};
use super::IntegrationMethod;

pub struct SimulationEngine {
    model: Model,
    config: SimulationConfig,
    state: SimulationState,
}

impl SimulationEngine {
    pub fn new(model: Model, config: SimulationConfig) -> Result<Self, String> {
        let state = SimulationState::initialize_from_model(&model)?;

        Ok(Self {
            model,
            config,
            state,
        })
    }

    pub fn run(&mut self) -> Result<SimulationResults, String> {
        let mut results = SimulationResults::new();

        // Record initial state
        results.add_point(self.state.time, self.state.clone());

        let dt = self.model.time.dt;
        let stop_time = self.model.time.stop;

        // Create integrator
        let integrator: Box<dyn Integrator> = match self.config.integration_method {
            IntegrationMethod::Euler => Box::new(EulerIntegrator),
            IntegrationMethod::RK4 => Box::new(RK4Integrator),
            IntegrationMethod::RK45 => Box::new(RK45Integrator::default()),
            IntegrationMethod::Heun => Box::new(HeunIntegrator),
            IntegrationMethod::BackwardEuler => Box::new(BackwardEulerIntegrator::default()),
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
                // Check if we've crossed an output interval boundary
                let current_interval = (self.state.time / interval).floor();
                let prev_interval = ((self.state.time - dt) / interval).floor();
                current_interval > prev_interval
            } else {
                // Record every step
                true
            };

            if should_record {
                results.add_point(self.state.time, self.state.clone());
            }
        }

        Ok(results)
    }

    pub fn step(&mut self) -> Result<(), String> {
        let integrator: Box<dyn Integrator> = match self.config.integration_method {
            IntegrationMethod::Euler => Box::new(EulerIntegrator),
            IntegrationMethod::RK4 => Box::new(RK4Integrator),
            IntegrationMethod::RK45 => Box::new(RK45Integrator::default()),
            IntegrationMethod::Heun => Box::new(HeunIntegrator),
            IntegrationMethod::BackwardEuler => Box::new(BackwardEulerIntegrator::default()),
        };

        self.state = integrator.step(&self.model, &self.state, self.model.time.dt)?;
        Ok(())
    }

    pub fn current_state(&self) -> &SimulationState {
        &self.state
    }

    pub fn current_time(&self) -> f64 {
        self.state.time
    }

    pub fn set_parameter(&mut self, name: &str, value: f64) -> Result<(), String> {
        if let Some(param) = self.model.parameters.get_mut(name) {
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
    use crate::model::{Model, Stock, Flow, Parameter};

    #[test]
    fn test_simulation_engine_simple() {
        let mut model = Model::new("Test");
        model.time.start = 0.0;
        model.time.stop = 10.0;
        model.time.dt = 1.0;

        model.add_stock(Stock::new("Population", "100")).unwrap();
        model.add_parameter(Parameter::new("growth_rate", 0.1)).unwrap();
        model.add_flow(Flow::new("growth", "Population * growth_rate")).unwrap();

        model.stocks.get_mut("Population").unwrap().inflows.push("growth".to_string());

        let config = SimulationConfig::default();
        let mut engine = SimulationEngine::new(model, config).unwrap();

        let results = engine.run().unwrap();

        assert!(results.times.len() > 0);
        assert_eq!(results.times[0], 0.0);
        assert!(results.times.last().unwrap() <= &10.0);
    }
}
