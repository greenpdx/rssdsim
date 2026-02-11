/// rsedsim - Rust System Dynamics Simulator
///
/// A CLI-based system dynamics simulation framework with support for:
/// - Stock/flow models
/// - Agent-based modeling
/// - Hybrid simulations
/// - Multi-dimensional variables
/// - MCP and A2A protocol integration

mod protocol;
mod model;
mod simulation;
mod io;

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use colored::*;

#[derive(Parser)]
#[command(name = "rsedsim")]
#[command(about = "Rust System Dynamics Simulator", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a simulation
    Run {
        /// Model file (JSON or YAML)
        model: PathBuf,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Override parameters (format: "param1=value1,param2=value2")
        #[arg(short, long)]
        params: Option<String>,

        /// Integration method (euler or rk4)
        #[arg(long, default_value = "euler")]
        integrator: String,

        /// Override timestep (dt)
        #[arg(long)]
        dt: Option<f64>,
    },

    /// Validate a model file
    Validate {
        /// Model file to validate
        model: PathBuf,
    },

    /// Show version and info
    Info,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Run { model, output, params, integrator, dt }) => {
            run_simulation(model, output, params, integrator, dt)?;
        }
        Some(Commands::Validate { model }) => {
            validate_model(model)?;
        }
        Some(Commands::Info) => {
            show_info();
        }
        None => {
            show_info();
        }
    }

    Ok(())
}

fn run_simulation(
    model_path: PathBuf,
    output_path: Option<PathBuf>,
    params: Option<String>,
    integrator: String,
    dt_override: Option<f64>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "Loading model...".cyan());
    let mut model = io::load_model(&model_path)
        .map_err(|e| format!("Failed to load model: {}", e))?;

    println!("  Model: {}", model.metadata.name.green());
    println!("  Stocks: {}", model.stocks.len());
    println!("  Flows: {}", model.flows.len());
    println!("  Parameters: {}", model.parameters.len());

    // Override parameters if specified
    if let Some(param_str) = params {
        println!("\n{}", "Applying parameter overrides...".cyan());
        for pair in param_str.split(',') {
            let parts: Vec<&str> = pair.split('=').collect();
            if parts.len() == 2 {
                let name = parts[0].trim();
                let value: f64 = parts[1].trim().parse()
                    .map_err(|_| format!("Invalid parameter value: {}", parts[1]))?;

                if let Some(param) = model.parameters.get_mut(name) {
                    println!("  {} = {} (was {})", name, value, param.value);
                    param.value = value;
                } else {
                    eprintln!("  {} {}", "Warning:".yellow(), format!("Parameter '{}' not found", name));
                }
            }
        }
    }

    // Override timestep if specified
    if let Some(dt) = dt_override {
        println!("\n{}", "Overriding timestep...".cyan());
        println!("  dt = {} (was {})", dt, model.time.dt);
        model.time.dt = dt;
    }

    // Create simulation config
    let integration_method = match integrator.to_lowercase().as_str() {
        "euler" => simulation::IntegrationMethod::Euler,
        "rk4" => simulation::IntegrationMethod::RK4,
        _ => {
            eprintln!("{} Unknown integrator '{}', using Euler", "Warning:".yellow(), integrator);
            simulation::IntegrationMethod::Euler
        }
    };

    let config = simulation::SimulationConfig {
        integration_method,
        output_interval: None,
    };

    println!("\n{}", "Running simulation...".cyan());
    println!("  Time: {} to {} (dt={})", model.time.start, model.time.stop, model.time.dt);
    println!("  Integrator: {:?}", integration_method);

    let mut engine = simulation::SimulationEngine::new(model, config)
        .map_err(|e| format!("Failed to create engine: {}", e))?;

    let results = engine.run()
        .map_err(|e| format!("Simulation failed: {}", e))?;

    println!("  {} steps completed", results.times.len().to_string().green());

    // Write output
    let output_file = output_path.unwrap_or_else(|| PathBuf::from("results.csv"));
    println!("\n{}", "Writing results...".cyan());
    io::write_csv(&results, &output_file)
        .map_err(|e| format!("Failed to write results: {}", e))?;

    println!("  Output: {}", output_file.display().to_string().green());

    println!("\n{}", "✓ Simulation complete!".green().bold());

    Ok(())
}

fn validate_model(model_path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "Validating model...".cyan());

    let model = io::load_model(&model_path)
        .map_err(|e| format!("Failed to load model: {}", e))?;

    println!("  Model: {}", model.metadata.name.green());
    println!("\n{}", "Structure:".bold());
    println!("  Stocks: {}", model.stocks.len());
    println!("  Flows: {}", model.flows.len());
    println!("  Auxiliaries: {}", model.auxiliaries.len());
    println!("  Parameters: {}", model.parameters.len());

    // Basic validation
    let mut errors = Vec::new();

    // Check that all flows referenced by stocks exist
    for (stock_name, stock) in &model.stocks {
        for inflow in &stock.inflows {
            if !model.flows.contains_key(inflow) {
                errors.push(format!("Stock '{}' references non-existent inflow '{}'", stock_name, inflow));
            }
        }
        for outflow in &stock.outflows {
            if !model.flows.contains_key(outflow) {
                errors.push(format!("Stock '{}' references non-existent outflow '{}'", stock_name, outflow));
            }
        }
    }

    if errors.is_empty() {
        println!("\n{}", "✓ Model is valid!".green().bold());
    } else {
        println!("\n{}", "✗ Validation errors:".red().bold());
        for error in errors {
            println!("  {}", error.red());
        }
    }

    Ok(())
}

fn show_info() {
    println!("{}", "rsedsim - Rust System Dynamics Simulator v0.1.0".bold());
    println!("==============================================\n");

    println!("A CLI-based system dynamics simulation framework\n");

    println!("{}", "Features:".bold());
    println!("  ✓ Stock-flow models");
    println!("  ✓ Expression evaluation");
    println!("  ✓ Multiple integrators (Euler, RK4)");
    println!("  ✓ JSON/YAML model format");
    println!("  ✓ CSV output");
    println!("  ○ Agent-based modeling (planned)");
    println!("  ○ Hybrid models (planned)");
    println!("  ○ Multi-dimensional variables (planned)");

    println!("\n{}", "Protocol support:".bold());
    println!("  ○ MCP (Model Context Protocol) - stubs ready");
    println!("  ○ A2A (Agent-to-Agent) - stubs ready");

    println!("\n{}", "Usage:".bold());
    println!("  rsedsim run <model.yaml> -o results.csv");
    println!("  rsedsim validate <model.yaml>");
    println!("  rsedsim run <model.json> -p \"param1=10,param2=0.5\"");

    println!("\n{}", "Examples:".bold());
    println!("  See examples/ directory for sample models");

    println!("\nFor more information: https://github.com/yourusername/rsedsim");
}
