# Quick Start Guide

## Installation

```bash
cd rsedsim
cargo build --release
```

## Running Your First Simulation

### 1. Exponential Growth Example

```bash
cargo run --release -- run examples/exponential_growth.yaml -o results.csv
```

This simulates a population growing at 10% per year starting from 100 people.

### 2. SIR Epidemic Model

```bash
cargo run --release -- run examples/sir_epidemic.yaml -o sir_results.csv
```

This simulates a classic SIR (Susceptible-Infected-Recovered) epidemic model.

### 3. Bank Account Example (JSON format)

```bash
cargo run --release -- run examples/bank_account.json -o account.csv
```

This simulates compound interest on a bank account.

## Parameter Overrides

You can override parameters at runtime:

```bash
cargo run -- run examples/sir_epidemic.yaml -o high_contact.csv -p "contact_rate=10"
```

Multiple parameters:

```bash
cargo run -- run examples/sir_epidemic.yaml -o modified.csv -p "contact_rate=10,infectivity=0.4"
```

## Validation

Validate a model file before running:

```bash
cargo run -- validate examples/sir_epidemic.yaml
```

## Viewing Results

Results are saved as CSV files. You can view them with any spreadsheet software or plot with Python:

```python
import pandas as pd
import matplotlib.pyplot as plt

df = pd.read_csv('sir_results.csv')
df.plot(x='Time', y=['Susceptible', 'Infected', 'Recovered'])
plt.show()
```

## Commands

- `rsedsim run <model> [options]` - Run a simulation
- `rsedsim validate <model>` - Validate a model file
- `rsedsim info` - Show version and features

## Options

- `-o, --output <file>` - Output file path (default: results.csv)
- `-p, --params <params>` - Override parameters (format: "param1=value1,param2=value2")
- `--integrator <method>` - Integration method: euler or rk4 (default: euler)

## What's Implemented

✅ Stock-flow models
✅ Expression evaluation (arithmetic, functions)
✅ Multiple integrators (Euler, RK4)
✅ JSON/YAML model formats
✅ CSV output
✅ Parameter overrides
✅ Model validation
✅ MCP & A2A protocol stubs

## What's Next

The framework is ready for expansion:
- Multi-dimensional variables (arrays, subscripts)
- Agent-based modeling
- Hybrid SD-ABM models
- More built-in functions (delays, lookups)
- Sensitivity analysis
- Monte Carlo simulation

See [ARCHITECTURE.md](ARCHITECTURE.md) for the complete design.
