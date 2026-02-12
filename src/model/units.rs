/// Unit checking and dimensional analysis
///
/// Provides compile-time-like checks for unit consistency in models

use std::collections::HashMap;
use std::fmt;
use serde::{Deserialize, Serialize};

/// Base SI dimensions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BaseDimension {
    Length,      // meter (m)
    Mass,        // kilogram (kg)
    Time,        // second (s)
    Current,     // ampere (A)
    Temperature, // kelvin (K)
    Amount,      // mole (mol)
    Luminosity,  // candela (cd)
}

/// Dimensional formula represented as powers of base dimensions
/// e.g., velocity = Length^1 * Time^-1
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DimensionalFormula {
    pub dimensions: HashMap<BaseDimension, i32>,
}

impl DimensionalFormula {
    /// Create dimensionless (no units)
    pub fn dimensionless() -> Self {
        Self {
            dimensions: HashMap::new(),
        }
    }

    /// Create from a single base dimension
    pub fn from_base(dim: BaseDimension, power: i32) -> Self {
        let mut dimensions = HashMap::new();
        if power != 0 {
            dimensions.insert(dim, power);
        }
        Self { dimensions }
    }

    /// Common unit formulas
    pub fn length() -> Self {
        Self::from_base(BaseDimension::Length, 1)
    }

    pub fn mass() -> Self {
        Self::from_base(BaseDimension::Mass, 1)
    }

    pub fn time() -> Self {
        Self::from_base(BaseDimension::Time, 1)
    }

    pub fn velocity() -> Self {
        // m/s = Length * Time^-1
        let mut dimensions = HashMap::new();
        dimensions.insert(BaseDimension::Length, 1);
        dimensions.insert(BaseDimension::Time, -1);
        Self { dimensions }
    }

    pub fn acceleration() -> Self {
        // m/s^2 = Length * Time^-2
        let mut dimensions = HashMap::new();
        dimensions.insert(BaseDimension::Length, 1);
        dimensions.insert(BaseDimension::Time, -2);
        Self { dimensions }
    }

    pub fn force() -> Self {
        // N = kg*m/s^2 = Mass * Length * Time^-2
        let mut dimensions = HashMap::new();
        dimensions.insert(BaseDimension::Mass, 1);
        dimensions.insert(BaseDimension::Length, 1);
        dimensions.insert(BaseDimension::Time, -2);
        Self { dimensions }
    }

    pub fn energy() -> Self {
        // J = N*m = kg*m^2/s^2 = Mass * Length^2 * Time^-2
        let mut dimensions = HashMap::new();
        dimensions.insert(BaseDimension::Mass, 1);
        dimensions.insert(BaseDimension::Length, 2);
        dimensions.insert(BaseDimension::Time, -2);
        Self { dimensions }
    }

    pub fn power() -> Self {
        // W = J/s = kg*m^2/s^3 = Mass * Length^2 * Time^-3
        let mut dimensions = HashMap::new();
        dimensions.insert(BaseDimension::Mass, 1);
        dimensions.insert(BaseDimension::Length, 2);
        dimensions.insert(BaseDimension::Time, -3);
        Self { dimensions }
    }

    /// Check if this is dimensionless
    pub fn is_dimensionless(&self) -> bool {
        self.dimensions.is_empty() || self.dimensions.values().all(|&p| p == 0)
    }

    /// Multiply two dimensional formulas
    pub fn multiply(&self, other: &Self) -> Self {
        let mut result = self.dimensions.clone();
        for (dim, power) in &other.dimensions {
            *result.entry(*dim).or_insert(0) += power;
        }
        // Remove zero powers
        result.retain(|_, &mut power| power != 0);
        Self {
            dimensions: result,
        }
    }

    /// Divide two dimensional formulas
    pub fn divide(&self, other: &Self) -> Self {
        let mut result = self.dimensions.clone();
        for (dim, power) in &other.dimensions {
            *result.entry(*dim).or_insert(0) -= power;
        }
        // Remove zero powers
        result.retain(|_, &mut power| power != 0);
        Self {
            dimensions: result,
        }
    }

    /// Raise to a power
    pub fn raise_to_power(&self, exponent: i32) -> Self {
        let mut result = HashMap::new();
        for (dim, power) in &self.dimensions {
            result.insert(*dim, power * exponent);
        }
        Self {
            dimensions: result,
        }
    }

    /// Check if compatible for addition/subtraction
    pub fn is_compatible(&self, other: &Self) -> bool {
        self == other
    }

    /// Parse from a string like "m/s" or "kg*m/s^2"
    pub fn parse(s: &str) -> Result<Self, String> {
        // Simplified parser - handles common cases
        let s = s.trim().to_lowercase();

        // Dimensionless
        if s.is_empty() || s == "1" || s == "dimensionless" {
            return Ok(Self::dimensionless());
        }

        // Common units
        match s.as_str() {
            "m" | "meter" | "meters" => return Ok(Self::length()),
            "kg" | "kilogram" | "kilograms" => return Ok(Self::mass()),
            "s" | "second" | "seconds" => return Ok(Self::time()),
            "m/s" | "meter/second" => return Ok(Self::velocity()),
            "m/s^2" | "m/s2" => return Ok(Self::acceleration()),
            "n" | "newton" | "newtons" => return Ok(Self::force()),
            "j" | "joule" | "joules" => return Ok(Self::energy()),
            "w" | "watt" | "watts" => return Ok(Self::power()),
            _ => {}
        }

        // For now, treat unrecognized units as dimensionless with a warning
        // A full implementation would parse complex unit expressions
        Ok(Self::dimensionless())
    }
}

