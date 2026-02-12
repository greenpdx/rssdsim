/// Multi-dimensional array value support for simulation state

use std::collections::HashMap;

/// A value that can be either scalar or multi-dimensional array
#[derive(Debug, Clone)]
pub enum ArrayValue {
    /// Scalar value
    Scalar(f64),
    /// Multi-dimensional array stored as flat vector with shape information
    /// shape: dimensions of the array (e.g., [3, 4] for 3x4 matrix)
    /// data: flat array in row-major order
    Array { shape: Vec<usize>, data: Vec<f64> },
}

impl ArrayValue {
    /// Create a scalar value
    pub fn scalar(value: f64) -> Self {
        ArrayValue::Scalar(value)
    }

    /// Create a multi-dimensional array with given shape, initialized to zero
    pub fn zeros(shape: Vec<usize>) -> Self {
        let size: usize = shape.iter().product();
        ArrayValue::Array {
            shape,
            data: vec![0.0; size],
        }
    }

    /// Create a multi-dimensional array with given shape and data
    pub fn from_vec(shape: Vec<usize>, data: Vec<f64>) -> Result<Self, String> {
        let expected_size: usize = shape.iter().product();
        if data.len() != expected_size {
            return Err(format!(
                "Data size {} does not match shape {:?} (expected {})",
                data.len(),
                shape,
                expected_size
            ));
        }
        Ok(ArrayValue::Array { shape, data })
    }

    /// Get scalar value (error if array)
    pub fn as_scalar(&self) -> Result<f64, String> {
        match self {
            ArrayValue::Scalar(v) => Ok(*v),
            ArrayValue::Array { .. } => Err("Cannot convert array to scalar".to_string()),
        }
    }

    /// Get value at specific indices (works for both scalar and array)
    pub fn get(&self, indices: &[usize]) -> Result<f64, String> {
        match self {
            ArrayValue::Scalar(v) => {
                if indices.is_empty() {
                    Ok(*v)
                } else {
                    Err("Cannot index scalar value".to_string())
                }
            }
            ArrayValue::Array { shape, data } => {
                if indices.len() != shape.len() {
                    return Err(format!(
                        "Expected {} indices, got {}",
                        shape.len(),
                        indices.len()
                    ));
                }

                // Check bounds
                for (i, &idx) in indices.iter().enumerate() {
                    if idx >= shape[i] {
                        return Err(format!(
                            "Index {} out of bounds for dimension {} (size {})",
                            idx, i, shape[i]
                        ));
                    }
                }

                // Convert multi-dimensional index to flat index
                let flat_idx = Self::indices_to_flat(indices, shape);
                Ok(data[flat_idx])
            }
        }
    }

    /// Set value at specific indices
    pub fn set(&mut self, indices: &[usize], value: f64) -> Result<(), String> {
        match self {
            ArrayValue::Scalar(v) => {
                if indices.is_empty() {
                    *v = value;
                    Ok(())
                } else {
                    Err("Cannot index scalar value".to_string())
                }
            }
            ArrayValue::Array { shape, data } => {
                if indices.len() != shape.len() {
                    return Err(format!(
                        "Expected {} indices, got {}",
                        shape.len(),
                        indices.len()
                    ));
                }

                // Check bounds
                for (i, &idx) in indices.iter().enumerate() {
                    if idx >= shape[i] {
                        return Err(format!(
                            "Index {} out of bounds for dimension {} (size {})",
                            idx, i, shape[i]
                        ));
                    }
                }

                let flat_idx = Self::indices_to_flat(indices, shape);
                data[flat_idx] = value;
                Ok(())
            }
        }
    }

    /// Convert multi-dimensional indices to flat index (row-major order)
    fn indices_to_flat(indices: &[usize], shape: &[usize]) -> usize {
        let mut flat_index = 0;
        let mut multiplier = 1;

        for i in (0..indices.len()).rev() {
            flat_index += indices[i] * multiplier;
            if i > 0 {
                multiplier *= shape[i];
            }
        }

        flat_index
    }

    /// Get the shape of this value (empty for scalar)
    pub fn shape(&self) -> Vec<usize> {
        match self {
            ArrayValue::Scalar(_) => vec![],
            ArrayValue::Array { shape, .. } => shape.clone(),
        }
    }

    /// Check if this is a scalar
    pub fn is_scalar(&self) -> bool {
        matches!(self, ArrayValue::Scalar(_))
    }

    /// Check if this is an array
    pub fn is_array(&self) -> bool {
        matches!(self, ArrayValue::Array { .. })
    }
}

/// Extended simulation state that supports multi-dimensional variables
#[derive(Debug, Clone)]
pub struct ArraySimulationState {
    pub time: f64,
    pub stocks: HashMap<String, ArrayValue>,
    pub flows: HashMap<String, ArrayValue>,
    pub auxiliaries: HashMap<String, ArrayValue>,
    pub delays: super::DelayManager,
    pub stochastic: super::StochasticManager,
    pub agents: super::AgentManager,
}

impl ArraySimulationState {
    pub fn new() -> Self {
        Self {
            time: 0.0,
            stocks: HashMap::new(),
            flows: HashMap::new(),
            auxiliaries: HashMap::new(),
            delays: super::DelayManager::new(),
            stochastic: super::StochasticManager::new(),
            agents: super::AgentManager::new(),
        }
    }

