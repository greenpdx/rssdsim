# Format Support Documentation

rsedsim supports multiple model file formats for maximum interoperability with other system dynamics tools.

## Supported Formats

### 1. JSON (Native Format)

**Extensions**: `.json`

**Example**:
```json
{
  "model": {
    "name": "My Model",
    "time": {
      "start": 0,
      "stop": 100,
      "dt": 0.25
    },
    "stocks": [
      {
        "name": "Population",
        "initial": 100,
        "inflows": ["births"]
      }
    ],
    "flows": [
      {
        "name": "births",
        "equation": "Population * growth_rate"
      }
    ],
    "parameters": [
      {
        "name": "growth_rate",
        "value": 0.1
      }
    ]
  }
}
```

**Usage**:
```bash
rsedsim run model.json -o results.csv
```

---

### 2. YAML (Native Format)

**Extensions**: `.yaml`, `.yml`

**Example**:
```yaml
model:
  name: My Model
  
  time:
    start: 0
    stop: 100
    dt: 0.25
  
  stocks:
    - name: Population
      initial: 100
      inflows: [births]
  
  flows:
    - name: births
      equation: Population * growth_rate
  
  parameters:
    - name: growth_rate
      value: 0.1
```

**Usage**:
```bash
rsedsim run model.yaml -o results.csv
```

---

### 3. XMILE (Industry Standard)

**Extensions**: `.xmile`, `.stmx` (Stella), `.itmx`, `.xml`

**Compatibility**: 
- Stella/iThink
- Vensim (with XMILE export)
- AnyLogic SD components
- Other XMILE v1.0 compliant tools

**Example**:
```xml
<?xml version="1.0" encoding="UTF-8"?>
<xmile version="1.0" xmlns="http://docs.oasis-open.org/xmile/ns/XMILE/v1.0">
    <header>
        <name>My Model</name>
    </header>
    
    <sim_specs>
        <start>0</start>
        <stop>100</stop>
        <dt>0.25</dt>
    </sim_specs>
    
    <model>
        <variables>
            <stock name="Population">
                <eqn>100</eqn>
                <inflow>births</inflow>
            </stock>
            
            <flow name="births">
                <eqn>Population * growth_rate</eqn>
            </flow>
            
            <aux name="growth_rate">
                <eqn>0.1</eqn>
            </aux>
        </variables>
    </model>
</xmile>
```

**Usage**:
```bash
# Run XMILE file directly
rsedsim run model.xmile -o results.csv

# Run Stella file
rsedsim run model.stmx -o results.csv
```

**Supported XMILE Features**:
- ✅ Stocks with initial values
- ✅ Flows (inflows/outflows)
- ✅ Auxiliaries (converters)
- ✅ Simulation specs (start, stop, dt)
- ✅ Units metadata
- ⚠️ Graphical functions (partially - treated as equations)
- ❌ Arrays/subscripts (not yet implemented)
- ❌ Modules (not yet implemented)

---

### 4. InsightMaker Format

**Extensions**: `.json` (auto-detected)

**Compatibility**: InsightMaker (online SD modeling tool)

**Example**:
```json
{
  "name": "My Model",
  "settings": {
    "start": 0,
    "stop": 100,
    "dt": 0.25
  },
  "primitives": [
    {
      "id": "s1",
      "type": "Stock",
      "name": "Population",
      "value": "100",
      "inflows": ["f1"]
    },
    {
      "id": "f1",
      "type": "Flow",
      "name": "births",
      "equation": "Population * growth_rate"
    },
    {
      "id": "p1",
      "type": "Parameter",
      "name": "growth_rate",
      "value": "0.1"
    }
  ]
}
```

**Usage**:
```bash
# Auto-detected as InsightMaker format
rsedsim run insightmaker_model.json -o results.csv
```

**Supported Primitive Types**:
- ✅ Stock
- ✅ Flow
- ✅ Converter (auxiliary)
- ✅ Parameter
- ✅ Variable (treated as auxiliary or parameter)
- ❌ Agent-based primitives (not yet implemented)
- ❌ Display elements (ignored)

---

## Format Detection

rsedsim automatically detects the format based on file extension:

| Extension | Format |
|-----------|--------|
| `.json` | JSON (tries InsightMaker first, then native) |
| `.yaml`, `.yml` | YAML |
| `.xmile` | XMILE |
| `.stmx` | XMILE (Stella) |
| `.itmx` | XMILE |
| `.xml` | XMILE |

## Converting Between Formats

You can use rsedsim to convert models:

```bash
# XMILE to JSON
rsedsim run model.xmile --dry-run | jq . > model.json

# InsightMaker to YAML
# (Future feature - not yet implemented)
rsedsim convert model.json -f yaml -o model.yaml
```

## Import from Other Tools

### From Stella/iThink

1. In Stella, go to **File > Export > XMILE**
2. Save as `.stmx` or `.xmile`
3. Run with rsedsim:
   ```bash
   rsedsim run model.stmx -o results.csv
   ```

### From Vensim

1. In Vensim, go to **File > Export > XMILE**
2. Save as `.xmile`
3. Run with rsedsim:
   ```bash
   rsedsim run model.xmile -o results.csv
   ```

### From InsightMaker

1. In InsightMaker, use **Export > JSON**
2. Save the JSON file
3. Run with rsedsim:
   ```bash
   rsedsim run insightmaker_export.json -o results.csv
   ```

### From PySD

PySD doesn't have a standard export format, but you can:
1. Export to XMILE from the original tool (Vensim/Stella)
2. Or manually create JSON/YAML from the PySD model structure

## Validation

Always validate imported models:

```bash
rsedsim validate model.xmile
```

This checks:
- ✅ All referenced flows exist
- ✅ Stock connections are valid
- ✅ No circular dependencies
- ✅ Equation syntax

## Limitations

### Current Limitations

1. **Arrays/Subscripts**: Not yet supported in any format
2. **Lookup Tables**: Parsed but not fully functional
3. **Delay Functions**: Parsed but implementation incomplete
4. **Modules/Submodels**: Not supported
5. **Graphical Functions**: Limited support

### Workarounds

For unsupported features:
- **Arrays**: Create separate variables for each element
- **Lookup Tables**: Use IF_THEN_ELSE chains
- **Delays**: Use explicit auxiliary variables
- **Modules**: Flatten into single model

## Examples

All format examples are in the `examples/` directory:

```
examples/
├── *.json           # Native JSON format
├── *.yaml           # Native YAML format
├── xmile/           # XMILE examples
│   ├── simple_growth.xmile
│   └── sir_epidemic.xmile
└── insightmaker/    # InsightMaker format
    ├── simple_growth.json
    └── sir_epidemic.json
```

## Future Format Support

Planned additions:
- [ ] Vensim `.mdl` format (native)
- [ ] Modelica (partial support)
- [ ] NetLogo (for hybrid models)
- [ ] Excel/CSV-based model definitions
