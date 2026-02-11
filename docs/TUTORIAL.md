# rsedsim Tutorial: Getting Started

This tutorial will guide you through creating, running, and analyzing your first system dynamics model with rsedsim.

## Prerequisites

- Rust installed (1.70 or later)
- Basic understanding of system dynamics concepts
- Familiarity with command-line tools

## Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/rsedsim
cd rsedsim

# Build the project
cargo build --release

# Install globally
cargo install --path .

# Verify installation
rsedsim --version
```

## Tutorial Outline

1. [Creating Your First Model](#lesson-1-creating-your-first-model)
2. [Running Simulations](#lesson-2-running-simulations)
3. [Multi-dimensional Models](#lesson-3-multi-dimensional-models)
4. [Hybrid SD-Agent Models](#lesson-4-hybrid-models)
5. [Sensitivity Analysis](#lesson-5-sensitivity-analysis)
6. [Protocol Integration](#lesson-6-protocol-integration)

---

## Lesson 1: Creating Your First Model

We'll start with a simple exponential growth model: a bank account with compound interest.

### Model Description

- **Stock**: `balance` (initial: $1000)
- **Flow**: `interest` (inflow to balance)
- **Parameter**: `interest_rate` (5% per year)
- **Equation**: `interest = balance * interest_rate`

### Create the Model File

Create a file named `bank_account.yaml`:

```yaml
model:
  name: Bank Account with Interest

  time:
    start: 0
    stop: 20
    dt: 0.1
    units: years

  stocks:
    - name: balance
      initial: 1000
      inflows: [interest]
      units: dollars

  flows:
    - name: interest
      equation: balance * interest_rate
      units: dollars/year

  parameters:
    - name: interest_rate
      value: 0.05
      units: 1/year
      description: Annual interest rate (5%)
```

### Run the Model

```bash
rsedsim run bank_account.yaml -o results.csv
```

### View Results

```bash
# View first 10 rows
head -n 10 results.csv
```

Output:
```
Time,balance
0.0,1000.0
0.1,1005.0
0.2,1010.0
...
20.0,2653.3
```

### Visualize (using Python)

```python
import pandas as pd
import matplotlib.pyplot as plt

df = pd.read_csv('results.csv')
plt.plot(df['Time'], df['balance'])
plt.xlabel('Time (years)')
plt.ylabel('Balance ($)')
plt.title('Bank Account Growth')
plt.grid(True)
plt.show()
```

### Exercise 1.1

Modify the model to include:
1. Regular monthly deposits of $100
2. An annual fee of $50

<details>
<summary>Solution</summary>

```yaml
model:
  name: Bank Account with Deposits and Fees

  time:
    start: 0
    stop: 20
    dt: 0.1
    units: years

  stocks:
    - name: balance
      initial: 1000
      inflows: [interest, deposits]
      outflows: [fees]
      units: dollars

  flows:
    - name: interest
      equation: balance * interest_rate
      units: dollars/year

    - name: deposits
      equation: monthly_deposit * 12
      units: dollars/year

    - name: fees
      equation: annual_fee
      units: dollars/year

  parameters:
    - name: interest_rate
      value: 0.05
      units: 1/year

    - name: monthly_deposit
      value: 100
      units: dollars/month

    - name: annual_fee
      value: 50
      units: dollars/year
```
</details>

---

## Lesson 2: Running Simulations

Learn different ways to run and control simulations.

### Basic Simulation

```bash
# Default run
rsedsim run model.yaml

# Specify output format
rsedsim run model.yaml -o results.csv
rsedsim run model.yaml -o results.json --format json

# Override parameters
rsedsim run bank_account.yaml -p "interest_rate=0.08"

# Override multiple parameters
rsedsim run bank_account.yaml -p "interest_rate=0.08,monthly_deposit=200"

# Override time configuration
rsedsim run bank_account.yaml --start 0 --stop 30 --dt 0.05
```

### Interactive Mode

```bash
rsedsim interactive bank_account.yaml
```

Interactive commands:
```
> step 10          # Advance 10 time steps
Current time: 1.0, balance: 1051.27

> set interest_rate 0.10    # Change parameter
Parameter updated

> continue         # Run until end
Simulation complete

> plot balance     # Show ASCII plot
Balance over time:
2000 │                                    ╭─
1800 │                                ╭───╯
1600 │                           ╭────╯
1400 │                     ╭─────╯
1200 │              ╭──────╯
1000 │──────────────╯
     └──────────────────────────────────────
     0                                     20

> quit
```

### Programmatic API (Rust)

```rust
use rsedsim::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load model
    let model = Model::load("bank_account.yaml")?;

    // Configure simulation
    let config = SimulationConfig::builder()
        .integration_method(IntegrationMethod::RK4)
        .output_interval(0.5)
        .build();

    // Run simulation
    let mut sim = SimulationEngine::new(model, config)?;
    let results = sim.run()?;

    // Access final value
    let final_balance = results.final_state().get_stock("balance")?;
    println!("Final balance: ${:.2}", final_balance);

    Ok(())
}
```

### Exercise 2.1

Run the bank account model with different interest rates (3%, 5%, 7%, 10%) and compare the final balances after 20 years.

<details>
<summary>Solution</summary>

```bash
for rate in 0.03 0.05 0.07 0.10; do
  echo "Interest rate: $rate"
  rsedsim run bank_account.yaml -p "interest_rate=$rate" -o results_$rate.csv
  tail -n 1 results_$rate.csv | cut -d',' -f2
