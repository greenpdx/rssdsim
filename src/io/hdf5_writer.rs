/// HDF5 output writer for large simulation datasets
///
/// HDF5 (Hierarchical Data Format) is a versatile data model and file format
/// for storing and managing large and complex data. It's ideal for:
/// - Hierarchical data structures
/// - Very large datasets (> 10GB)
/// - High-performance I/O
/// - Chunking and compression

#[cfg(feature = "with-hdf5")]
use hdf5::File;
#[cfg(feature = "with-hdf5")]
use std::path::Path;
#[cfg(feature = "with-hdf5")]
use crate::simulation::SimulationResults;

#[cfg(feature = "with-hdf5")]
pub struct HDF5Writer;

#[cfg(feature = "with-hdf5")]
impl HDF5Writer {
    /// Write simulation results to HDF5 file
    pub fn write<P: AsRef<Path>>(
        results: &SimulationResults,
        path: P,
    ) -> Result<(), String> {
        let file = File::create(path)
            .map_err(|e| format!("Failed to create HDF5 file: {}", e))?;

        // Extract time series
        let time_values: Vec<f64> = results.states.iter().map(|s| s.time).collect();

        // Create time dataset
        file.new_dataset::<f64>()
            .create("time", time_values.len())
            .map_err(|e| format!("Failed to create time dataset: {}", e))?
            .write(&time_values)
            .map_err(|e| format!("Failed to write time data: {}", e))?;

        // Create stocks group
        let stocks_group = file
            .create_group("stocks")
            .map_err(|e| format!("Failed to create stocks group: {}", e))?;

        if !results.states.is_empty() {
            // Write stock variables
            for stock_name in results.states[0].stocks.keys() {
                let values: Vec<f64> = results.states
                    .iter()
                    .map(|s| *s.stocks.get(stock_name).unwrap_or(&0.0))
                    .collect();

                stocks_group
                    .new_dataset::<f64>()
                    .create(stock_name, values.len())
                    .map_err(|e| format!("Failed to create dataset for '{}': {}", stock_name, e))?
                    .write(&values)
                    .map_err(|e| format!("Failed to write values for '{}': {}", stock_name, e))?;
            }

            // Create flows group
            let flows_group = file
                .create_group("flows")
                .map_err(|e| format!("Failed to create flows group: {}", e))?;

            // Write flow variables
            for flow_name in results.states[0].flows.keys() {
                let values: Vec<f64> = results.states
                    .iter()
                    .map(|s| *s.flows.get(flow_name).unwrap_or(&0.0))
                    .collect();

                flows_group
                    .new_dataset::<f64>()
                    .create(flow_name, values.len())
                    .map_err(|e| format!("Failed to create dataset for '{}': {}", flow_name, e))?
                    .write(&values)
                    .map_err(|e| format!("Failed to write values for '{}': {}", flow_name, e))?;
            }

            // Create auxiliaries group
            let aux_group = file
                .create_group("auxiliaries")
                .map_err(|e| format!("Failed to create auxiliaries group: {}", e))?;

            // Write auxiliary variables
            for aux_name in results.states[0].auxiliaries.keys() {
                let values: Vec<f64> = results.states
                    .iter()
                    .map(|s| *s.auxiliaries.get(aux_name).unwrap_or(&0.0))
                    .collect();

                aux_group
                    .new_dataset::<f64>()
                    .create(aux_name, values.len())
                    .map_err(|e| format!("Failed to create dataset for '{}': {}", aux_name, e))?
                    .write(&values)
                    .map_err(|e| format!("Failed to write values for '{}': {}", aux_name, e))?;
            }
        }

        // Add metadata attributes
        file.new_attr::<hdf5::types::VarLenUnicode>()
            .create("title")
            .map_err(|e| format!("Failed to create title attribute: {}", e))?
            .write_scalar(&hdf5::types::VarLenUnicode::from_str("System Dynamics Simulation Results"))
            .map_err(|e| format!("Failed to write title: {}", e))?;

        file.new_attr::<hdf5::types::VarLenUnicode>()
            .create("creator")
            .map_err(|e| format!("Failed to create creator attribute: {}", e))?
            .write_scalar(&hdf5::types::VarLenUnicode::from_str("rssdsim"))
            .map_err(|e| format!("Failed to write creator: {}", e))?;

        Ok(())
    }

