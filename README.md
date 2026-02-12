# rssdsim - Rust System Dynamics Simulator

A high-performance CLI-based system dynamics simulation framework with comprehensive file format support and extensible architecture.

## Features

### Core Simulation Capabilities
- **Stock-Flow Modeling**: Traditional system dynamics with stocks, flows, auxiliaries, and parameters
- **Expression Evaluation**: Full equation parsing with 47+ built-in functions
- **Multiple Integration Methods**: Euler, Heun (RK2), RK4, and Backward Euler (implicit)
- **Model Validation**: Structural validation and dependency checking
- **Parameter Overrides**: Runtime parameter modification
- **Monte Carlo Simulation**: Uncertainty quantification with comprehensive statistics

### Built-in Functions
Complete function library including:
- **Math**: MIN, MAX, ABS, SQRT, EXP, LN, LOG, LOG10, POW, MODULO
- **Trigonometric**: SIN, COS, TAN, ASIN, ACOS, ATAN
- **Rounding**: FLOOR, CEIL, ROUND
- **System Dynamics**: PULSE, STEP, RAMP, TIME
- **Delay Functions**: DELAY1, DELAY3, DELAYP, SMOOTH (exponential and pipeline delays)
- **Lookup Tables**: WITH_LOOKUP (inline graphical functions with linear interpolation)
- **Stochastic Elements**: RANDOM, UNIFORM, NORMAL, LOGNORMAL, POISSON (with reproducible seeds)
- **Agent-Based Modeling**: AGENT_COUNT (hybrid SD/ABM support)
- **Logic**: IF-THEN-ELSE conditionals
- **Operators**: Arithmetic (+, -, *, /, ^), comparison (>, <, >=, <=, ==, !=)

### Data I/O
- **Input Formats**: JSON, YAML, XMILE (Stella/Vensim compatible), InsightMaker
- **Output Formats**:
  - CSV with time-series data (always available)
  - NetCDF for large datasets (optional: `--features with-netcdf`)
  - HDF5 with compression (optional: `--features with-hdf5`)
- **Interoperability**: Import models from commercial SD tools (Stella, Vensim, InsightMaker)
- **Supported Extensions**: `.json`, `.yaml`, `.yml`, `.xmile`, `.stmx` (Stella), `.itmx`

### Protocol Foundations (In Development)
- **MCP (Model Context Protocol)**: Framework for AI agent integration (message structures defined)
- **A2A (Agent-to-Agent)**: Framework for distributed agent communication (protocol design complete)

## Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/greenpdx/rssdsim
cd rssdsim

# Build the project
cargo build --release

# Install globally
cargo install --path .
```

### Basic Usage

```bash
# Run a simulation
rssdsim run model.json -o results.csv

# Run with parameter overrides
rssdsim run model.json -p "contact_rate=10,infectivity=0.3"

# Specify integration method
rssdsim run model.json --integrator rk4

# Validate model structure
rssdsim validate model.json

# Show version and info
rssdsim info
```

## Example Model

### Simple SIR Epidemic Model (JSON)

```json
{
  "model": {
    "name": "SIR Epidemic Model",
    "time": {
      "start": 0,
      "stop": 100,
      "dt": 0.25,
      "units": "days"
    },
    "stocks": [
      {
        "name": "Susceptible",
        "initial": 990,
        "outflows": ["infection_rate"]
      },
      {
        "name": "Infected",
        "initial": 10,
        "inflows": ["infection_rate"],
        "outflows": ["recovery_rate"]
      },
      {
        "name": "Recovered",
        "initial": 0,
        "inflows": ["recovery_rate"]
      }
    ],
    "flows": [
      {
        "name": "infection_rate",
        "equation": "contact_rate * infectivity * Susceptible * Infected / total_population"
      },
      {
        "name": "recovery_rate",
        "equation": "Infected / duration"
      }
    ],
    "auxiliaries": [
      {
        "name": "total_population",
        "equation": "Susceptible + Infected + Recovered"
      }
    ],
    "parameters": [
      {
        "name": "contact_rate",
        "value": 5.0,
        "units": "contacts/person/day"
      },
      {
        "name": "infectivity",
        "value": 0.25,
        "units": "dimensionless"
      },
      {
        "name": "duration",
        "value": 5.0,
        "units": "days"
      }
    ]
  }
}
```

### YAML Format

```yaml
model:
  name: SIR Epidemic Model

  time:
    start: 0
    stop: 100
    dt: 0.25
    units: days

  stocks:
    - name: Susceptible
      initial: 990
      outflows: [infection_rate]

    - name: Infected
      initial: 10
      inflows: [infection_rate]
      outflows: [recovery_rate]

    - name: Recovered
      initial: 0
      inflows: [recovery_rate]

  flows:
    - name: infection_rate
      equation: contact_rate * infectivity * Susceptible * Infected / total_population

    - name: recovery_rate
      equation: Infected / duration

  parameters:
    contact_rate: 5.0
    infectivity: 0.25
    duration: 5.0
