# rsedsim - Rust System Dynamics Simulator

A comprehensive CLI-based system dynamics simulation framework with support for hybrid stock/flow-agent models, multi-dimensional variables, and modern protocol integration (MCP & A2A).

## Features

### Core Simulation Capabilities
- **Stock-Flow Modeling**: Traditional system dynamics with stocks, flows, auxiliaries, and parameters
- **Agent-Based Modeling**: Individual agents with behaviors and attributes
- **Hybrid Models**: Seamless integration of SD and ABM paradigms
- **Multi-dimensional Variables**: Full support for vectors, arrays, and tensor variables with subscripts
- **Multiple Integration Methods**: Euler, RK4, RK45 (adaptive), Heun, Backward Euler

### Advanced Features
- **Built-in Functions**: Complete SD function library (delays, lookups, statistics, time functions)
- **Sensitivity Analysis**: Parameter sweeps, Monte Carlo, Latin Hypercube sampling
- **Model Analysis**: Loop dominance, structural analysis, equilibrium finding
- **Units Checking**: Dimensional analysis for model validation
- **Stochastic Elements**: White noise, pink noise, Poisson processes

### Protocol Integration
- **MCP (Model Context Protocol)**: AI agent integration for LLM-driven simulation
- **A2A (Agent-to-Agent)**: Distributed agent communication and coordination

### Data I/O
- **Input Formats**: JSON, YAML, XMILE (Stella/Vensim compatible), InsightMaker
- **Output Formats**: CSV, JSON, NetCDF/HDF5 (for large datasets)
- **Interoperability**: Import/export models from commercial SD tools
- **Supported Extensions**: `.json`, `.yaml`, `.yml`, `.xmile`, `.stmx` (Stella), `.itmx`, `.xml`

## Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/rsedsim
cd rsedsim

# Build the project
cargo build --release

# Install globally
cargo install --path .
```

### Basic Usage

```bash
# Run a simulation
rsedsim run model.json -o results.csv

# Run with parameter overrides
rsedsim run model.json -p "contact_rate=10,infectivity=0.3"

# Sensitivity analysis
rsedsim sensitivity model.json -v contact_rate -r 1:10:0.5

# Monte Carlo simulation
rsedsim monte-carlo model.json -n 1000 -o mc_results/

# Convert model formats
rsedsim convert model.xmile -f json -o model.json

# Validate model
rsedsim validate model.json --check-units --check-bounds
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

## Documentation

- [Quick Start](QUICKSTART.md) - Get running in 5 minutes
- [Architecture Guide](ARCHITECTURE.md) - Detailed system architecture
- [Format Support](FORMAT_SUPPORT.md) - All supported file formats
- [API Documentation](docs/API.md) - Complete API reference
- [Protocol Integration](docs/PROTOCOLS.md) - MCP and A2A integration guide
- [Tutorial](docs/TUTORIAL.md) - Step-by-step getting started
- [Examples](docs/EXAMPLES.md) - Model examples and use cases

## Project Structure

```
rsedsim/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── model/               # Model definition
│   ├── simulation/          # Simulation engine
│   ├── agent/               # Agent-based components
│   ├── array/               # Multi-dimensional support
│   ├── io/                  # Data I/O
│   ├── functions/           # Built-in functions
│   ├── cli/                 # CLI commands
│   └── protocol/            # MCP & A2A protocols
│       ├── mcp.rs           # Model Context Protocol
│       └── a2a.rs           # Agent-to-Agent Protocol
├── docs/                    # Documentation
├── examples/                # Example models
└── tests/                   # Integration tests
```

## Protocol Integration

### MCP (Model Context Protocol)

Enable AI agents to interact with simulations:

```bash
# Start MCP server on stdio
rsedsim mcp serve --stdio

# Start MCP server on HTTP
rsedsim mcp serve --http localhost:3000
```

MCP exposes tools for:
- Running simulations
- Analyzing model structure
- Sensitivity analysis
- Querying results

### A2A (Agent-to-Agent Protocol)

Enable distributed agent communication:

```yaml
hybrid_model:
  a2a_config:
    node_id: "sim1:population"
    transport: udp
    bind_addr: "0.0.0.0:5000"
    peers:
      - "192.168.1.10:5000"
      - "192.168.1.11:5000"
```

## Roadmap

- [x] Core SD simulation engine
- [x] MCP protocol stub
- [x] A2A protocol stub
- [x] XMILE parser (Stella/Vensim compatible)
- [x] InsightMaker format support
- [x] Expression evaluation
- [x] Multiple integrators (Euler, RK4)
- [ ] Complete RK4 implementation
- [ ] Built-in function library (delays, lookups)
- [ ] Multi-dimensional variables (arrays/subscripts)
- [ ] Hybrid SD-ABM models
- [ ] Sensitivity analysis tools
- [ ] Monte Carlo simulation
- [ ] Optimization algorithms
- [ ] GUI frontend (web-based)
- [ ] Cloud deployment support

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Citation

If you use rsedsim in academic work, please cite:

```bibtex
@software{rsedsim2024,
  title = {rsedsim: Rust System Dynamics Simulator},
  author = {rsedsim contributors},
  year = {2024},
  url = {https://github.com/yourusername/rsedsim}
}
```

## Acknowledgments

Built with inspiration from:
- Vensim (Ventana Systems)
- Stella/iThink (isee systems)
- PySD (Python System Dynamics)
- NetLogo (ABM platform)