    /// Write simulation results with compression
    pub fn write_compressed<P: AsRef<Path>>(
        results: &SimulationResults,
        path: P,
        compression_level: u8,
    ) -> Result<(), String> {
        let file = File::create(path)
            .map_err(|e| format!("Failed to create HDF5 file: {}", e))?;

        let time_values: Vec<f64> = results.states.iter().map(|s| s.time).collect();

        // Create time dataset with compression
        file.new_dataset::<f64>()
            .gzip(compression_level)
            .create("time", time_values.len())
            .map_err(|e| format!("Failed to create compressed time dataset: {}", e))?
            .write(&time_values)
            .map_err(|e| format!("Failed to write time data: {}", e))?;

        let stocks_group = file
            .create_group("stocks")
            .map_err(|e| format!("Failed to create stocks group: {}", e))?;

        if !results.states.is_empty() {
            // Write compressed stock variables
            for stock_name in results.states[0].stocks.keys() {
                let values: Vec<f64> = results.states
                    .iter()
                    .map(|s| *s.stocks.get(stock_name).unwrap_or(&0.0))
                    .collect();

                stocks_group
                    .new_dataset::<f64>()
                    .gzip(compression_level)
                    .create(stock_name, values.len())
                    .map_err(|e| format!("Failed to create dataset for '{}': {}", stock_name, e))?
                    .write(&values)
                    .map_err(|e| format!("Failed to write values for '{}': {}", stock_name, e))?;
            }

            // Similar for flows and auxiliaries...
            let flows_group = file
                .create_group("flows")
                .map_err(|e| format!("Failed to create flows group: {}", e))?;

            for flow_name in results.states[0].flows.keys() {
                let values: Vec<f64> = results.states
                    .iter()
                    .map(|s| *s.flows.get(flow_name).unwrap_or(&0.0))
                    .collect();

                flows_group
                    .new_dataset::<f64>()
                    .gzip(compression_level)
                    .create(flow_name, values.len())
                    .map_err(|e| format!("Failed to create dataset for '{}': {}", flow_name, e))?
                    .write(&values)
                    .map_err(|e| format!("Failed to write values for '{}': {}", flow_name, e))?;
            }
        }

        Ok(())
    }
}

// Stub implementation when feature is not enabled
#[cfg(not(feature = "with-hdf5"))]
pub struct HDF5Writer;

#[cfg(not(feature = "with-hdf5"))]
impl HDF5Writer {
    pub fn write<P>(_results: &crate::simulation::SimulationResults, _path: P) -> Result<(), String>
    where
        P: AsRef<std::path::Path>,
    {
        Err("HDF5 support not enabled. Compile with --features with-hdf5".to_string())
    }

    pub fn write_compressed<P>(
        _results: &crate::simulation::SimulationResults,
        _path: P,
        _compression_level: u8,
    ) -> Result<(), String>
    where
        P: AsRef<std::path::Path>,
    {
        Err("HDF5 support not enabled. Compile with --features with-hdf5".to_string())
    }
}

#[cfg(all(test, feature = "with-hdf5"))]
mod tests {
    use super::*;
    use crate::model::{Model, Stock, Flow, Parameter};
    use crate::simulation::{SimulationEngine, SimulationConfig};

    #[test]
    fn test_hdf5_write() {
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

        let temp_file = "/tmp/test_output.h5";
        HDF5Writer::write(&results, temp_file).unwrap();

        // Verify file was created
        assert!(std::path::Path::new(temp_file).exists());

        // Clean up
        let _ = std::fs::remove_file(temp_file);
    }

    #[test]
    fn test_hdf5_write_compressed() {
        let mut model = Model::new("Test");
        model.time.start = 0.0;
        model.time.stop = 100.0;
        model.time.dt = 0.1;

        model.add_stock(Stock::new("X", "100")).unwrap();
        model.add_parameter(Parameter::new("k", 0.1)).unwrap();
        model.add_flow(Flow::new("f", "X * k")).unwrap();
        model.stocks.get_mut("X").unwrap().inflows.push("f".to_string());

        let config = SimulationConfig::default();
        let mut engine = SimulationEngine::new(model, config).unwrap();
        let results = engine.run().unwrap();

        let temp_file = "/tmp/test_output_compressed.h5";
        HDF5Writer::write_compressed(&results, temp_file, 6).unwrap();

        assert!(std::path::Path::new(temp_file).exists());

        let _ = std::fs::remove_file(temp_file);
    }
}