```

## Advanced Features

### Delay Functions
Model time delays in information and material flows:
- **DELAY1(input, delay_time, [initial])**: First-order exponential delay
- **DELAY3(input, delay_time, [initial])**: Third-order delay (smoother response)
- **DELAYP(input, delay_time, initial)**: Pipeline delay (fixed time delay)
- **SMOOTH(input, smooth_time)**: Alias for DELAY1

Example: `DELAY1(production, 30)` creates a 30-day exponential delay

### Lookup Tables
Define nonlinear relationships through graphical functions:
- **WITH_LOOKUP(x, x1,y1, x2,y2, ...)**: Inline lookup with linear interpolation

Example: `WITH_LOOKUP(TIME, 0,1.0, 50,1.5, 100,2.0)` defines time-varying multiplier

### Stochastic Elements
Add randomness and uncertainty to models:
- **RANDOM()**: Uniform random [0, 1)
- **UNIFORM(min, max)**: Uniform random in range
- **NORMAL(mean, std_dev)**: Normal distribution
- **LOGNORMAL(mean, std_dev)**: Log-normal distribution
- **POISSON(lambda)**: Poisson distribution

Supports reproducible random seeds for Monte Carlo simulation.

### Agent-Based Modeling
Hybrid system dynamics / agent-based models:
- Individual agents with state and behavior rules
- Agent populations with aggregate statistics
- **AGENT_COUNT()**: Query total agent count
- Integration with SD stocks and flows
- **Bidirectional coupling**: Agents affect SD variables and vice versa
- **Spatial agents**: 1D/2D/3D positioning with movement
- **Agent networks**: Proximity-based and custom network topologies
- **Agent aggregation**: Sum, Mean, Count, Max, Min, Median
- **SD distribution**: Direct, Scaled, PerCapita, Conditional mapping

### Unit Checking
Dimensional analysis for model validation:
- Parse and validate units (meters, kg/s^2, etc.)
- Check dimensional consistency in operations
- Catch unit mismatch errors (e.g., adding meters to seconds)
- Support for SI base dimensions and derived units

### Sensitivity Analysis ⭐ NEW
Comprehensive parameter analysis and uncertainty quantification:
- **Parameter Sweeps**: One-at-a-time sensitivity analysis
- **Latin Hypercube Sampling (LHS)**: Efficient parameter space exploration
- **Morris Screening**: Identify influential parameters using elementary effects
- **Results Export**: CSV export for further analysis
- **Monte Carlo Support**: Reproducible random seeds for uncertainty analysis

Example usage:
```rust
let mut analyzer = SensitivityAnalyzer::new(param_ranges);
analyzer.latin_hypercube_sampling(&model, &config, 100, Some(42))?;
let csv = analyzer.export_results("population_final")?;
```

### Model Structure Analysis ⭐ NEW
Analyze feedback structure and model dependencies:
- **Dependency Graph**: Automatic construction from equations
- **Feedback Loop Detection**: Find all loops up to specified length
- **Loop Polarity**: Classify as Reinforcing (R) or Balancing (B)
- **Structural Reports**: Generate comprehensive structure summaries
- **DOT Export**: Graphviz visualization of model structure

Example usage:
```rust
let analyzer = StructureAnalyzer::new(&model);
println!("Reinforcing loops: {}", analyzer.reinforcing_loops().len());
println!("Balancing loops: {}", analyzer.balancing_loops().len());
std::fs::write("model.dot", analyzer.export_dot())?;
```

### Monte Carlo Simulation ⭐ NEW
Uncertainty quantification and statistical analysis:
- **Multiple Runs**: Execute hundreds or thousands of simulations with random parameters
- **Comprehensive Statistics**: Mean, std dev, percentiles, confidence intervals
- **Parameter Sampling**: Uniform random sampling across parameter ranges
- **Results Export**: CSV export with all statistical measures
- **Memory Efficient**: Optional storage of individual runs

Example usage:
```rust
let param_ranges = vec![
    ParameterRange::new("growth_rate".into(), 0.05, 0.15, 0.1),
    ParameterRange::new("capacity".into(), 800.0, 1200.0, 1000.0),
];