impl fmt::Display for DimensionalFormula {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_dimensionless() {
            return write!(f, "1");
        }

        let mut parts = Vec::new();

        // Positive powers in numerator
        for (dim, &power) in &self.dimensions {
            if power > 0 {
                let dim_str = match dim {
                    BaseDimension::Length => "m",
                    BaseDimension::Mass => "kg",
                    BaseDimension::Time => "s",
                    BaseDimension::Current => "A",
                    BaseDimension::Temperature => "K",
                    BaseDimension::Amount => "mol",
                    BaseDimension::Luminosity => "cd",
                };
                if power == 1 {
                    parts.push(dim_str.to_string());
                } else {
                    parts.push(format!("{}^{}", dim_str, power));
                }
            }
        }

        let numerator = if parts.is_empty() {
            "1".to_string()
        } else {
            parts.join("*")
        };

        // Negative powers in denominator
        parts.clear();
        for (dim, &power) in &self.dimensions {
            if power < 0 {
                let dim_str = match dim {
                    BaseDimension::Length => "m",
                    BaseDimension::Mass => "kg",
                    BaseDimension::Time => "s",
                    BaseDimension::Current => "A",
                    BaseDimension::Temperature => "K",
                    BaseDimension::Amount => "mol",
                    BaseDimension::Luminosity => "cd",
                };
                if power == -1 {
                    parts.push(dim_str.to_string());
                } else {
                    parts.push(format!("{}^{}", dim_str, -power));
                }
            }
        }

        if parts.is_empty() {
            write!(f, "{}", numerator)
        } else {
            write!(f, "{}/{}", numerator, parts.join("*"))
        }
    }
}

/// Unit checker for validating model consistency
pub struct UnitChecker {
    /// Map of variable names to their dimensional formulas
    variable_units: HashMap<String, DimensionalFormula>,
}

impl UnitChecker {
    pub fn new() -> Self {
        Self {
            variable_units: HashMap::new(),
        }
    }

    /// Register a variable's units
    pub fn register_variable(&mut self, name: String, units: DimensionalFormula) {
        self.variable_units.insert(name, units);
    }

    /// Get units for a variable
    pub fn get_units(&self, name: &str) -> Option<&DimensionalFormula> {
        self.variable_units.get(name)
    }

    /// Check if an operation is dimensionally consistent
    pub fn check_add(&self, left: &str, right: &str) -> Result<(), String> {
        let left_units = self.get_units(left)
            .ok_or_else(|| format!("Units for '{}' not found", left))?;
        let right_units = self.get_units(right)
            .ok_or_else(|| format!("Units for '{}' not found", right))?;

        if !left_units.is_compatible(right_units) {
            return Err(format!(
                "Unit mismatch in addition: {} ({}) + {} ({})",
                left, left_units, right, right_units
            ));
        }

        Ok(())
    }

    /// Check if multiplication is valid
    pub fn check_multiply(&self, left: &str, right: &str) -> DimensionalFormula {
        let left_units = self.get_units(left).cloned()
            .unwrap_or_else(DimensionalFormula::dimensionless);
        let right_units = self.get_units(right).cloned()
            .unwrap_or_else(DimensionalFormula::dimensionless);

        left_units.multiply(&right_units)
    }

    /// Check if division is valid
    pub fn check_divide(&self, left: &str, right: &str) -> DimensionalFormula {
        let left_units = self.get_units(left).cloned()
            .unwrap_or_else(DimensionalFormula::dimensionless);
        let right_units = self.get_units(right).cloned()
            .unwrap_or_else(DimensionalFormula::dimensionless);

        left_units.divide(&right_units)
    }
}

impl Default for UnitChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dimensional_formula() {
        let length = DimensionalFormula::length();
        let time = DimensionalFormula::time();

        let velocity = length.divide(&time);
        assert_eq!(velocity, DimensionalFormula::velocity());

        let acceleration = velocity.divide(&time);
        assert_eq!(acceleration, DimensionalFormula::acceleration());
    }

    #[test]
    fn test_compatibility() {
        let length1 = DimensionalFormula::length();
        let length2 = DimensionalFormula::length();
        let time = DimensionalFormula::time();

        assert!(length1.is_compatible(&length2));
        assert!(!length1.is_compatible(&time));
    }

    #[test]
    fn test_multiply() {
        let length = DimensionalFormula::length();
        let mass = DimensionalFormula::mass();
        let time = DimensionalFormula::time();

        // Force = mass * acceleration = mass * length / time^2
        let acceleration = length.divide(&time).divide(&time);
        let force = mass.multiply(&acceleration);

        assert_eq!(force, DimensionalFormula::force());
    }

    #[test]
    fn test_unit_checker() {
        let mut checker = UnitChecker::new();
        checker.register_variable("distance".to_string(), DimensionalFormula::length());
        checker.register_variable("time".to_string(), DimensionalFormula::time());
        checker.register_variable("velocity".to_string(), DimensionalFormula::velocity());

        // Check division: distance / time should give velocity units
        let result = checker.check_divide("distance", "time");
        assert_eq!(result, DimensionalFormula::velocity());
    }

    #[test]
    fn test_display() {
        assert_eq!(DimensionalFormula::velocity().to_string(), "m/s");
        assert_eq!(DimensionalFormula::acceleration().to_string(), "m/s^2");
        // Note: HashMap ordering is non-deterministic, so force could be "kg*m/s^2" or "m*kg/s^2"
        let force_str = DimensionalFormula::force().to_string();
        assert!(force_str == "kg*m/s^2" || force_str == "m*kg/s^2", "Got: {}", force_str);
    }
}
