# Protocol Integration Guide

This document describes how to integrate rsedsim with external systems using the MCP and A2A protocols.

## Table of Contents

1. [MCP (Model Context Protocol)](#mcp-model-context-protocol)
2. [A2A (Agent-to-Agent Protocol)](#a2a-agent-to-agent-protocol)
3. [Use Cases](#use-cases)
4. [Integration Examples](#integration-examples)

---

## MCP (Model Context Protocol)

### Overview

The Model Context Protocol enables AI agents and Large Language Models to interact with rsedsim simulations through a standardized interface. This allows LLMs to:

- Run simulations programmatically
- Query model structure and state
- Perform sensitivity analysis
- Extract and analyze results

### Protocol Specification

MCP follows the specification at https://modelcontextprotocol.io/

**Version**: 2024-11-05

### Server Capabilities

rsedsim exposes the following MCP capabilities:

#### Resources

Resources provide read-only access to simulation data:

| URI | Description | MIME Type |
|-----|-------------|-----------|
| `rsedsim://models/list` | List all loaded models | `application/json` |
| `rsedsim://simulation/state` | Current simulation state | `application/json` |
| `rsedsim://results/latest` | Latest simulation results | `application/json` |
| `rsedsim://model/{id}/structure` | Model structure (stocks, flows) | `application/json` |
| `rsedsim://model/{id}/dependencies` | Dependency graph | `application/json` |

#### Tools

Tools enable active operations on simulations:

##### 1. `run_simulation`

Execute a simulation with specified parameters.

**Input Schema**:
```json
{
  "type": "object",
  "properties": {
    "model": {
      "type": "string",
      "description": "Model file path or URI"
    },
    "parameters": {
      "type": "object",
      "description": "Parameter overrides (key-value pairs)"
    },
    "time_config": {
      "type": "object",
      "properties": {
        "start": {"type": "number"},
        "stop": {"type": "number"},
        "dt": {"type": "number"}
      }
    }
  },
  "required": ["model"]
}
```

**Example**:
```json
{
  "name": "run_simulation",
  "arguments": {
    "model": "sir_model.json",
    "parameters": {
      "contact_rate": 10,
      "infectivity": 0.3
    },
    "time_config": {
      "start": 0,
      "stop": 100,
      "dt": 0.25
    }
  }
}
```

**Output**:
```json
{
  "simulation_id": "uuid-1234",
  "status": "completed",
  "final_time": 100,
  "results_uri": "rsedsim://results/uuid-1234"
}
```

##### 2. `analyze_model`

Analyze model structure and behavior.

**Analysis Types**:
- `structure`: List stocks, flows, loops
- `loops`: Identify feedback loops and dominance
- `sensitivity`: Parameter sensitivity analysis
- `equilibrium`: Find equilibrium points

**Example**:
```json
{
  "name": "analyze_model",
  "arguments": {
    "model": "sir_model.json",
    "analysis_type": "loops"
  }
}
```

**Output**:
```json
{
  "loops": [
    {
      "type": "reinforcing",
      "polarity": "+",
      "elements": ["Susceptible", "infection_rate", "Infected"]
    },
    {
      "type": "balancing",
      "polarity": "-",
      "elements": ["Infected", "recovery_rate", "Recovered"]
    }
  ]
}
```

##### 3. `sensitivity_analysis`

Perform parameter sensitivity analysis.

**Example**:
```json
{
  "name": "sensitivity_analysis",
  "arguments": {
    "model": "sir_model.json",
    "parameters": ["contact_rate", "infectivity"],
    "ranges": {
      "contact_rate": {"min": 1, "max": 10},
      "infectivity": {"min": 0.1, "max": 0.5}
    },
    "samples": 100,
    "method": "latin_hypercube"
  }
}
```

**Output**:
```json
{
  "sensitivity_indices": {
    "contact_rate": {
      "first_order": 0.65,
      "total": 0.72
    },
    "infectivity": {
      "first_order": 0.28,
      "total": 0.31
    }
  },
  "results_uri": "rsedsim://results/sensitivity/uuid-5678"
}
```

##### 4. `get_variable_timeseries`

Extract time series data for variables.

**Example**:
```json
{
  "name": "get_variable_timeseries",
  "arguments": {
    "simulation_id": "uuid-1234",
    "variables": ["Susceptible", "Infected", "Recovered"]
  }
}
```

**Output**:
```json
{
  "time": [0, 0.25, 0.5, ...],
  "variables": {
    "Susceptible": [990, 985.3, 978.2, ...],
    "Infected": [10, 14.2, 20.8, ...],
    "Recovered": [0, 0.5, 1.0, ...]
  }
}
```

### Starting the MCP Server

#### Stdio Transport (for local CLI tools)

```bash
rsedsim mcp serve --stdio
```

This starts an MCP server that communicates via stdin/stdout using JSON-RPC.

**Client Connection Example** (Python):
```python
import json
import subprocess

# Start rsedsim MCP server
proc = subprocess.Popen(
    ["rsedsim", "mcp", "serve", "--stdio"],
    stdin=subprocess.PIPE,
    stdout=subprocess.PIPE,
    text=True
)

# Send initialize request
request = {
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {
        "protocolVersion": "2024-11-05",
        "capabilities": {},
        "clientInfo": {
            "name": "my-client",
            "version": "1.0.0"
        }
    }
}

proc.stdin.write(json.dumps(request) + "\n")
proc.stdin.flush()

# Read response
response = json.loads(proc.stdout.readline())
print(response)
```

#### HTTP Transport (for web applications)

```bash
rsedsim mcp serve --http localhost:3000
```

This starts an HTTP server with Server-Sent Events (SSE) for real-time updates.

**Client Connection Example** (JavaScript):
```javascript
const evtSource = new EventSource("http://localhost:3000/sse");

evtSource.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log("MCP message:", data);
};

// Call a tool
fetch("http://localhost:3000/call-tool", {
  method: "POST",
  headers: { "Content-Type": "application/json" },
  body: JSON.stringify({
    name: "run_simulation",
    arguments: {
      model: "sir_model.json",
      parameters: { contact_rate: 10 }
    }
  })
})
.then(response => response.json())
.then(data => console.log("Result:", data));
```

### MCP Message Flow

```
Client                                    rsedsim MCP Server
  │                                              │
  ├──────── Initialize ────────────────────────→ │
  │                                              │
  │ ←────── Initialize Response ────────────────┤
  │                                              │
  ├──────── List Tools ─────────────────────────→ │
  │                                              │
  │ ←────── Tools List ──────────────────────────┤
  │         (run_simulation, analyze_model, ...) │
  │                                              │
  ├──────── Call Tool: run_simulation ─────────→ │
  │         {model: "sir.json", ...}             │
  │                                              ├─── Load model
  │                                              ├─── Run simulation
  │                                              ├─── Save results
  │ ←────── Tool Result ─────────────────────────┤
  │         {simulation_id: "...", ...}          │
  │                                              │
  ├──────── Call Tool: get_variable_timeseries ─→│
  │         {simulation_id: "...", ...}          │
  │                                              │
  │ ←────── Tool Result ─────────────────────────┤
  │         {time: [...], variables: {...}}      │
```

---

## A2A (Agent-to-Agent Protocol)

### Overview

The Agent-to-Agent protocol enables distributed agent communication in hybrid SD-ABM models. Agents can:

- Discover other agents dynamically
- Send direct messages
- Publish/subscribe to topics
- Synchronize state across network boundaries
- Coordinate distributed simulations

### Protocol Design

A2A is a lightweight, asynchronous messaging protocol designed for:
- **Low latency**: UDP transport for fast message delivery
- **Scalability**: Thousands of agents across multiple nodes
- **Flexibility**: Pluggable transport layer
- **Resilience**: Tolerates message loss (eventual consistency)

### Message Structure

```rust
pub struct A2aMessage {
    pub message_id: String,       // Unique ID for tracking
    pub from: AgentId,            // Source agent
    pub to: Option<AgentId>,      // Destination (None = broadcast)
    pub timestamp: u64,           // Unix timestamp (ms)
    pub ttl: Option<u32>,         // Time-to-live (hops or seconds)
    pub payload: A2aPayload,      // Message content
}

pub struct AgentId {
    pub namespace: String,  // e.g., "simulation1"
    pub id: String,         // e.g., "agent_42"
}
```

### Message Types

#### 1. Registration & Discovery

**Register Agent**:
```json
{
  "type": "Register",
  "data": {
    "agent_info": {
      "id": {"namespace": "sim1", "id": "agent_1"},
      "agent_type": "Person",
      "capabilities": ["mobile", "infectious"],
      "attributes": {
        "age": 25,
        "location": [37.7749, -122.4194]
      },
      "endpoint": "udp://192.168.1.10:5000"
    }
  }
}
```

**Discover Agents**:
```json
{
  "type": "Discover",
  "data": {
    "query": {
      "agent_type": "Person",
      "capabilities": ["mobile"],
      "namespace": "sim1"
    }
  }
}
```

**Discovery Result**:
```json
{
  "type": "DiscoveryResult",
  "data": {
    "agents": [
      {
        "id": {"namespace": "sim1", "id": "agent_1"},
        "agent_type": "Person",
        "capabilities": ["mobile", "infectious"],
        ...
      }
    ]
  }
}
```

#### 2. Direct Messaging

```json
{
  "message_id": "msg_12345",
  "from": {"namespace": "sim1", "id": "agent_1"},
  "to": {"namespace": "sim1", "id": "agent_2"},
  "timestamp": 1234567890000,
  "ttl": 10,
  "payload": {
    "type": "DirectMessage",
    "data": {
      "content": {
        "message": "hello",
        "data": {"value": 42}
      }
    }
  }
}
```

#### 3. Publish/Subscribe

**Subscribe to Topic**:
```json
{
  "type": "Subscribe",
  "data": {
    "topic": "infection_events"
  }
}
```

**Publish to Topic**:
```json
{
  "type": "Publish",
  "data": {
    "topic": "infection_events",
    "content": {
      "agent_id": "agent_42",
      "time": 1234567890,
      "location": [37.7749, -122.4194]
    }
  }
}
```

#### 4. State Synchronization

**Request State**:
```json
{
  "type": "StateRequest",
  "data": {
    "keys": ["position", "velocity", "health"]
  }
}
```

**State Response**:
```json
{
  "type": "StateResponse",
  "data": {
    "state": {
      "position": [100.5, 200.3],
      "velocity": [1.2, -0.5],
      "health": 95
    }
  }
}
```

#### 5. Simulation Control

**Start Simulation**:
```json
{
  "type": "SimControl",
  "data": {
    "command": {
      "cmd": "Start",
      "time_config": {
        "start": 0,
        "stop": 100,
        "dt": 0.1
      }
    }
  }
}
```

**Synchronization Barrier**:
```json
{
  "type": "SimControl",
  "data": {
    "command": {
      "cmd": "Barrier",
      "barrier_id": "step_100",
      "required_agents": [
        {"namespace": "sim1", "id": "agent_1"},
        {"namespace": "sim1", "id": "agent_2"}
      ]
    }
  }
}
```

### Transport Layers

#### UDP Transport (Default)

Fast, connectionless, suitable for high-frequency updates.

```rust
let mut transport = UdpTransport::new();
transport.bind("0.0.0.0:5000").await?;
transport.add_peer("192.168.1.11:5000".parse()?).await;

let node = A2aNode::new(agent_id)
    .with_transport(Box::new(transport));
```

**Configuration in Model**:
```yaml
a2a_config:
  transport: udp
  bind_addr: "0.0.0.0:5000"
  peers:
    - "192.168.1.10:5000"
    - "192.168.1.11:5000"
    - "192.168.1.12:5000"
  multicast_group: "239.255.0.1:5000"  # Optional multicast
```

#### TCP Transport

Reliable, ordered delivery for critical messages.

```yaml
a2a_config:
  transport: tcp
  bind_addr: "0.0.0.0:5001"
  peers:
    - "192.168.1.10:5001"
```

#### WebSocket Transport

For browser-based agents.

```yaml
a2a_config:
  transport: websocket
  url: "wss://simulation-server.com/a2a"
```

### Using A2A in Hybrid Models

#### Example: Distributed Epidemic Model

```yaml
hybrid_model:
  name: "Distributed Epidemic"

  # A2A Configuration
  a2a_config:
    node_id: "sim1:population_node_1"
    transport: udp
    bind_addr: "0.0.0.0:5000"
    peers:
      - "192.168.1.10:5000"  # Node 2
      - "192.168.1.11:5000"  # Node 3

  # SD Components
  sd_model:
    stocks:
      - name: TotalInfected
        initial: 0
        inflows: [new_infections]

    flows:
      - name: new_infections
        equation: AGENT_AGGREGATE("infection_count")

  # Agent Population
  agent_populations:
    - name: People
      count: 10000
      initial_distribution: uniform

      attributes:
        - name: infected
          initial: false
        - name: location
          initial: RANDOM_LOCATION()
        - name: contact_radius
          initial: RANDOM_NORMAL(10, 2)

      behavior:
        on_step:
          # Discover nearby agents (across network)
          - nearby = A2A_DISCOVER({
              distance_from: this.location,
              max_distance: this.contact_radius
            })

          # Check for infection
          - if this.susceptible:
              infected_nearby = COUNT(nearby, agent.infected == true)
              infection_prob = infectivity * infected_nearby / total_population
              if RANDOM() < infection_prob:
                this.infected = true
                A2A_PUBLISH("infection_events", {
                  agent_id: this.id,
                  location: this.location,
                  time: TIME()
                })

        on_message:
          # React to published infection events
          - if message.topic == "infection_events":
              log("Nearby infection at", message.content.location)

      # Aggregations to SD
      outputs_to_sd:
        - infection_count: COUNT(agent.infected == true)
        - average_contacts: MEAN(agent.contact_count)
```

### A2A Node Lifecycle

```
1. Initialize
   ├─→ Create A2aNode with agent_id
   ├─→ Configure transport
   └─→ Register message handlers

2. Connect
   ├─→ Bind to local address
   ├─→ Connect to peers
   └─→ Register with directory (if using centralized discovery)

3. Run
   ├─→ Start message processing loop
   ├─→ Handle incoming messages
   │    ├─→ Update local registry
   │    ├─→ Route to handlers
   │    └─→ Forward if necessary
   └─→ Send outgoing messages

4. Shutdown
   ├─→ Unregister from directory
   ├─→ Close transport
   └─→ Clean up resources
```

---

## Use Cases

### 1. LLM-Driven Simulation (MCP)

**Scenario**: AI agent explores policy interventions for epidemic control.

```python
# AI agent using MCP to run simulations

# 1. Load base model
response = mcp_client.call_tool("run_simulation", {
    "model": "epidemic.json"
})
baseline_results = response["results_uri"]

# 2. Explore interventions
interventions = [
    {"mask_mandate": True, "vaccination_rate": 0.5},
    {"mask_mandate": False, "vaccination_rate": 0.8},
    {"lockdown": True, "school_closure": True},
]

for intervention in interventions:
    result = mcp_client.call_tool("run_simulation", {
        "model": "epidemic.json",
        "parameters": intervention
    })

    # 3. Analyze results
    analysis = mcp_client.call_tool("analyze_model", {
        "model": result["simulation_id"],
        "analysis_type": "sensitivity"
    })

    # AI decides best intervention
    if analysis["peak_infections"] < best_peak:
        best_intervention = intervention
```

### 2. Distributed Multi-Region Simulation (A2A)

**Scenario**: Epidemic model with agents distributed across regions (different servers).

```
┌──────────────────┐       ┌──────────────────┐       ┌──────────────────┐
│   Region North   │       │   Region South   │       │   Region East    │
│ Server 192.168.1.10│     │ Server 192.168.1.11│    │ Server 192.168.1.12│
│                  │       │                  │       │                  │
│ 10,000 agents    │◄─────►│ 10,000 agents    │◄─────►│ 10,000 agents    │
│ A2A Node:5000    │  UDP  │ A2A Node:5000    │  UDP  │ A2A Node:5000    │
└──────────────────┘       └──────────────────┘       └──────────────────┘
         │                          │                          │
         └──────────────────────────┴──────────────────────────┘
                        Cross-region travel
                        (A2A DirectMessage)
```

**Travel Between Regions**:
```rust
// Agent in North region wants to travel to South
let message = A2aMessage {
    from: AgentId::new("north", "agent_123"),
    to: Some(AgentId::new("south", "population_manager")),
    payload: A2aPayload::DirectMessage {
        content: json!({
            "action": "migrate_agent",
            "agent_data": {
                "attributes": {...},
                "state": {...}
            }
        })
    },
    ...
};

a2a_node.send(message).await?;
```

### 3. Real-Time Dashboard (MCP + WebSocket)

**Scenario**: Web dashboard showing live simulation updates.

```javascript
// Connect to MCP server via HTTP
const mcp = new McpClient("http://localhost:3000");

// Start simulation
const sim = await mcp.callTool("run_simulation", {
  model: "epidemic.json"
});

// Subscribe to updates (SSE)
const eventSource = new EventSource(`http://localhost:3000/simulation/${sim.simulation_id}/stream`);

eventSource.onmessage = (event) => {
  const data = JSON.parse(event.data);

  // Update charts
  chart.update({
    time: data.time,
    infected: data.variables.Infected,
    susceptible: data.variables.Susceptible
  });
};
```

---

## Integration Examples

### Claude Desktop + MCP

Add to Claude Desktop config (`claude_desktop_config.json`):

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

Now Claude can run simulations:

> **User**: "Run the SIR model with a contact rate of 10 and show me the peak infection time"
>
> **Claude**: *Calls `run_simulation` tool* → *Calls `get_variable_timeseries` tool* → "The peak infection occurs at day 23.5 with 342 infected individuals."

### Multi-Agent Simulation with A2A

```rust
// Initialize 3 nodes on different machines

// Machine 1
let node1 = A2aNode::new(AgentId::new("sim1", "node1"))
    .with_transport(UdpTransport::bind("0.0.0.0:5000")?);
node1.add_peer("192.168.1.11:5000").await;
node1.add_peer("192.168.1.12:5000").await;

// Machine 2
let node2 = A2aNode::new(AgentId::new("sim1", "node2"))
    .with_transport(UdpTransport::bind("0.0.0.0:5000")?);
node2.add_peer("192.168.1.10:5000").await;
node2.add_peer("192.168.1.12:5000").await;

// Machine 3
let node3 = A2aNode::new(AgentId::new("sim1", "node3"))
    .with_transport(UdpTransport::bind("0.0.0.0:5000")?);
node3.add_peer("192.168.1.10:5000").await;
node3.add_peer("192.168.1.11:5000").await;

// Each node runs agents locally, communicates via A2A
tokio::spawn(async move { node1.run().await });
tokio::spawn(async move { node2.run().await });
tokio::spawn(async move { node3.run().await });
```

---

## Security Considerations

### MCP

- **Authentication**: Implement OAuth2 or API keys for HTTP transport
- **Authorization**: Role-based access control for tools/resources
- **Validation**: Sanitize all inputs to prevent injection attacks
- **Rate Limiting**: Prevent DoS via excessive tool calls

### A2A

- **Message Signing**: HMAC or digital signatures for message integrity
- **Encryption**: TLS for TCP/WebSocket, DTLS for UDP
- **Agent Verification**: Certificate-based agent identity
- **Network Isolation**: Firewall rules to restrict A2A traffic

---

## Performance Tuning

### MCP

- Use stdio transport for local connections (lower latency)
- Batch multiple tool calls when possible
- Cache resource reads (model structure doesn't change)
- Stream large results instead of buffering

### A2A

- Use UDP for high-frequency, loss-tolerant messages
- Implement message batching (send multiple messages per packet)
- Use multicast for broadcast messages
- Tune TTL to prevent message storms
- Consider reliable multicast (PGM) for critical messages

---

## Troubleshooting

### MCP Issues

**Problem**: Client can't connect to stdio server

- Check that `rsedsim mcp serve --stdio` is running
- Verify JSON-RPC format of messages
- Enable debug logging: `RUST_LOG=debug rsedsim mcp serve --stdio`

**Problem**: Tool call returns error

- Check tool input schema
- Validate parameter types
- Review logs: `rsedsim mcp serve --http localhost:3000 --log-level debug`

### A2A Issues

**Problem**: Agents can't discover each other

- Verify network connectivity (`ping`, `nc -u`)
- Check firewall rules (allow UDP port)
- Ensure peers are configured correctly
- Use centralized directory for discovery

**Problem**: High message loss

- Switch from UDP to TCP
- Reduce message frequency
- Implement retry logic
- Check network bandwidth

---

## Further Reading

- [MCP Specification](https://modelcontextprotocol.io/)
- [System Dynamics Society](https://systemdynamics.org/)
- [NetLogo Hubnet Protocol](https://ccl.northwestern.edu/netlogo/docs/hubnet.html) (inspiration for A2A)