let mc_config = MonteCarloConfig {
    n_runs: 1000,
    seed: Some(42),
    confidence_level: 0.95,
    save_individual_runs: false,
};

let simulator = MonteCarloSimulator::new(param_ranges, mc_config);
let results = simulator.run(&model, &sim_config)?;
let csv = simulator.export_csv(&results, "Population")?;
```

### Advanced Integration Methods ⭐ NEW
Additional numerical integrators for different problem types:
- **Heun (RK2)**: 2nd-order predictor-corrector, balance of speed and accuracy
- **Backward Euler**: Implicit method for stiff systems, unconditionally stable
- **RK4**: 4th-order Runge-Kutta (previously available)
- **Euler**: Simple explicit method (previously available)

Example usage:
```rust
// Use Heun for better accuracy than Euler
let integrator = HeunIntegrator;
let new_state = integrator.step(&model, &state, dt)?;

// Use Backward Euler for stiff systems
let integrator = BackwardEulerIntegrator::default();
let new_state = integrator.step(&model, &state, dt)?;
```

### Large Dataset Output ⭐ NEW
NetCDF and HDF5 support for large simulations:
- **NetCDF**: Self-describing format, widely used in earth/climate science
- **HDF5**: Hierarchical format with compression, ideal for very large datasets
- **Compression**: Reduce file size by 5-10x with HDF5 gzip compression
- **Metadata**: Store variable attributes and model information
- **Optional Features**: Enable with `--features with-netcdf` or `--features with-hdf5`

Example usage:
```rust
// Requires: cargo build --features with-hdf5
use rssdsim::io::HDF5Writer;