done
```

Results:
- 3%: $1806.11
- 5%: $2653.30
- 7%: $3869.68
- 10%: $6727.50
</details>

---

## Lesson 3: Multi-dimensional Models

Learn to work with subscripted variables (arrays).

### Example: Multi-Region Population Model

```yaml
model:
  name: Multi-Region Population

  time:
    start: 0
    stop: 50
    dt: 0.25
    units: years

  # Define dimensions
  subscripts:
    region: [North, South, East, West]
    age_group: [Young, Adult, Senior]

    # Define subsets
    coastal:
      subset_of: region
      elements: [North, East]

  # Multi-dimensional stock
  stocks:
    - name: Population
      dimensions: [region, age_group]
      initial:
        North:
          Young: 10000
          Adult: 25000
          Senior: 5000
        South:
          Young: 8000
          Adult: 20000
          Senior: 4000
        East:
          Young: 12000
          Adult: 30000
          Senior: 6000
        West:
          Young: 9000
          Adult: 22000
          Senior: 5500
      inflows: [births, immigration]
      outflows: [deaths, aging, emigration]

  flows:
    - name: births
      dimensions: [region]
      equation: SUM(Population[region, age_group], age_group) * birth_rate[region]

    - name: deaths
      dimensions: [region, age_group]
      equation: Population[region, age_group] / lifespan[age_group]

    - name: aging
      dimensions: [region, age_group]
      equation: Population[region, age_group] / aging_time[age_group]

  auxiliaries:
    - name: total_population
      dimensions: [region]
      equation: SUM(Population[region, age_group], age_group)

    - name: coastal_population
      equation: SUM(Population[coastal, age_group], region, age_group)

  parameters:
    - name: birth_rate
      dimensions: [region]
      values:
        North: 0.015
        South: 0.018
        East: 0.014
        West: 0.016

    - name: lifespan
      dimensions: [age_group]
      values:
        Young: 20
        Adult: 40
        Senior: 25

    - name: aging_time
      dimensions: [age_group]
      values:
        Young: 15
        Adult: 25
        Senior: 999999  # Don't age out of Senior
```

### Run Multi-dimensional Model

```bash
rsedsim run multi_region.yaml -o results.csv
```

Results will include all combinations:
```
Time,Population[North,Young],Population[North,Adult],...
0.0,10000,25000,...
```

### Access Specific Subscripts

```bash
# Export only coastal regions
rsedsim run multi_region.yaml --select "Population[coastal,*]" -o coastal_results.csv

# Export only senior population
rsedsim run multi_region.yaml --select "Population[*,Senior]" -o senior_results.csv
```

### Exercise 3.1

Add migration flows between regions:
- 2% of people move from inland (South, West) to coastal (North, East) per year
- 1% move from coastal to inland

<details>
<summary>Hint</summary>

You'll need:
1. Two new flows: `coastal_migration` and `inland_migration`
2. Use subset references: `coastal` and create `inland` subset
3. Flows will have `[region, age_group]` dimensions
</details>

---

## Lesson 4: Hybrid Models

Combine system dynamics with agent-based modeling.

### Example: Simple Epidemic with Agents

```yaml
hybrid_model:
  name: Agent-Based SIR Epidemic

  time:
    start: 0
    stop: 100
    dt: 0.25
    units: days

  # SD Components
  sd_model:
    stocks:
      - name: TotalInfected
        initial: 0
        inflows: [new_infections]

    flows:
      - name: new_infections
        equation: AGENT_AGGREGATE("infection_count")

    parameters:
      - name: infectivity
        value: 0.25
      - name: recovery_time
        value: 10

  # Agent Population
  agent_populations:
    - name: People
      count: 1000

      attributes:
        - name: state
          type: enum
          values: [Susceptible, Infected, Recovered]
          initial: Susceptible

        - name: days_infected
          type: float
          initial: 0

        - name: contacts
          type: integer
          initial: RANDOM_POISSON(5)

      # Initialize 10 infected agents
      initial_conditions:
        - count: 10
          state: Infected

      # Agent behavior
      behavior:
        on_step:
          # Recovery logic
          - if this.state == Infected:
              this.days_infected += DT
              if this.days_infected >= recovery_time:
                this.state = Recovered

          # Infection logic
          - if this.state == Susceptible:
              # Sample random contacts
              contact_agents = SAMPLE_AGENTS(this.contacts)
              infected_contacts = COUNT(contact_agents, agent.state == Infected)

              infection_prob = infectivity * infected_contacts / this.contacts
              if RANDOM() < infection_prob * DT:
                this.state = Infected
                this.days_infected = 0

      # Aggregate to SD
      outputs_to_sd:
        - infection_count:
            type: count
            condition: agent.state == Infected AND agent.days_infected < DT
