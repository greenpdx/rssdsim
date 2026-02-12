# rssdsim - Rust System Dynamics Simulator

A high-performance CLI-based system dynamics simulation framework with comprehensive file format support and extensible architecture.

## Features

### Core Simulation Capabilities
- **Stock-Flow Modeling**: Traditional system dynamics with stocks, flows, auxiliaries, and parameters
- **Expression Evaluation**: Full equation parsing with 47+ built-in functions
- **Multiple Integration Methods**: Euler and RK4 (Runge-Kutta 4th order)
- **Model Validation**: Structural validation and dependency checking
- **Parameter Overrides**: Runtime parameter modification

### Built-in Functions
Complete function library including:
- **Math**: MIN, MAX, ABS, SQRT, EXP, LN, LOG, LOG10, POW, MODULO
- **Trigonometric**: SIN, COS, TAN, ASIN, ACOS, ATAN
- **Rounding**: FLOOR, CEIL, ROUND
- **System Dynamics**: PULSE, STEP, RAMP, TIME
- **Logic**: IF-THEN-ELSE conditionals
- **Operators**: Arithmetic (+, -, *, /, ^), comparison (>, <, >=, <=, ==, !=)

### Data I/O
- **Input Formats**: JSON, YAML, XMILE (Stella/Vensim compatible), InsightMaker
- **Output Formats**: CSV with time-series data
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

## Project Structure

```
rssdsim/
├── src/
│   ├── main.rs              # CLI entry point and command handling
│   ├── model/               # Model definition and expression evaluation
│   │   ├── mod.rs           # Core model structures
│   │   ├── expression.rs    # Expression parser and evaluator
│   │   └── dimension.rs     # Multi-dimensional array support (framework)
│   ├── simulation/          # Simulation engine
│   │   ├── mod.rs           # Simulation state and engine
│   │   ├── integrator.rs    # Euler and RK4 integrators
│   │   └── arrayvalue.rs    # Array value types (framework)
│   ├── io/                  # Data I/O
│   │   ├── parser.rs        # JSON/YAML parser
│   │   ├── xmile.rs         # XMILE format parser
│   │   ├── insightmaker.rs  # InsightMaker format parser
│   │   └── writer.rs        # CSV output writer
│   └── protocol/            # Protocol frameworks (in development)
│       ├── mcp.rs           # Model Context Protocol framework
│       └── a2a.rs           # Agent-to-Agent Protocol framework
├── docs/                    # Documentation
├── examples/                # Example models
└── tests/                   # Integration tests
```

## Roadmap

### Completed
- [x] Core SD simulation engine
- [x] Euler integration method
- [x] RK4 (Runge-Kutta 4th order) integration
- [x] Expression evaluation with 47+ built-in functions
- [x] XMILE parser (Stella/Vensim compatible)
- [x] InsightMaker format support
- [x] JSON/YAML model format support
- [x] CSV output writer
- [x] Parameter override at runtime
- [x] Model validation tools
- [x] Dimension/Array framework (data structures complete)
- [x] MCP protocol framework (message structures)
- [x] A2A protocol framework (message structures)

### In Progress
- [ ] Multi-dimensional variables integration (connect array framework to engine)
- [ ] MCP transport layer (stdio and HTTP)
- [ ] A2A network layer (UDP transport)

### Planned
- [ ] RK45 adaptive integration
- [ ] Heun and Backward Euler integrators
- [ ] Delay functions (DELAY, SMOOTH)
- [ ] Lookup table functions
- [ ] Stochastic elements (white noise, pink noise, Poisson processes)
- [ ] Sensitivity analysis tools (parameter sweeps, LHS)
- [ ] Monte Carlo simulation
- [ ] Agent-based modeling components
- [ ] Units checking and dimensional analysis
- [ ] Model analysis (loop dominance, structural analysis)
- [ ] NetCDF/HDF5 output for large datasets
- [ ] Optimization algorithms
- [ ] GUI frontend (web-based)
- [ ] Cloud deployment support

## Development Status

**Production Ready:**
- Core stock-flow system dynamics simulation
- Model loading from multiple file formats
- Expression evaluation and built-in functions
- Basic validation and error checking

**Experimental:**
- MCP and A2A protocol frameworks (message structures only, no transport)
- Multi-dimensional array support (data structures exist but not integrated into engine)

**Not Yet Implemented:**
Features mentioned in roadmap under "Planned" are not yet available. The CLI currently supports only `run`, `validate`, and `info` commands.

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