HDF5Writer::write_compressed(&results, "output.h5", 6)?;
```

## Project Structure

```
rssdsim/
├── src/
│   ├── main.rs              # CLI entry point and command handling
│   ├── model/               # Model definition and expression evaluation
│   │   ├── mod.rs           # Core model structures
│   │   ├── expression.rs    # Expression parser and evaluator (60+ functions)
│   │   ├── dimension.rs     # Multi-dimensional array support
│   │   └── units.rs         # Unit checking and dimensional analysis
│   ├── simulation/          # Simulation engine
│   │   ├── mod.rs           # Simulation state and engine
│   │   ├── integrator.rs    # Euler and RK4 integrators
│   │   ├── delay.rs         # Delay functions (DELAY1, DELAY3, DELAYP)
│   │   ├── lookup.rs        # Lookup table functions
│   │   ├── stochastic.rs    # Random number generation
│   │   ├── abm.rs           # Agent-based modeling framework
│   │   ├── agent_sd_bridge.rs # Agent-SD bidirectional coupling ⭐ NEW
│   │   └── arrayvalue.rs    # Array value types
│   ├── analysis/            # Analysis tools ⭐ NEW
│   │   ├── mod.rs           # Analysis module exports
│   │   ├── sensitivity.rs   # LHS, Morris screening, parameter sweeps
│   │   └── structure.rs     # Loop detection, dependency analysis
│   ├── io/                  # Data I/O
│   │   ├── parser.rs        # JSON/YAML parser
│   │   ├── xmile.rs         # XMILE format parser
│   │   ├── insightmaker.rs  # InsightMaker format parser
│   │   └── writer.rs        # CSV output writer
│   └── protocol/            # Protocol frameworks (in development)
│       ├── mcp.rs           # Model Context Protocol framework
│       └── a2a.rs           # Agent-to-Agent Protocol framework
├── docs/                    # Documentation
│   ├── NEW_FEATURES.md      # Phase 1 features documentation
│   ├── ADVANCED_FEATURES_V2.md # Phase 2 advanced analysis features
│   ├── IMPLEMENTATION_SUMMARY.md # Complete implementation summary
│   └── MONTE_CARLO_AND_INTEGRATORS.md # Phase 3 Monte Carlo & integrators
├── examples/                # Example models
│   ├── advanced_features.yaml  # Demo of delays, lookups, stochastic
│   └── *.yaml/*.json        # Various model examples
└── tests/                   # Integration tests
```

## Roadmap

### Completed ✅
- [x] Core SD simulation engine
- [x] Euler integration method
- [x] RK4 (Runge-Kutta 4th order) integration
- [x] Expression evaluation with 60+ built-in functions
- [x] XMILE parser (Stella/Vensim compatible)
- [x] InsightMaker format support
- [x] JSON/YAML model format support
- [x] CSV output writer
- [x] Parameter override at runtime
- [x] Model validation tools
- [x] **Delay functions** (DELAY1, DELAY3, DELAYP, SMOOTH)
- [x] **Lookup table functions** (WITH_LOOKUP, linear interpolation)
- [x] **Stochastic elements** (RANDOM, UNIFORM, NORMAL, LOGNORMAL, POISSON)
- [x] **Agent-based modeling framework** (hybrid SD/ABM)
- [x] **Units checking and dimensional analysis**
- [x] **Sensitivity analysis tools** (LHS, Morris screening, parameter sweeps)
- [x] **Model structure analysis** (loop detection, polarity analysis, DOT export)
- [x] **Improved agent-SD integration** (bidirectional coupling, spatial agents, networks)
- [x] **Monte Carlo simulation** (uncertainty quantification, comprehensive statistics) ⭐ NEW
- [x] **Heun integrator** (RK2, 2nd-order predictor-corrector) ⭐ NEW
- [x] **Backward Euler integrator** (implicit method for stiff systems) ⭐ NEW
- [x] **NetCDF output** (optional feature for large datasets) ⭐ NEW
- [x] **HDF5 output** (optional feature with compression) ⭐ NEW
- [x] Dimension/Array framework (data structures complete)
- [x] MCP protocol framework (message structures)
- [x] A2A protocol framework (message structures)

### In Progress 🚧
- [ ] Multi-dimensional variables integration (connect array framework to engine)
- [ ] MCP transport layer (stdio and HTTP)
- [ ] A2A network layer (UDP transport)

### Planned 📋
- [ ] RK45 adaptive integration
- [ ] Pink noise and white noise generators
- [ ] Sobol variance-based sensitivity indices
- [ ] Parallel Monte Carlo and sensitivity analysis (rayon)
- [ ] Eigensystem analysis for stability
- [ ] Calibration and optimization algorithms (gradient-based, genetic algorithms)
- [ ] Real-time simulation dashboard
- [ ] GUI frontend (web-based)
- [ ] Cloud deployment support

## Development Status

**Production Ready ✅:**
- Core stock-flow system dynamics simulation
- Model loading from multiple file formats (JSON, YAML, XMILE, InsightMaker)
- Expression evaluation with 60+ built-in functions
- Four integration methods: Euler, Heun (RK2), RK4, Backward Euler
- Delay functions (DELAY1, DELAY3, DELAYP, SMOOTH)
- Lookup tables with linear interpolation
- Stochastic elements (5 distributions with reproducible seeds)
- Agent-based modeling framework (hybrid SD/ABM)
- Unit checking and dimensional analysis
- Sensitivity analysis tools (LHS, Morris screening, parameter sweeps)
- Model structure analysis (loop detection, polarity analysis, DOT export)
- Improved agent-SD integration (bidirectional coupling, spatial agents, networks)
- **Monte Carlo simulation** (uncertainty quantification with comprehensive statistics)
- **Multiple output formats** (CSV, NetCDF*, HDF5* with compression)
- Model validation and error checking
- Parameter override at runtime

*NetCDF and HDF5 require optional features: `--features with-netcdf` or `--features with-hdf5`

**Experimental 🧪:**
- MCP and A2A protocol frameworks (message structures only, no transport)
- Multi-dimensional array support (data structures exist but not integrated into engine)
- Agent-ABM aggregation functions (framework ready, string argument parsing needed)

**Not Yet Implemented ⏳:**
Features mentioned in roadmap under "Planned" are not yet available.

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Citation

If you use rssdsim in academic work, please cite:

```bibtex
@software{rssdsim2024,
  title = {rssdsim: Rust System Dynamics Simulator},
  author = {Shaun Savage},
  year = {2024},
  url = {https://github.com/greenpdx/rssdsim}
}
```

## Acknowledgments

Built with inspiration from:
- Vensim (Ventana Systems)
- Stella/iThink (isee systems)
- PySD (Python System Dynamics)
- NetLogo (ABM platform)
