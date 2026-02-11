/// Result writers for various formats

use std::fs::File;
use std::io::Write;
use std::path::Path;
use crate::simulation::SimulationResults;

pub trait ResultWriter {
    fn write_file<P: AsRef<Path>>(results: &SimulationResults, path: P) -> Result<(), String>;
}

pub struct CsvWriter;

impl CsvWriter {
    pub fn write_file<P: AsRef<Path>>(results: &SimulationResults, path: P) -> Result<(), String> {
        let mut file = File::create(path)
            .map_err(|e| format!("Failed to create file: {}", e))?;

        if results.states.is_empty() {
            return Err("No results to write".to_string());
        }

        // Get all variable names from first state
        let first_state = &results.states[0];
        let mut var_names: Vec<String> = Vec::new();

        // Collect stocks
        let mut stock_names: Vec<_> = first_state.stocks.keys().cloned().collect();
        stock_names.sort();
        var_names.extend(stock_names);

        // Collect flows
        let mut flow_names: Vec<_> = first_state.flows.keys().cloned().collect();
        flow_names.sort();
        var_names.extend(flow_names);

        // Collect auxiliaries
        let mut aux_names: Vec<_> = first_state.auxiliaries.keys().cloned().collect();
        aux_names.sort();
        var_names.extend(aux_names);

        // Write header
        write!(file, "Time")
            .map_err(|e| format!("Write error: {}", e))?;
        for var_name in &var_names {
            write!(file, ",{}", var_name)
                .map_err(|e| format!("Write error: {}", e))?;
        }
        writeln!(file)
            .map_err(|e| format!("Write error: {}", e))?;

        // Write data rows
        for (i, state) in results.states.iter().enumerate() {
            write!(file, "{}", results.times[i])
                .map_err(|e| format!("Write error: {}", e))?;

            for var_name in &var_names {
                let value = state.stocks.get(var_name)
                    .or_else(|| state.flows.get(var_name))
                    .or_else(|| state.auxiliaries.get(var_name))
                    .unwrap_or(&0.0);

                write!(file, ",{}", value)
                    .map_err(|e| format!("Write error: {}", e))?;
            }
            writeln!(file)
                .map_err(|e| format!("Write error: {}", e))?;
        }

        Ok(())
    }
}

impl ResultWriter for CsvWriter {
    fn write_file<P: AsRef<Path>>(results: &SimulationResults, path: P) -> Result<(), String> {
        Self::write_file(results, path)
    }
}

pub struct JsonWriter;

impl JsonWriter {
    pub fn write_file<P: AsRef<Path>>(results: &SimulationResults, path: P) -> Result<(), String> {
        // Simplified JSON output
        let mut file = File::create(path)
            .map_err(|e| format!("Failed to create file: {}", e))?;

        writeln!(file, "{{")
            .map_err(|e| format!("Write error: {}", e))?;
        writeln!(file, "  \"times\": {:?},", results.times)
            .map_err(|e| format!("Write error: {}", e))?;
        writeln!(file, "  \"num_points\": {}", results.times.len())
            .map_err(|e| format!("Write error: {}", e))?;
        writeln!(file, "}}")
            .map_err(|e| format!("Write error: {}", e))?;

        Ok(())
    }
}

impl ResultWriter for JsonWriter {
    fn write_file<P: AsRef<Path>>(results: &SimulationResults, path: P) -> Result<(), String> {
        Self::write_file(results, path)
    }
}
