/// NetCDF output writer for large simulation datasets
///
/// NetCDF (Network Common Data Form) is a self-describing, machine-independent
/// data format for array-oriented scientific data. It's ideal for:
/// - Multi-dimensional time series
/// - Large datasets (> 1GB)
/// - Metadata-rich outputs
/// - Cross-platform compatibility

#[cfg(feature = "with-netcdf")]
use netcdf::{create, types};
#[cfg(feature = "with-netcdf")]
use std::path::Path;
#[cfg(feature = "with-netcdf")]
use crate::simulation::SimulationResults;

#[cfg(feature = "with-netcdf")]
pub struct NetCDFWriter;

#[cfg(feature = "with-netcdf")]
impl NetCDFWriter {
    /// Write simulation results to NetCDF file
    pub fn write<P: AsRef<Path>>(
        results: &SimulationResults,
        path: P,
    ) -> Result<(), String> {
        let mut file = create(path)
            .map_err(|e| format!("Failed to create NetCDF file: {}", e))?;

        // Extract time series
        let n_steps = results.states.len();
        let time_values: Vec<f64> = results.states.iter().map(|s| s.time).collect();

        // Define dimensions
        file.add_dimension("time", n_steps)
            .map_err(|e| format!("Failed to add time dimension: {}", e))?;

        // Add time variable
        let mut time_var = file
            .add_variable::<f64>("time", &["time"])
            .map_err(|e| format!("Failed to add time variable: {}", e))?;

        time_var
            .add_attribute("units", "time units")
            .map_err(|e| format!("Failed to add time units: {}", e))?;
        time_var
            .add_attribute("long_name", "simulation time")
            .map_err(|e| format!("Failed to add time long_name: {}", e))?;

        time_var
            .put_values(&time_values, None, None)
            .map_err(|e| format!("Failed to write time values: {}", e))?;

        // Add stock variables
        if !results.states.is_empty() {
            for stock_name in results.states[0].stocks.keys() {
                let values: Vec<f64> = results.states
                    .iter()
                    .map(|s| *s.stocks.get(stock_name).unwrap_or(&0.0))
                    .collect();

                let mut var = file
                    .add_variable::<f64>(stock_name, &["time"])
                    .map_err(|e| format!("Failed to add variable '{}': {}", stock_name, e))?;

                var.add_attribute("long_name", stock_name.clone())
                    .map_err(|e| format!("Failed to add attribute: {}", e))?;
                var.add_attribute("variable_type", "stock")
                    .map_err(|e| format!("Failed to add attribute: {}", e))?;

                var.put_values(&values, None, None)
                    .map_err(|e| format!("Failed to write values for '{}': {}", stock_name, e))?;
            }

            // Add flow variables
            for flow_name in results.states[0].flows.keys() {
                let values: Vec<f64> = results.states
                    .iter()
                    .map(|s| *s.flows.get(flow_name).unwrap_or(&0.0))
                    .collect();

                let mut var = file
                    .add_variable::<f64>(flow_name, &["time"])
                    .map_err(|e| format!("Failed to add variable '{}': {}", flow_name, e))?;

                var.add_attribute("long_name", flow_name.clone())
                    .map_err(|e| format!("Failed to add attribute: {}", e))?;
                var.add_attribute("variable_type", "flow")
                    .map_err(|e| format!("Failed to add attribute: {}", e))?;

                var.put_values(&values, None, None)
                    .map_err(|e| format!("Failed to write values for '{}': {}", flow_name, e))?;
            }

            // Add auxiliary variables
            for aux_name in results.states[0].auxiliaries.keys() {
                let values: Vec<f64> = results.states
                    .iter()
                    .map(|s| *s.auxiliaries.get(aux_name).unwrap_or(&0.0))
                    .collect();

                let mut var = file
                    .add_variable::<f64>(aux_name, &["time"])
                    .map_err(|e| format!("Failed to add variable '{}': {}", aux_name, e))?;

                var.add_attribute("long_name", aux_name.clone())
                    .map_err(|e| format!("Failed to add attribute: {}", e))?;
                var.add_attribute("variable_type", "auxiliary")
                    .map_err(|e| format!("Failed to add attribute: {}", e))?;

                var.put_values(&values, None, None)
                    .map_err(|e| format!("Failed to write values for '{}': {}", aux_name, e))?;
            }
        }

        // Add global metadata
        file.add_attribute("title", "System Dynamics Simulation Results")
            .map_err(|e| format!("Failed to add global attribute: {}", e))?;
        file.add_attribute("creator", "rssdsim")
            .map_err(|e| format!("Failed to add global attribute: {}", e))?;

        Ok(())
    }
}

// Stub implementation when feature is not enabled
#[cfg(not(feature = "with-netcdf"))]
pub struct NetCDFWriter;

#[cfg(not(feature = "with-netcdf"))]
impl NetCDFWriter {
    pub fn write<P>(_results: &crate::simulation::SimulationResults, _path: P) -> Result<(), String>
    where
        P: AsRef<std::path::Path>,
    {
        Err("NetCDF support not enabled. Compile with --features with-netcdf".to_string())
    }
}

#[cfg(all(test, feature = "with-netcdf"))]
mod tests {
    use super::*;
    use crate::model::{Model, Stock, Flow, Parameter};
    use crate::simulation::{SimulationEngine, SimulationConfig};

    #[test]
    fn test_netcdf_write() {
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

        let temp_file = "/tmp/test_output.nc";
        NetCDFWriter::write(&results, temp_file).unwrap();

        // Verify file was created
        assert!(std::path::Path::new(temp_file).exists());

        // Clean up
        let _ = std::fs::remove_file(temp_file);
    }
}
