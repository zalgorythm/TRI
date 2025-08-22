//! Fractal triangle implementation with hierarchical structure

use uuid::Uuid;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::core::{
    triangle::Triangle,
    state::TriangleState,
    address::TriangleAddress,
    errors::{SierpinskiError, SierpinskiResult},
};

/// A triangle within the Sierpinski fractal system with metadata
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FractalTriangle {
    /// Unique identifier for this triangle
    pub id: Uuid,
    /// The geometric triangle
    pub triangle: Triangle,
    /// Current state of the triangle
    pub state: TriangleState,
    /// Hierarchical address in the fractal
    pub address: TriangleAddress,
    /// Subdivision depth (0 for genesis)
    pub depth: u8,
    /// Parent triangle ID (None for genesis)
    pub parent_id: Option<Uuid>,
    /// Child triangle IDs (empty if not subdivided)
    pub child_ids: Vec<Uuid>,
    /// Creation timestamp
    pub created_at: u64,
    /// Last state change timestamp
    pub updated_at: u64,
}

impl FractalTriangle {
    /// Create a new fractal triangle
    pub fn new(triangle: Triangle, state: TriangleState, address: TriangleAddress, depth: u8) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        FractalTriangle {
            id: Uuid::new_v4(),
            triangle,
            state,
            address,
            depth,
            parent_id: None,
            child_ids: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Create the genesis triangle
    pub fn genesis(triangle: Triangle) -> Self {
        FractalTriangle::new(
            triangle,
            TriangleState::Genesis,
            TriangleAddress::genesis(),
            0,
        )
    }

    /// Create a child triangle
    pub fn child(
        triangle: Triangle,
        parent: &FractalTriangle,
        child_index: u8,
    ) -> SierpinskiResult<Self> {
        if parent.depth >= crate::MAX_SUBDIVISION_DEPTH {
            return Err(SierpinskiError::MaxDepthExceeded {
                max_depth: crate::MAX_SUBDIVISION_DEPTH,
            });
        }

        let child_address = parent.address.child(child_index)?;
        let mut child = FractalTriangle::new(
            triangle,
            TriangleState::Active,
            child_address,
            parent.depth + 1,
        );
        child.parent_id = Some(parent.id);
        Ok(child)
    }

    /// Change the state of the triangle
    pub fn change_state(&mut self, new_state: TriangleState) -> SierpinskiResult<()> {
        if !self.state.can_transition_to(new_state) {
            return Err(SierpinskiError::StateTransitionError {
                from: self.state.to_string(),
                to: new_state.to_string(),
            });
        }

        self.state = new_state;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Ok(())
    }

    /// Add a child triangle ID
    pub fn add_child(&mut self, child_id: Uuid) {
        if !self.child_ids.contains(&child_id) {
            self.child_ids.push(child_id);
        }
    }

    /// Check if this triangle has children
    pub fn has_children(&self) -> bool {
        !self.child_ids.is_empty()
    }

    /// Check if this triangle can be subdivided
    pub fn can_subdivide(&self) -> bool {
        self.state.can_subdivide() && self.depth < crate::MAX_SUBDIVISION_DEPTH
    }

    /// Get the total area covered by this triangle
    pub fn area(&self) -> SierpinskiResult<rust_decimal::Decimal> {
        self.triangle.area()
    }

    /// Check if this triangle is at the maximum depth
    pub fn is_at_max_depth(&self) -> bool {
        self.depth >= crate::MAX_SUBDIVISION_DEPTH
    }

    /// Get the generation (same as depth) of this triangle
    pub fn generation(&self) -> u8 {
        self.depth
    }

    /// Calculate the theoretical area ratio compared to genesis
    pub fn area_ratio_to_genesis(&self) -> rust_decimal::Decimal {
        // Each subdivision reduces area by 3/4
        let three_fourths = rust_decimal::Decimal::new(3, 0) / rust_decimal::Decimal::new(4, 0);
        let mut ratio = rust_decimal::Decimal::ONE;
        
        for _ in 0..self.depth {
            ratio *= three_fourths;
        }
        
        ratio
    }

    /// Get a hash representation of this fractal triangle
    pub fn hash(&self) -> String {
        let mut hasher = blake3::Hasher::new();
        hasher.update(self.id.as_bytes());
        hasher.update(self.triangle.hash().as_bytes());
        hasher.update(&[self.depth]);
        hasher.finalize().to_hex().to_string()
    }
}

/// A collection of fractal triangles forming the complete fractal structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FractalStructure {
    /// Map of triangle ID to fractal triangle
    triangles: HashMap<Uuid, FractalTriangle>,
    /// Genesis triangle ID
    genesis_id: Option<Uuid>,
    /// Maximum depth reached
    max_depth: u8,
    /// Total number of triangles
    total_count: usize,
}

impl FractalStructure {
    /// Create a new empty fractal structure
    pub fn new() -> Self {
        FractalStructure {
            triangles: HashMap::new(),
            genesis_id: None,
            max_depth: 0,
            total_count: 0,
        }
    }