```

### Run Hybrid Model

```bash
rsedsim run epidemic_hybrid.yaml -o results.csv
```

### Exercise 4.1

Add spatial structure:
1. Give each agent an `(x, y)` location
2. Agents only contact others within distance 10
3. Track spatial clustering of infections

---

## Lesson 5: Sensitivity Analysis

Understand how parameters affect model behavior.

### One-at-a-Time Sensitivity

```bash
# Vary contact_rate from 1 to 10 with 20 samples
rsedsim sensitivity sir_model.yaml \
  -p contact_rate \
  -r 1:10:20 \
  -o sensitivity/

# Results in sensitivity/results_*.csv
```

### Multi-parameter Sensitivity

```bash
# Latin Hypercube sampling
rsedsim sensitivity sir_model.yaml \
  -p contact_rate:1:10 \
  -p infectivity:0.1:0.5 \
  -p duration:3:10 \
  --samples 100 \
  --method lhs \
  -o sensitivity_lhs/
```

### Analyze Results (Python)

```python
import pandas as pd
import numpy as np
import glob

# Load all results
files = glob.glob('sensitivity_lhs/results_*.csv')
results = []

for f in files:
    df = pd.read_csv(f)
    peak_infected = df['Infected'].max()
    peak_time = df.loc[df['Infected'].idxmax(), 'Time']

    # Extract parameters from filename
    # Format: results_contact_rate_5.2_infectivity_0.3_duration_7.1.csv
    params = {}
    parts = f.split('_')
    for i in range(1, len(parts)-1, 2):
        params[parts[i]] = float(parts[i+1])

    results.append({
        **params,
        'peak_infected': peak_infected,
        'peak_time': peak_time
    })

df_results = pd.DataFrame(results)

# Calculate correlations
print("Correlation with peak_infected:")
print(df_results[['contact_rate', 'infectivity', 'duration', 'peak_infected']].corr()['peak_infected'])
```

### Monte Carlo Simulation

```bash
# Define parameter distributions in config file
cat > mc_config.yaml <<EOF
parameters:
  contact_rate:
    distribution: normal
    mean: 5
    std: 1
  infectivity:
    distribution: uniform
    min: 0.2
    max: 0.3
  duration:
    distribution: lognormal
    mean: 5
    std: 2
EOF

# Run 1000 Monte Carlo simulations
rsedsim monte-carlo sir_model.yaml \
  --config mc_config.yaml \
  --runs 1000 \
  --parallel \
  -o mc_results/

# Analyze results
rsedsim analyze mc_results/ --statistics
```

---

## Lesson 6: Protocol Integration

### MCP: LLM Integration

Start MCP server:
```bash
rsedsim mcp serve --stdio
```

Use with Claude Desktop (add to config):
```json
{
  "mcpServers": {
    "rsedsim": {
      "command": "rsedsim",
      "args": ["mcp", "serve", "--stdio"]
    }
  }
}
```

Now you can ask Claude:
> "Run the SIR model with contact_rate=8 and tell me when infections peak"

Claude will:
1. Call `run_simulation` tool
2. Call `get_variable_timeseries` tool
3. Analyze the data and respond

### A2A: Distributed Agents

Configure distributed simulation across 3 machines:

**Machine 1** (`config_node1.yaml`):
```yaml
hybrid_model:
  name: Distributed Epidemic - Node 1

  a2a_config:
    node_id: "epidemic:node1"
    transport: udp
    bind_addr: "0.0.0.0:5000"
    peers:
      - "192.168.1.11:5000"  # Node 2
      - "192.168.1.12:5000"  # Node 3

  agent_populations:
    - name: Region1People
      count: 5000
      # ... agent config
```

**Machine 2 & 3**: Similar configs with different node IDs and populations

Run on each machine:
```bash
# Machine 1
rsedsim run config_node1.yaml --distributed

# Machine 2
rsedsim run config_node2.yaml --distributed

# Machine 3
rsedsim run config_node3.yaml --distributed
```

Agents will automatically discover and communicate across network!

---

## Next Steps

- Explore the [Examples](EXAMPLES.md) for more complex models
- Read the [API Documentation](API.md) for programmatic usage
- Check [PROTOCOLS.md](PROTOCOLS.md) for advanced protocol integration
- Read [ARCHITECTURE.md](../ARCHITECTURE.md) to understand internals

## Common Issues

**Problem**: Model won't run - "Circular dependency detected"

**Solution**: Check that your model doesn't have circular references. Use:
```bash
rsedsim validate model.yaml --check-cycles
```

**Problem**: Results show NaN or Inf values

**Solution**: Try a smaller time step (dt) or use a more stable integrator:
```bash
rsedsim run model.yaml --dt 0.01 --integrator rk4
```

**Problem**: Simulation is slow

**Solution**:
- Increase time step if possible
- Use Euler integrator for faster (but less accurate) results
- Reduce output frequency: `--output-interval 1.0`

## Getting Help

- Documentation: `rsedsim --help`
- Command help: `rsedsim run --help`
- Report issues: https://github.com/yourusername/rsedsim/issues
- Discussions: https://github.com/yourusername/rsedsim/discussions