    /// Get a scalar or array value
    pub fn get_value(&self, name: &str) -> Option<&ArrayValue> {
        self.stocks
            .get(name)
            .or_else(|| self.flows.get(name))
            .or_else(|| self.auxiliaries.get(name))
    }

    /// Get a specific element from a variable (handles both scalar and array)
    pub fn get_element(&self, name: &str, indices: &[usize]) -> Result<f64, String> {
        self.get_value(name)
            .ok_or_else(|| format!("Variable '{}' not found", name))?
            .get(indices)
    }

    /// Initialize from a model, detecting array dimensions from model definition
    pub fn initialize_from_model(model: &crate::model::Model) -> Result<Self, String> {
        use crate::model::expression::EvaluationContext;

        let mut state = Self::new();
        state.time = model.time.start;

        // Initialize stocks with their initial values
        // For now, treat all as scalars (array support requires dimension info in model)
        for (name, stock) in &model.stocks {
            let mut temp_state_scalar = super::SimulationState::new();
            temp_state_scalar.time = model.time.start;

            let mut context = EvaluationContext::new(model, &mut temp_state_scalar, model.time.start);
            let initial_value = stock.initial.evaluate(&mut context)?;

            // Check if stock has dimensions defined
            if let Some(ref dim_names) = stock.dimensions {
                // Get shape from dimensions
                let mut shape = Vec::new();
                let mut valid = true;
                for dim_name in dim_names {
                    if let Some(dimension) = model.dimensions.get(dim_name) {
                        shape.push(dimension.elements.len());
                    } else {
                        valid = false;
                        break;
                    }
                }

                if valid && !shape.is_empty() {
                    // Create array initialized with the scalar value
                    let size: usize = shape.iter().product();
                    state.stocks.insert(name.clone(), ArrayValue::Array {
                        shape,
                        data: vec![initial_value; size],
                    });
                } else {
                    // Fallback to scalar if dimensions not found
                    state.stocks.insert(name.clone(), ArrayValue::Scalar(initial_value));
                }
            } else {
                // No dimensions defined, use scalar
                state.stocks.insert(name.clone(), ArrayValue::Scalar(initial_value));
            }

            // Merge back state changes
            state.delays = temp_state_scalar.delays;
            state.stochastic = temp_state_scalar.stochastic;
            state.agents = temp_state_scalar.agents;
        }

        // Initialize flows
        for name in model.flows.keys() {
            state.flows.insert(name.clone(), ArrayValue::Scalar(0.0));
        }

        // Initialize auxiliaries
        for name in model.auxiliaries.keys() {
            state.auxiliaries.insert(name.clone(), ArrayValue::Scalar(0.0));
        }

        Ok(state)
    }

    /// Convert to scalar SimulationState (for backward compatibility)
    /// Arrays are flattened or summed
    pub fn to_scalar_state(&self) -> super::SimulationState {
        let mut scalar_state = super::SimulationState::new();
        scalar_state.time = self.time;
        scalar_state.delays = self.delays.clone();
        scalar_state.stochastic = self.stochastic.clone();
        scalar_state.agents = self.agents.clone();

        // Convert stocks
        for (name, value) in &self.stocks {
            scalar_state.stocks.insert(
                name.clone(),
                value.as_scalar().unwrap_or_else(|_| {
                    // Sum all elements for arrays
                    if let ArrayValue::Array { data, .. } = value {
                        data.iter().sum()
                    } else {
                        0.0
                    }
                }),
            );
        }

        // Convert flows
        for (name, value) in &self.flows {
            scalar_state.flows.insert(
                name.clone(),
                value.as_scalar().unwrap_or_else(|_| {
                    if let ArrayValue::Array { data, .. } = value {
                        data.iter().sum()
                    } else {
                        0.0
                    }
                }),
            );
        }

        // Convert auxiliaries
        for (name, value) in &self.auxiliaries {
            scalar_state.auxiliaries.insert(
                name.clone(),
                value.as_scalar().unwrap_or_else(|_| {
                    if let ArrayValue::Array { data, .. } = value {
                        data.iter().sum()
                    } else {
                        0.0
                    }
                }),
            );
        }

        scalar_state
    }
}

impl Default for ArraySimulationState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scalar_value() {
        let val = ArrayValue::scalar(42.0);
        assert!(val.is_scalar());
        assert_eq!(val.as_scalar().unwrap(), 42.0);
        assert_eq!(val.get(&[]).unwrap(), 42.0);
    }

    #[test]
    fn test_1d_array() {
        let mut val = ArrayValue::from_vec(vec![3], vec![1.0, 2.0, 3.0]).unwrap();
        assert!(val.is_array());
        assert_eq!(val.shape(), vec![3]);
        assert_eq!(val.get(&[0]).unwrap(), 1.0);
        assert_eq!(val.get(&[2]).unwrap(), 3.0);

        val.set(&[1], 5.0).unwrap();
        assert_eq!(val.get(&[1]).unwrap(), 5.0);
    }

    #[test]
    fn test_2d_array() {
        // 2x3 matrix: [[1,2,3], [4,5,6]]
        let val = ArrayValue::from_vec(vec![2, 3], vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]).unwrap();
        assert_eq!(val.shape(), vec![2, 3]);
        assert_eq!(val.get(&[0, 0]).unwrap(), 1.0);
        assert_eq!(val.get(&[0, 2]).unwrap(), 3.0);
        assert_eq!(val.get(&[1, 0]).unwrap(), 4.0);
        assert_eq!(val.get(&[1, 2]).unwrap(), 6.0);
    }
}