    /// Add the genesis triangle
    pub fn set_genesis(&mut self, triangle: FractalTriangle) -> SierpinskiResult<()> {
        if triangle.state != TriangleState::Genesis {
            return Err(SierpinskiError::validation(
                "Genesis triangle must have Genesis state",
            ));
        }

        self.genesis_id = Some(triangle.id);
        self.triangles.insert(triangle.id, triangle);
        self.total_count = 1;
        Ok(())
    }

    /// Add a triangle to the structure
    pub fn add_triangle(&mut self, triangle: FractalTriangle) -> SierpinskiResult<()> {
        // Update max depth
        if triangle.depth > self.max_depth {
            self.max_depth = triangle.depth;
        }

        // If triangle has a parent, add this as a child
        if let Some(parent_id) = triangle.parent_id {
            if let Some(parent) = self.triangles.get_mut(&parent_id) {
                parent.add_child(triangle.id);
            }
        }

        self.triangles.insert(triangle.id, triangle);
        self.total_count = self.triangles.len();
        Ok(())
    }

    /// Get a triangle by ID
    pub fn get_triangle(&self, id: &Uuid) -> Option<&FractalTriangle> {
        self.triangles.get(id)
    }

    /// Get a mutable reference to a triangle by ID
    pub fn get_triangle_mut(&mut self, id: &Uuid) -> Option<&mut FractalTriangle> {
        self.triangles.get_mut(id)
    }

    /// Get the genesis triangle
    pub fn genesis(&self) -> Option<&FractalTriangle> {
        self.genesis_id.and_then(|id| self.triangles.get(&id))
    }

    /// Get all triangles at a specific depth
    pub fn triangles_at_depth(&self, depth: u8) -> Vec<&FractalTriangle> {
        self.triangles
            .values()
            .filter(|t| t.depth == depth)
            .collect()
    }

    /// Get triangles by state
    pub fn triangles_by_state(&self, state: TriangleState) -> Vec<&FractalTriangle> {
        self.triangles
            .values()
            .filter(|t| t.state == state)
            .collect()
    }

    /// Get the total number of triangles
    pub fn total_triangles(&self) -> usize {
        self.total_count
    }

    /// Get the maximum depth reached
    pub fn max_depth(&self) -> u8 {
        self.max_depth
    }

    /// Calculate total area of all active triangles
    pub fn total_active_area(&self) -> SierpinskiResult<rust_decimal::Decimal> {
        let mut total = rust_decimal::Decimal::ZERO;
        
        for triangle in self.triangles.values() {
            if triangle.state == TriangleState::Active || triangle.state == TriangleState::Genesis {
                total += triangle.area()?;
            }
        }
        
        Ok(total)
    }

    /// Get children of a triangle
    pub fn children(&self, parent_id: &Uuid) -> Vec<&FractalTriangle> {
        if let Some(parent) = self.triangles.get(parent_id) {
            parent
                .child_ids
                .iter()
                .filter_map(|id| self.triangles.get(id))
                .collect()
        } else {
            Vec::new()
        }
    }
}

impl Default for FractalStructure {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::geometry::Point;

    fn create_test_triangle() -> Triangle {
        Triangle::new(
            Point::from_f64(0.0, 0.0).unwrap(),
            Point::from_f64(1.0, 0.0).unwrap(),
            Point::from_f64(0.5, 0.866).unwrap(),
        ).unwrap()
    }

    #[test]
    fn test_fractal_triangle_creation() {
        let triangle = create_test_triangle();
        let fractal_triangle = FractalTriangle::genesis(triangle);
        
        assert_eq!(fractal_triangle.state, TriangleState::Genesis);
        assert_eq!(fractal_triangle.depth, 0);
        assert!(fractal_triangle.parent_id.is_none());
        assert!(fractal_triangle.child_ids.is_empty());
    }

    #[test]
    fn test_child_creation() {
        let parent_triangle = create_test_triangle();
        let parent = FractalTriangle::genesis(parent_triangle);
        
        let child_triangle = create_test_triangle();
        let child = FractalTriangle::child(child_triangle, &parent, 0).unwrap();
        
        assert_eq!(child.depth, 1);
        assert_eq!(child.parent_id, Some(parent.id));
        assert_eq!(child.state, TriangleState::Active);
    }

    #[test]
    fn test_fractal_structure() {
        let mut structure = FractalStructure::new();
        let triangle = create_test_triangle();
        let genesis = FractalTriangle::genesis(triangle);
        let genesis_id = genesis.id;
        
        structure.set_genesis(genesis).unwrap();
        
        assert_eq!(structure.total_triangles(), 1);
        assert_eq!(structure.max_depth(), 0);
        assert!(structure.genesis().is_some());
        assert_eq!(structure.genesis().unwrap().id, genesis_id);
    }

    #[test]
    fn test_state_transitions() {
        let triangle = create_test_triangle();
        let mut fractal_triangle = FractalTriangle::genesis(triangle);
        
        // Genesis can transition to Subdivided
        assert!(fractal_triangle.change_state(TriangleState::Subdivided).is_ok());
        assert_eq!(fractal_triangle.state, TriangleState::Subdivided);
        
        // Subdivided cannot transition back
        assert!(fractal_triangle.change_state(TriangleState::Active).is_err());
    }
}
