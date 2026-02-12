/// Lookup table (graphical functions) support
///
/// Provides LOOKUP and WITH_LOOKUP functions for nonlinear relationships

use serde::{Deserialize, Serialize};

/// A lookup table / graphical function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LookupTable {
    pub name: String,
    /// Data points as (x, y) pairs, must be sorted by x
    pub points: Vec<(f64, f64)>,
}

impl LookupTable {
    pub fn new(name: String, points: Vec<(f64, f64)>) -> Result<Self, String> {
        if points.is_empty() {
            return Err("Lookup table must have at least one point".to_string());
        }

        // Verify points are sorted by x
        for i in 1..points.len() {
            if points[i].0 < points[i - 1].0 {
                return Err("Lookup table points must be sorted by x value".to_string());
            }
        }

        Ok(Self { name, points })
    }

    /// Lookup a value with linear interpolation
    /// Extrapolates flat (constant) outside the range
    pub fn lookup(&self, x: f64) -> f64 {
        if self.points.is_empty() {
            return 0.0;
        }

        // Before first point - return first y value
        if x <= self.points[0].0 {
            return self.points[0].1;
        }

        // After last point - return last y value
        if x >= self.points[self.points.len() - 1].0 {
            return self.points[self.points.len() - 1].1;
        }

        // Find the two points to interpolate between
        for i in 1..self.points.len() {
            if x <= self.points[i].0 {
                // Interpolate between i-1 and i
                let (x1, y1) = self.points[i - 1];
                let (x2, y2) = self.points[i];

                // Linear interpolation
                let alpha = (x - x1) / (x2 - x1);
                return y1 + alpha * (y2 - y1);
            }
        }

        // Should never reach here
        self.points[self.points.len() - 1].1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_interpolation() {
        let table = LookupTable::new(
            "test".to_string(),
            vec![(0.0, 0.0), (1.0, 10.0), (2.0, 5.0)],
        )
        .unwrap();

        // Exact points
        assert_eq!(table.lookup(0.0), 0.0);
        assert_eq!(table.lookup(1.0), 10.0);
        assert_eq!(table.lookup(2.0), 5.0);

        // Interpolation
        assert_eq!(table.lookup(0.5), 5.0);
        assert_eq!(table.lookup(1.5), 7.5);

        // Extrapolation
        assert_eq!(table.lookup(-1.0), 0.0);
        assert_eq!(table.lookup(3.0), 5.0);
    }
}
