/// I/O module - model and results serialization

use std::fs;
use std::path::Path;
use crate::model::Model;
use crate::simulation::SimulationResults;

pub mod parser;
pub mod writer;
pub mod xmile;
pub mod insightmaker;
pub mod netcdf_writer;
pub mod hdf5_writer;

pub use parser::ModelParser;
pub use writer::ResultWriter;
pub use netcdf_writer::NetCDFWriter;
pub use hdf5_writer::HDF5Writer;

/// Load model from file (auto-detect format)
pub fn load_model<P: AsRef<Path>>(path: P) -> Result<Model, String> {
    let path = path.as_ref();
    let contents = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let extension = path.extension()
        .and_then(|s| s.to_str())
        .ok_or("No file extension")?;

    match extension {
        "json" => {
            // Try InsightMaker format first, fall back to standard JSON
            if let Ok(model) = insightmaker::parse_insightmaker(&contents) {
                return Ok(model);
            }

            let json_model: parser::JsonModel = serde_json::from_str(&contents)
                .map_err(|e| format!("Failed to parse JSON: {}", e))?;
            parser::JsonModel::to_model(json_model)
        }
        "yaml" | "yml" => {
            parser::parse_yaml(&contents)
        }
        "xmile" | "stmx" | "itmx" | "xml" => {
            xmile::parse_xmile(&contents)
        }
        _ => Err(format!("Unsupported file format: {}", extension)),
    }
}

/// Write results to CSV file
pub fn write_csv<P: AsRef<Path>>(results: &SimulationResults, path: P) -> Result<(), String> {
    writer::CsvWriter::write_file(results, path)
}

/// Write results to NetCDF file
#[cfg(feature = "with-netcdf")]
pub fn write_netcdf<P: AsRef<Path>>(results: &SimulationResults, path: P) -> Result<(), String> {
    netcdf_writer::NetCDFWriter::write(results, path)
}

/// Write results to HDF5 file
#[cfg(feature = "with-hdf5")]
pub fn write_hdf5<P: AsRef<Path>>(results: &SimulationResults, path: P) -> Result<(), String> {
    hdf5_writer::HDF5Writer::write(results, path)
}

/// Write results to HDF5 file with compression
#[cfg(feature = "with-hdf5")]
pub fn write_hdf5_compressed<P: AsRef<Path>>(
    results: &SimulationResults,
    path: P,
    compression_level: u8,
) -> Result<(), String> {
    hdf5_writer::HDF5Writer::write_compressed(results, path, compression_level)
}
