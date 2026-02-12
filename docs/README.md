# RSSDSIM Codebase Analysis - Complete Documentation

This folder contains a comprehensive analysis of the **rssdsim** (Rust System Dynamics Simulator) codebase.

## Documents Included

### 1. **EXPLORATION_SUMMARY.txt** (This file)
Quick executive summary of findings with:
- Project overview
- Key findings organized by topic
- File locations
- Lines of code statistics
- Quick start guide
- Future development roadmap

### 2. **rssdsim_codebase_analysis.md** (MAIN DOCUMENT - 700+ lines)
The most comprehensive reference document covering:

1. **Overall Project Structure** - Directory layout and key technologies
2. **Integration Methods Implementation** - Details on Euler and RK4 integrators
3. **Function/Builtin Support** - All 9 built-in functions documented
4. **Array/Subset Implementation** - Current status (not implemented)
5. **Key Data Structures** - Model, Stock, Flow, Auxiliary, Parameter, Expression definitions
6. **Expression Evaluation Flow** - Parsing and evaluation process
7. **Simulation Execution Flow** - Complete workflow from load to output
8. **Derivative Computation** - How stocks are updated
9. **Input/Output Formats** - JSON, YAML, XMILE, InsightMaker support
10. **CLI Interface** - Command-line usage
11. **Testing** - Test coverage and how to run tests
12. **Architecture Patterns** - Design patterns used
13. **Current Limitations** - Known constraints
14. **Planned Features** - Future roadmap
15. **Example Walkthrough** - Simple growth model execution
16. **Key Files Summary** - Quick reference table
17. **Comparison: Euler vs RK4** - Integration method comparison

### 3. **rssdsim_code_snippets.md** (QUICK REFERENCE)
Exact source code excerpts from the 10 key implementation sections:

1. Integrator Trait Definition
2. Euler Integration Complete Algorithm
3. RK4 Integration Core Loop
4. Expression Parser Structure
5. Expression Evaluation Core
6. Simulation State Initialization
7. Simulation Engine Run Loop
8. Variable Resolution Priority
9. CLI Command Parsing
10. Model Structure Definitions

### 4. **ARCHITECTURE_DIAGRAMS.txt** (VISUAL REFERENCE)
10 detailed ASCII diagrams showing:

1. Module Dependency Diagram
2. Expression Parsing Pipeline
3. Simulation Execution Flow
4. Integration Step Detail (Euler)
5. RK4 Stages (Runge-Kutta 4th Order)
6. Data Flow in Simulation State
7. Class Diagram (Key Types)
8. Function Call Resolution
9. Variable Resolution Stack
10. Auxiliary Convergence (Fixed-Point Iteration)

## Quick Navigation

### By Topic

**Integration Methods**
- See: rssdsim_codebase_analysis.md Section 2
- See: ARCHITECTURE_DIAGRAMS.txt Sections 4-5
- See: rssdsim_code_snippets.md Sections 2-3

**Expression Parsing & Evaluation**
- See: rssdsim_codebase_analysis.md Section 6
- See: ARCHITECTURE_DIAGRAMS.txt Sections 2, 8-9
- See: rssdsim_code_snippets.md Sections 4-5

**Data Structures**
- See: rssdsim_codebase_analysis.md Section 5
- See: ARCHITECTURE_DIAGRAMS.txt Section 7
- See: rssdsim_code_snippets.md Sections 6, 10

**Simulation Flow**
- See: rssdsim_codebase_analysis.md Sections 7-8
- See: ARCHITECTURE_DIAGRAMS.txt Sections 3-6
- See: rssdsim_code_snippets.md Section 7

**Built-in Functions**
- See: rssdsim_codebase_analysis.md Section 3
- See: ARCHITECTURE_DIAGRAMS.txt Section 8
- See: rssdsim_code_snippets.md Section 5

**I/O and File Formats**
- See: rssdsim_codebase_analysis.md Section 9
- See: rssdsim_code_snippets.md Section 10

**CLI Usage**
- See: rssdsim_codebase_analysis.md Section 10
- See: rssdsim_code_snippets.md Section 9

### By File Location

