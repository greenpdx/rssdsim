/// Dimension/Subscript support for multi-dimensional variables

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A dimension (also called subscript or index) for array variables
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dimension {
    pub name: String,
    pub elements: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl Dimension {
    pub fn new(name: &str, elements: Vec<String>) -> Self {
        Self {
            name: name.to_string(),
            elements,
            description: None,
        }
    }

    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    /// Get the size of this dimension
    pub fn size(&self) -> usize {
        self.elements.len()
    }

    /// Get the index of an element in this dimension
    pub fn get_index(&self, element: &str) -> Option<usize> {
        self.elements.iter().position(|e| e == element)
    }

    /// Check if an element exists in this dimension
    pub fn contains(&self, element: &str) -> bool {
        self.elements.iter().any(|e| e == element)
    }
}

/// A subscript reference in an expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SubscriptRef {
    /// Reference to a specific element (e.g., "North")
    Element(String),
    /// Reference to entire dimension (e.g., "Region" - sum over all)
    Dimension(String),
    /// Wildcard/asterisk to iterate over all elements
    Wildcard,
}

/// Manager for dimension definitions and subscript resolution
#[derive(Debug, Clone, Default)]
pub struct DimensionManager {
    dimensions: HashMap<String, Dimension>,
}

impl DimensionManager {
    pub fn new() -> Self {
        Self {
            dimensions: HashMap::new(),
        }
    }

    /// Add a dimension
    pub fn add_dimension(&mut self, dimension: Dimension) -> Result<(), String> {
        if self.dimensions.contains_key(&dimension.name) {
            return Err(format!("Dimension '{}' already exists", dimension.name));
        }
        self.dimensions.insert(dimension.name.clone(), dimension);
        Ok(())
    }

    /// Get a dimension by name
    pub fn get_dimension(&self, name: &str) -> Option<&Dimension> {
        self.dimensions.get(name)
    }

    /// Resolve a subscript reference to element names
    /// Returns a vector of element names
    pub fn resolve_subscript(
        &self,
        subscript: &SubscriptRef,
        dimension_name: &str,
    ) -> Result<Vec<String>, String> {
        let dimension = self
            .get_dimension(dimension_name)
            .ok_or_else(|| format!("Dimension '{}' not found", dimension_name))?;

        match subscript {
            SubscriptRef::Element(element) => {
                if dimension.contains(element) {
                    Ok(vec![element.clone()])
                } else {
                    Err(format!(
                        "Element '{}' not found in dimension '{}'",
                        element, dimension_name
                    ))
                }
            }
            SubscriptRef::Dimension(dim_name) => {
                // Check if this is the same dimension or a subset
                if dim_name == dimension_name {
                    Ok(dimension.elements.clone())
                } else {
                    // Could be a subset - not implemented yet
                    Err(format!("Subset '{}' not found", dim_name))
                }
            }
            SubscriptRef::Wildcard => Ok(dimension.elements.clone()),
        }
    }

    /// Get the shape (sizes) of multiple dimensions
    pub fn get_shape(&self, dimension_names: &[String]) -> Result<Vec<usize>, String> {
        dimension_names
            .iter()
            .map(|name| {
                self.get_dimension(name)
                    .map(|d| d.size())
                    .ok_or_else(|| format!("Dimension '{}' not found", name))
            })
            .collect()
    }

    /// Convert multi-dimensional index to flat index
    /// For example, [2, 3] in a [5, 4] array -> 2*4 + 3 = 11
    pub fn indices_to_flat(&self, indices: &[usize], shape: &[usize]) -> usize {
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

    /// Convert flat index to multi-dimensional indices
    pub fn flat_to_indices(&self, flat_index: usize, shape: &[usize]) -> Vec<usize> {
        let mut indices = vec![0; shape.len()];
        let mut remaining = flat_index;

        for i in (0..shape.len()).rev() {
            let stride: usize = shape[i + 1..].iter().product();
            indices[i] = remaining / stride;
            remaining %= stride;
        }

        indices
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dimension_creation() {
        let dim = Dimension::new("Region", vec!["North".to_string(), "South".to_string()]);
        assert_eq!(dim.name, "Region");
        assert_eq!(dim.size(), 2);
        assert!(dim.contains("North"));
        assert!(!dim.contains("East"));
    }

    #[test]
    fn test_dimension_manager() {
        let mut manager = DimensionManager::new();
        let dim = Dimension::new("Region", vec!["North".to_string(), "South".to_string()]);
        manager.add_dimension(dim).unwrap();

        let elements = manager
            .resolve_subscript(&SubscriptRef::Wildcard, "Region")
            .unwrap();
        assert_eq!(elements, vec!["North", "South"]);
    }
}
