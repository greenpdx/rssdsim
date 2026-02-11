# Example Models

This document contains a collection of example models demonstrating various features of rsedsim.

## Table of Contents

1. [Classic System Dynamics Models](#classic-system-dynamics-models)
2. [Multi-dimensional Models](#multi-dimensional-models)
3. [Hybrid SD-Agent Models](#hybrid-sd-agent-models)
4. [Advanced Features](#advanced-features)
5. [Real-World Applications](#real-world-applications)

---

## Classic System Dynamics Models

### 1. Exponential Growth/Decay

**File**: `examples/exponential_growth.yaml`

The simplest SD model - a stock with proportional feedback.

```yaml
model:
  name: Exponential Growth

  time:
    start: 0
    stop: 10
    dt: 0.1

  stocks:
    - name: Population
      initial: 100
      inflows: [births]

  flows:
    - name: births
      equation: Population * birth_rate

  parameters:
    - name: birth_rate
      value: 0.1
      description: Net birth rate (births - deaths)
```

**Analytical Solution**: `P(t) = P0 * e^(r*t)`

**Run**: `rsedsim run examples/exponential_growth.yaml`

---

### 2. Goal-Seeking Behavior

**File**: `examples/goal_seeking.yaml`

Stock adjusts toward a goal value (negative feedback).

```yaml
model:
  name: Inventory Management

  time:
    start: 0
    stop: 50
    dt: 0.25

  stocks:
    - name: Inventory
      initial: 50
      inflows: [production]
      outflows: [sales]

  flows:
    - name: production
      equation: MAX(0, (target_inventory - Inventory) / adjustment_time)

    - name: sales
      equation: 10  # Constant demand

  parameters:
    - name: target_inventory
      value: 100

    - name: adjustment_time
      value: 5
```

**Behavior**: Inventory oscillates and converges to target.

---

### 3. S-Shaped Growth (Logistic)

**File**: `examples/logistic_growth.yaml`

Growth limited by carrying capacity.

```yaml
model:
  name: Logistic Growth

  time:
    start: 0
    stop: 100
    dt: 0.25

  stocks:
    - name: Population
      initial: 10
      inflows: [growth]

  flows:
    - name: growth
      equation: growth_rate * Population * (1 - Population / carrying_capacity)

  parameters:
    - name: growth_rate
      value: 0.1

    - name: carrying_capacity
      value: 1000
```

**Analytical Solution**: `P(t) = K / (1 + ((K - P0) / P0) * e^(-r*t))`

---

### 4. Oscillation (Predator-Prey)

**File**: `examples/predator_prey.yaml`

Classic Lotka-Volterra model.

```yaml
model:
  name: Predator-Prey (Lotka-Volterra)

  time:
    start: 0
    stop: 200
    dt: 0.1

  stocks:
    - name: Prey
      initial: 100
      inflows: [prey_growth]
      outflows: [predation]

    - name: Predators
      initial: 20
      inflows: [predator_growth]
      outflows: [predator_deaths]

  flows:
    - name: prey_growth
      equation: prey_birth_rate * Prey

    - name: predation
      equation: predation_rate * Prey * Predators

    - name: predator_growth
      equation: efficiency * predation_rate * Prey * Predators

    - name: predator_deaths
      equation: predator_death_rate * Predators

  parameters:
    - name: prey_birth_rate
      value: 0.5

    - name: predation_rate
      value: 0.01

    - name: efficiency
      value: 0.5
      description: Energy conversion efficiency

    - name: predator_death_rate
      value: 0.2
```

**Behavior**: Cyclical oscillations with phase lag between prey and predators.

---

### 5. SIR Epidemic Model

**File**: `examples/sir_epidemic.yaml`

Classic compartmental epidemic model.

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
      units: people

    - name: Infected
      initial: 10
      inflows: [infection_rate]
      outflows: [recovery_rate]
      units: people

    - name: Recovered
      initial: 0
      inflows: [recovery_rate]
      units: people

  flows:
    - name: infection_rate
      equation: contact_rate * infectivity * Susceptible * Infected / total_population
      units: people/day

    - name: recovery_rate
      equation: Infected / recovery_time
      units: people/day

  auxiliaries:
    - name: total_population
      equation: Susceptible + Infected + Recovered
      units: people

    - name: R0
      equation: contact_rate * infectivity * recovery_time
      description: Basic reproduction number

  parameters:
    - name: contact_rate
      value: 5.0
      units: contacts/person/day

    - name: infectivity
      value: 0.25
      units: dimensionless

    - name: recovery_time
      value: 10.0
      units: days
```

**Key Metric**: `R0 > 1` → epidemic spreads, `R0 < 1` → epidemic dies out

**Run with interventions**:
```bash
# Social distancing (reduce contacts)
rsedsim run examples/sir_epidemic.yaml -p "contact_rate=2"

# Vaccination (reduce susceptibles)
rsedsim run examples/sir_epidemic.yaml -p "Susceptible=700,Recovered=300"
```

---

## Multi-dimensional Models

### 6. Age-Structured Population

**File**: `examples/age_structured_population.yaml`

Population aging through age cohorts.

```yaml
model:
  name: Age-Structured Population

  time:
    start: 0
    stop: 100
    dt: 0.25
    units: years

  subscripts:
    age_group: [Child, Youth, Adult, Senior]

  stocks:
    - name: Population
      dimensions: [age_group]
      initial:
        Child: 5000
        Youth: 4000
        Adult: 8000
        Senior: 3000
      inflows: [aging_in, births_by_age]
      outflows: [aging_out, deaths]

  flows:
    - name: aging_in
      dimensions: [age_group]
      equation: |
        IF age_group == "Child" THEN
          0  # No inflow to first age group
        ELSE
          Population[PREV(age_group)] / aging_time[PREV(age_group)]

    - name: aging_out
      dimensions: [age_group]
      equation: |
        IF age_group == "Senior" THEN
          0  # Seniors don't age out
        ELSE
          Population[age_group] / aging_time[age_group]

    - name: births_by_age
      dimensions: [age_group]
      equation: |
        IF age_group == "Child" THEN
          SUM(Population[reproductive_ages] * fertility_rate[reproductive_ages])
        ELSE
          0

    - name: deaths
      dimensions: [age_group]
      equation: Population[age_group] * mortality_rate[age_group]

  parameters:
    - name: aging_time
      dimensions: [age_group]
      values:
        Child: 12
        Youth: 8
        Adult: 35
        Senior: 999999  # Don't age out

    - name: fertility_rate
      dimensions: [age_group]
      values:
        Child: 0
        Youth: 0.02
        Adult: 0.015
        Senior: 0

    - name: mortality_rate
      dimensions: [age_group]
      values:
        Child: 0.001
        Youth: 0.0005
        Adult: 0.002
        Senior: 0.02

  subscripts:
    reproductive_ages:
      subset_of: age_group
      elements: [Youth, Adult]
```

---

### 7. Multi-Region Economic Model

**File**: `examples/multi_region_economy.yaml`

Economic model with trade between regions.

```yaml
model:
  name: Multi-Region Economic Model

  time:
    start: 0
    stop: 50
    dt: 0.25
    units: years

  subscripts:
    region: [North, South, East, West]

  stocks:
    - name: Capital
      dimensions: [region]
      initial:
        North: 1000
        South: 800
        East: 1200
        West: 900
      inflows: [investment]
      outflows: [depreciation]

    - name: Labor
      dimensions: [region]
      initial:
        North: 500
        South: 400
        East: 600
        West: 450
      inflows: [labor_growth, immigration]
      outflows: [emigration]

  flows:
    - name: investment
      dimensions: [region]
      equation: output[region] * savings_rate[region]

    - name: depreciation
      dimensions: [region]
      equation: Capital[region] / capital_lifetime

    - name: labor_growth
      dimensions: [region]
      equation: Labor[region] * population_growth_rate[region]

    - name: immigration
      dimensions: [region]
      equation: SUM((wage[other_region] - wage[region]) * migration_sensitivity, other_region != region)

    - name: emigration
      dimensions: [region]
      equation: SUM(MAX(0, (wage[other_region] - wage[region]) * migration_sensitivity), other_region != region)

  auxiliaries:
    - name: output
      dimensions: [region]
      equation: productivity * (Capital[region] ^ capital_share) * (Labor[region] ^ (1 - capital_share))

    - name: wage
      dimensions: [region]
      equation: output[region] / Labor[region]

  parameters:
    - name: productivity
      value: 1.0

    - name: capital_share
      value: 0.3

    - name: capital_lifetime
      value: 20

    - name: savings_rate
      dimensions: [region]
      values:
        North: 0.15
        South: 0.12
        East: 0.18
        West: 0.14

    - name: population_growth_rate
      dimensions: [region]
      values:
        North: 0.01
        South: 0.015
        East: 0.008
        West: 0.012

    - name: migration_sensitivity
      value: 0.001
```

---

## Hybrid SD-Agent Models

### 8. Traffic Flow (SD + Agents)

**File**: `examples/hybrid_traffic.yaml`

Aggregate traffic flow (SD) with individual vehicles (agents).

```yaml
hybrid_model:
  name: Highway Traffic Flow

  time:
    start: 0
    stop: 60
    dt: 0.1
    units: minutes

  sd_model:
    stocks:
      - name: AverageSpeed
        initial: 60
        inflows: [speed_increase]
        outflows: [speed_decrease]
        units: mph

    flows:
      - name: speed_increase
        equation: (free_flow_speed - AverageSpeed) / speed_adjustment_time

      - name: speed_decrease
        equation: congestion_effect * (AverageSpeed / free_flow_speed)

    auxiliaries:
      - name: congestion_effect
        equation: AGENT_AGGREGATE("vehicle_density") / max_density

    parameters:
      - name: free_flow_speed
        value: 65

      - name: max_density
        value: 100

      - name: speed_adjustment_time
        value: 2

  agent_populations:
    - name: Vehicles
      count: 500

      attributes:
        - name: position
          type: float
          initial: RANDOM_UNIFORM(0, 1000)
          units: meters

        - name: speed
          type: float
          initial: 60
          units: mph

        - name: desired_speed
          type: float
          initial: RANDOM_NORMAL(65, 5)

      behavior:
        on_step:
          # Update speed based on traffic ahead
          - vehicles_ahead = COUNT_AGENTS_IN_RANGE(this.position, this.position + 50)
          - if vehicles_ahead > 3:
              this.speed = this.speed * 0.9  # Slow down
            else:
              this.speed = MIN(this.desired_speed, AverageSpeed)

          # Update position
          - this.position += this.speed * DT * 0.44704  # mph to m/s

          # Wrap around highway
          - if this.position > 1000:
              this.position -= 1000

      outputs_to_sd:
        - vehicle_density:
            type: count
            description: Number of vehicles per km
```

---

### 9. Disease Spread with Social Network

**File**: `examples/hybrid_network_epidemic.yaml`

Epidemic on a social network (agents) with population-level tracking (SD).

```yaml
hybrid_model:
  name: Network Epidemic Model

  time:
    start: 0
    stop: 100
    dt: 0.5
    units: days

  sd_model:
    stocks:
      - name: CumulativeInfections
        initial: 10
        inflows: [new_infections]

    flows:
      - name: new_infections
        equation: AGENT_AGGREGATE("new_infection_count")

    parameters:
      - name: base_infectivity
        value: 0.3

      - name: recovery_time
        value: 10

  agent_populations:
    - name: People
      count: 1000

      network:
        type: small_world
        average_degree: 8
        rewiring_probability: 0.1

      attributes:
        - name: state
          type: enum
          values: [S, I, R]
          initial: S

        - name: infection_time
          type: float
          initial: 0

        - name: new_infection
          type: boolean
          initial: false

      initial_conditions:
        - count: 10
          state: I

      behavior:
        on_step:
          # Reset new infection flag
          - this.new_infection = false

          # Recovery
          - if this.state == I:
              this.infection_time += DT
              if this.infection_time >= recovery_time:
                this.state = R

          # Infection via network contacts
          - if this.state == S:
              infected_neighbors = COUNT(this.neighbors, neighbor.state == I)
              infection_prob = 1 - (1 - base_infectivity) ^ infected_neighbors

              if RANDOM() < infection_prob * DT:
                this.state = I
                this.infection_time = 0
                this.new_infection = true

      outputs_to_sd:
        - new_infection_count:
            type: count
            condition: agent.new_infection == true
```

---

### 10. Supply Chain with Facilities

**File**: `examples/hybrid_supply_chain.yaml`

Supply chain with aggregate flows (SD) and individual facilities (agents).

```yaml
hybrid_model:
  name: Supply Chain Network

  time:
    start: 0
    stop: 365
    dt: 1
    units: days

  sd_model:
    stocks:
      - name: TotalInventory
        initial: 0
        inflows: [aggregate_production]
        outflows: [aggregate_sales]

    flows:
      - name: aggregate_production
        equation: AGENT_AGGREGATE("production_rate")

      - name: aggregate_sales
        equation: MIN(TotalInventory, market_demand)

    auxiliaries:
      - name: market_demand
        equation: base_demand * (1 + 0.2 * SIN(TIME / 365 * 2 * PI))

    parameters:
      - name: base_demand
        value: 1000

  agent_populations:
    - name: Factories
      count: 5

      attributes:
        - name: capacity
          type: float
          initial: RANDOM_UNIFORM(100, 300)

        - name: inventory
          type: float
          initial: 50

        - name: production_rate
          type: float
          initial: 0

        - name: utilization
          type: float
          initial: 0.8

      behavior:
        on_step:
          # Produce based on capacity and utilization
          - this.production_rate = this.capacity * this.utilization

          # Update local inventory
          - this.inventory += this.production_rate * DT

          # Ship to central inventory
          - if this.inventory > this.capacity * 0.5:
              shipment = this.inventory * 0.3
              this.inventory -= shipment
              # Shipment handled by SD aggregate

          # Adjust utilization based on market demand
          - demand_signal = market_demand / AGENT_COUNT("Factories")
          - if demand_signal > this.production_rate:
              this.utilization = MIN(1.0, this.utilization + 0.05)
            else:
              this.utilization = MAX(0.5, this.utilization - 0.05)

      outputs_to_sd:
        - production_rate:
            type: sum
            field: agent.production_rate
```

---

## Advanced Features

### 11. Delays and Smoothing

**File**: `examples/delays.yaml`

Demonstrating various delay functions.

```yaml
model:
  name: Delay Functions Demo

  time:
    start: 0
    stop: 100
    dt: 0.25

  stocks:
    - name: Input
      initial: 10
      inflows: [input_flow]

  flows:
    - name: input_flow
      equation: STEP(20, 10) + PULSE(30, 5) + PULSE(50, 5)

  auxiliaries:
    - name: delay1_output
      equation: DELAY1(Input, 5)
      description: First-order exponential delay

    - name: delay3_output
      equation: DELAY3(Input, 5)
      description: Third-order delay (smoother)

    - name: delay_fixed_output
      equation: DELAY_FIXED(Input, 5)
      description: Pipeline delay (exact delay)

    - name: smooth_output
      equation: SMOOTH(Input, 10)
      description: Exponential smoothing

  parameters: []
```

---

### 12. Lookup Tables

**File**: `examples/lookups.yaml`

Non-linear relationships using table functions.

```yaml
model:
  name: Lookup Tables Demo

  time:
    start: 0
    stop: 100
    dt: 0.5

  stocks:
    - name: Resource
      initial: 100
      outflows: [consumption]

  flows:
    - name: consumption
      equation: base_consumption * consumption_multiplier

  auxiliaries:
    - name: consumption_multiplier
      equation: WITH_LOOKUP(Resource, "resource_effect_on_consumption")

  parameters:
    - name: base_consumption
      value: 5

  lookups:
    - name: resource_effect_on_consumption
      data:
        - [0, 0.1]      # Low resource → low consumption
        - [50, 0.5]
        - [100, 1.0]    # Normal
        - [200, 1.5]
        - [500, 2.0]    # High resource → high consumption
      interpolation: linear
```

---

### 13. Stochastic Simulation

**File**: `examples/stochastic.yaml`

Model with random elements.

```yaml
model:
  name: Stochastic Epidemic

  time:
    start: 0
    stop: 100
    dt: 0.25

  stocks:
    - name: Susceptible
      initial: 1000
      outflows: [infections]

    - name: Infected
      initial: 10
      inflows: [infections]
      outflows: [recoveries]

    - name: Recovered
      initial: 0
      inflows: [recoveries]

  flows:
    - name: infections
      equation: |
        # Stochastic infection rate
        base_rate = contact_rate * infectivity * Susceptible * Infected / total_pop
        noise_factor = 1 + RANDOM_NORMAL(0, 0.2)  # 20% noise
        base_rate * noise_factor

    - name: recoveries
      equation: |
        # Poisson-distributed recoveries
        expected_recoveries = Infected / recovery_time
        RANDOM_POISSON(expected_recoveries * DT) / DT

  auxiliaries:
    - name: total_pop
      equation: Susceptible + Infected + Recovered

  parameters:
    - name: contact_rate
      value: 5
    - name: infectivity
      value: 0.25
    - name: recovery_time
      value: 10
```

**Run multiple times** to see variability:
```bash
for i in {1..10}; do
  rsedsim run examples/stochastic.yaml -o results_$i.csv
done
```

---

## Real-World Applications

### 14. Climate-Economy Model

**File**: `examples/climate_economy.yaml`

Simplified integrated assessment model (like DICE/RICE).

```yaml
model:
  name: Simple Climate-Economy Model

  time:
    start: 2020
    stop: 2100
    dt: 1
    units: years

  stocks:
    - name: Capital
      initial: 100
      inflows: [investment]
      outflows: [depreciation]
      units: trillion USD

    - name: AtmosphericCO2
      initial: 850
      inflows: [emissions]
      outflows: [carbon_removal]
      units: GtC

    - name: Temperature
      initial: 1.1
      inflows: [temperature_increase]
      units: degrees C above preindustrial

  flows:
    - name: investment
      equation: savings_rate * GDP

    - name: depreciation
      equation: Capital / capital_lifetime

    - name: emissions
      equation: emissions_per_gdp * GDP * (1 - abatement_rate)

    - name: carbon_removal
      equation: (AtmosphericCO2 - preindustrial_co2) / carbon_lifetime

    - name: temperature_increase
      equation: climate_sensitivity * LN(AtmosphericCO2 / preindustrial_co2) / LN(2) / temperature_adjustment_time

  auxiliaries:
    - name: GDP
      equation: productivity * (Capital ^ capital_elasticity) * (1 - damage_fraction)

    - name: damage_fraction
      equation: damage_coefficient * (Temperature ^ 2)

    - name: abatement_rate
      equation: MIN(0.9, abatement_coefficient * TIME / 100)

  parameters:
    - name: productivity
      value: 5

    - name: capital_elasticity
      value: 0.3

    - name: savings_rate
      value: 0.2

    - name: capital_lifetime
      value: 20

    - name: emissions_per_gdp
      value: 0.5

    - name: damage_coefficient
      value: 0.0025

    - name: climate_sensitivity
      value: 3

    - name: carbon_lifetime
      value: 100

    - name: temperature_adjustment_time
      value: 50

    - name: preindustrial_co2
      value: 600

    - name: abatement_coefficient
      value: 0.1
```

**Policy experiments**:
```bash
# Business as usual
rsedsim run examples/climate_economy.yaml -o bau.csv

# Strong abatement
rsedsim run examples/climate_economy.yaml -p "abatement_coefficient=0.5" -o strong_policy.csv

# Compare outcomes
python compare_scenarios.py bau.csv strong_policy.csv
```

---

### 15. Urban Growth Model

**File**: `examples/urban_growth.yaml`

City growth with housing, jobs, and population dynamics.

```yaml
model:
  name: Urban Growth and Development

  time:
    start: 0
    stop: 50
    dt: 0.25
    units: years

  stocks:
    - name: Population
      initial: 100000
      inflows: [births, immigration]
      outflows: [deaths, emigration]

    - name: Housing
      initial: 40000
      inflows: [construction]
      outflows: [demolition]

    - name: Jobs
      initial: 50000
      inflows: [job_creation]
      outflows: [job_destruction]

  flows:
    - name: births
      equation: Population * birth_rate

    - name: deaths
      equation: Population * death_rate

    - name: immigration
      equation: attractiveness * migration_sensitivity

    - name: emigration
      equation: Population * (1 - attractiveness) * emigration_rate

    - name: construction
      equation: housing_gap / construction_time

    - name: demolition
      equation: Housing / housing_lifetime

    - name: job_creation
      equation: job_gap / job_creation_time

    - name: job_destruction
      equation: Jobs / job_lifetime

  auxiliaries:
    - name: housing_gap
      equation: MAX(0, target_housing - Housing)

    - name: target_housing
      equation: Population / occupancy_rate

    - name: job_gap
      equation: MAX(0, target_jobs - Jobs)

    - name: target_jobs
      equation: Population * labor_force_participation * employment_target

    - name: attractiveness
      equation: (Jobs / Population) * (Housing / Population) / 0.5
      description: Normalized attractiveness index

  parameters:
    - name: birth_rate
      value: 0.012

    - name: death_rate
      value: 0.008

    - name: migration_sensitivity
      value: 5000

    - name: emigration_rate
      value: 0.01

    - name: occupancy_rate
      value: 2.5

    - name: construction_time
      value: 3

    - name: housing_lifetime
      value: 50

    - name: labor_force_participation
      value: 0.65

    - name: employment_target
      value: 0.95

    - name: job_creation_time
      value: 2

    - name: job_lifetime
      value: 20
```

---

## Running the Examples

```bash
# Run any example
rsedsim run examples/sir_epidemic.yaml -o results.csv

# Run with custom parameters
rsedsim run examples/predator_prey.yaml -p "prey_birth_rate=0.7"

# Validate before running
rsedsim validate examples/climate_economy.yaml

# Run sensitivity analysis
rsedsim sensitivity examples/sir_epidemic.yaml -p contact_rate -r 1:10:20

# Monte Carlo
rsedsim monte-carlo examples/stochastic.yaml --runs 100 --parallel
```

## Creating Your Own Examples

1. Start with a simple model
2. Test with analytical solutions when possible
3. Add complexity incrementally
4. Document assumptions clearly
5. Include units
6. Test extreme conditions
7. Share with the community!

## Contributing Examples

We welcome example contributions! Please:

1. Add model file to `examples/`
2. Add description to this document
3. Include expected behavior/results
4. Test thoroughly
5. Submit a pull request

---

## Further Reading

- [Tutorial](TUTORIAL.md) - Step-by-step learning
- [API Documentation](API.md) - Programmatic usage
- [System Dynamics Society](https://systemdynamics.org/) - SD resources
- [Ventana Systems](https://vensim.com/documentation/) - Vensim docs (for SD concepts)