**`/home/svvs/rssdsim/src/simulation/integrator.rs`**
- Euler and RK4 implementation
- See: rssdsim_codebase_analysis.md Section 2
- See: rssdsim_code_snippets.md Sections 2-3
- See: ARCHITECTURE_DIAGRAMS.txt Sections 4-5

**`/home/svvs/rssdsim/src/model/expression.rs`**
- Expression parser and evaluator
- See: rssdsim_codebase_analysis.md Section 6
- See: rssdsim_code_snippets.md Sections 4-5
- See: ARCHITECTURE_DIAGRAMS.txt Sections 2, 8-9

**`/home/svvs/rssdsim/src/model/mod.rs`**
- Model data structures
- See: rssdsim_codebase_analysis.md Section 5
- See: rssdsim_code_snippets.md Sections 8, 10

**`/home/svvs/rssdsim/src/simulation/mod.rs`**
- SimulationState, SimulationConfig
- See: rssdsim_codebase_analysis.md Section 5
- See: rssdsim_code_snippets.md Section 6

**`/home/svvs/rssdsim/src/simulation/engine.rs`**
- Simulation orchestration
- See: rssdsim_codebase_analysis.md Section 7
- See: rssdsim_code_snippets.md Section 7

**`/home/svvs/rssdsim/src/main.rs`**
- CLI interface
- See: rssdsim_codebase_analysis.md Section 10
- See: rssdsim_code_snippets.md Section 9

**`/home/svvs/rssdsim/src/io/`**
- Model loading and results output
- See: rssdsim_codebase_analysis.md Section 9

## Key Findings Summary

### What's Implemented
- Stock-flow dynamics modeling
- Two integration methods: Euler (1st order), RK4 (4th order)
- Expression parser with 9 built-in functions
- Multiple input formats: JSON, YAML, XMILE, InsightMaker
- CSV output
- Fixed-point iteration for auxiliary variables
- Variable constraints (non-negative, max value)
- CLI interface with parameter override

### What's Not Implemented
- Multi-dimensional arrays/vectors
- Delay functions
- Lookup tables
- Random number generation
- MCP and A2A protocols (stubs only)

### Architecture Strengths
- Clean trait-based design (Integrator trait)
- Recursive descent parsing with proper precedence
- Well-organized module structure
- Good separation of concerns
- Production-quality error handling

## Project Statistics

- **Language**: Rust (Edition 2024)
- **Total Lines**: ~2500 (source code)
- **Main Modules**: 4 (model, simulation, io, protocol)
- **Files**: 15+ source files
- **Built-in Functions**: 9
- **Integration Methods**: 2
- **Input Formats**: 4
- **Output Format**: 1 (CSV)

## How to Use This Documentation

1. **Start here**: Read EXPLORATION_SUMMARY.txt for overview
2. **Deep dive**: Read rssdsim_codebase_analysis.md for comprehensive details
3. **Quick reference**: Use rssdsim_code_snippets.md for exact code
4. **Visual understanding**: Consult ARCHITECTURE_DIAGRAMS.txt for workflows

## For Development

If you're modifying the codebase:

1. **Adding a new integration method**: See Section 2 & rssdsim_code_snippets.md Section 1
2. **Adding new functions**: See Section 3 & rssdsim_code_snippets.md Section 5
3. **Extending expression syntax**: See Section 6 & rssdsim_code_snippets.md Section 4
4. **Understanding the workflow**: See ARCHITECTURE_DIAGRAMS.txt Section 3
5. **Debugging integrators**: See ARCHITECTURE_DIAGRAMS.txt Sections 4-5

## Quick Commands

```bash
# Build the project
cargo build --release

# Run a simulation
cargo run -- run examples/exponential_growth.yaml

# Run tests
cargo test

# Show help
cargo run -- info
```

## Directory Structure

```
/home/svvs/rssdsim/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── model/               # SD model definitions
│   ├── simulation/          # Integration and execution
│   ├── io/                  # File I/O and parsing
│   └── protocol/            # External protocols (stubs)
├── examples/                # Example models
├── Cargo.toml              # Dependencies
└── target/                 # Build artifacts
```

## Contact & Contributing

See repository at: https://github.com/yourusername/rsedsim

---

**Documentation Generated**: February 2026  
**Codebase Version**: 0.1.0  
**Analysis Scope**: Complete codebase exploration
