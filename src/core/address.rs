//! Hierarchical addressing system for triangles in the fractal

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::core::errors::{SierpinskiError, SierpinskiResult};

/// Hierarchical address for a triangle in the Sierpinski fractal
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TriangleAddress {
    /// Path components from root to this triangle
    /// Each component represents which child (0, 1, 2) was taken at each level
    path: Vec<u8>,
}

impl TriangleAddress {
    /// Create a new address from a path
    pub fn new(path: Vec<u8>) -> SierpinskiResult<Self> {
        // Validate that all path components are in valid range (0-2 for children, 3 for void)
        for &component in &path {
            if component > 3 {
                return Err(SierpinskiError::AddressComponentOutOfRange { component });
            }
        }
        Ok(TriangleAddress { path })
    }

    /// Create the genesis address (empty path)
    pub fn genesis() -> Self {
        TriangleAddress { path: Vec::new() }
    }

    /// Create a child address by appending a component
    pub fn child(&self, component: u8) -> SierpinskiResult<Self> {
        if component > 3 {
            return Err(SierpinskiError::AddressComponentOutOfRange { component });
        }
        
        let mut new_path = self.path.clone();
        new_path.push(component);
        Ok(TriangleAddress { path: new_path })
    }

    /// Get the parent address
    pub fn parent(&self) -> Option<Self> {
        if self.path.is_empty() {
            None // Genesis has no parent
        } else {
            let mut parent_path = self.path.clone();
            parent_path.pop();
            Some(TriangleAddress { path: parent_path })
        }
    }

    /// Get the depth of this address (number of components)
    pub fn depth(&self) -> u8 {
        self.path.len() as u8
    }

    /// Check if this is the genesis address
    pub fn is_genesis(&self) -> bool {
        self.path.is_empty()
    }

    /// Check if this address is a child of another address
    pub fn is_child_of(&self, other: &TriangleAddress) -> bool {
        if self.path.len() != other.path.len() + 1 {
            return false;
        }
        
        for (i, &component) in other.path.iter().enumerate() {
            if self.path[i] != component {
                return false;
            }
        }
        
        true
    }

    /// Check if this address is an ancestor of another address
    pub fn is_ancestor_of(&self, other: &TriangleAddress) -> bool {
        if self.path.len() >= other.path.len() {
            return false;
        }
        
        for (i, &component) in self.path.iter().enumerate() {
            if other.path[i] != component {
                return false;
            }
        }
        
        true
    }

    /// Check if this address represents a void triangle
    pub fn is_void(&self) -> bool {
        self.path.last() == Some(&3)
    }

    /// Get the last component of the address
    pub fn last_component(&self) -> Option<u8> {
        self.path.last().copied()
    }

    /// Get all components of the address
    pub fn components(&self) -> &[u8] {
        &self.path
    }

    /// Convert to string representation (e.g., "0.1.2")
    pub fn to_string_representation(&self) -> String {
        if self.path.is_empty() {
            "genesis".to_string()
        } else {
            self.path
                .iter()
                .map(|&c| c.to_string())
                .collect::<Vec<_>>()
                .join(".")
        }
    }

    /// Parse from string representation
    pub fn from_string_representation(s: &str) -> SierpinskiResult<Self> {
        if s == "genesis" {
            return Ok(TriangleAddress::genesis());
        }
        
        let components: Result<Vec<u8>, _> = s
            .split('.')
            .map(|part| {
                part.parse::<u8>().map_err(|_| {
                    SierpinskiError::InvalidAddress {
                        address: s.to_string(),
                    }
                })
            })
            .collect();
        
        let path = components?;
        TriangleAddress::new(path)
    }

    /// Get all sibling addresses (same parent, different last component)
    pub fn siblings(&self) -> Vec<TriangleAddress> {
        if self.is_genesis() {
            return vec![]; // Genesis has no siblings
        }
        
        let parent = self.parent().unwrap();
        let mut siblings = Vec::new();
        
        for component in 0..=3 {
            if let Ok(sibling) = parent.child(component) {
                if sibling != *self {
                    siblings.push(sibling);
                }
            }
        }
        
        siblings
    }

    /// Generate all possible child addresses
    pub fn children(&self) -> Vec<TriangleAddress> {
        let mut children = Vec::new();
        
        for component in 0..=3 {
            if let Ok(child) = self.child(component) {
                children.push(child);
            }
        }
        
        children
    }

    /// Calculate the theoretical position index at this depth
    pub fn position_index(&self) -> u64 {
        let mut index = 0u64;
        let mut multiplier = 1u64;
        
        for &component in self.path.iter().rev() {
            index += (component as u64) * multiplier;
            multiplier *= 4; // 4 possible children (0, 1, 2, 3)
        }
        
        index
    }

    /// Get the common ancestor with another address
    pub fn common_ancestor(&self, other: &TriangleAddress) -> TriangleAddress {
        let mut common_path = Vec::new();
        
        let min_len = self.path.len().min(other.path.len());
        for i in 0..min_len {
            if self.path[i] == other.path[i] {
                common_path.push(self.path[i]);
            } else {
                break;
            }
        }
        
        TriangleAddress { path: common_path }
    }
}

impl fmt::Display for TriangleAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string_representation())
    }
}

/// Iterator for traversing addresses in breadth-first order
pub struct AddressBfsIterator {
    current_depth: u8,
    max_depth: u8,
    current_indices: Vec<u64>,
    total_at_depth: u64,
}

impl AddressBfsIterator {
    /// Create a new breadth-first iterator up to max_depth
    pub fn new(max_depth: u8) -> Self {
        AddressBfsIterator {
            current_depth: 0,
            max_depth,
            current_indices: vec![0],
            total_at_depth: 1,
        }
    }
}

impl Iterator for AddressBfsIterator {
    type Item = TriangleAddress;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_depth > self.max_depth {
            return None;
        }

        // Generate address for current position
        let mut path = Vec::new();
        let mut index = self.current_indices[0];
        
        for _ in 0..self.current_depth {
            path.push((index % 4) as u8);
            index /= 4;
        }
        path.reverse();

        let address = TriangleAddress { path };

        // Move to next position
        self.current_indices[0] += 1;
        
        // Check if we've exhausted current depth
        if self.current_indices[0] >= self.total_at_depth {
            self.current_depth += 1;
            if self.current_depth <= self.max_depth {
                self.total_at_depth = 4_u64.pow(self.current_depth as u32);
                self.current_indices = vec![0];
            }
        }

        Some(address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_creation() {
        let genesis = TriangleAddress::genesis();
        assert!(genesis.is_genesis());
        assert_eq!(genesis.depth(), 0);

        let child = genesis.child(1).unwrap();
        assert_eq!(child.depth(), 1);
        assert!(!child.is_genesis());
    }

    #[test]
    fn test_string_representation() {
        let genesis = TriangleAddress::genesis();
        assert_eq!(genesis.to_string_representation(), "genesis");

        let address = TriangleAddress::new(vec![0, 1, 2]).unwrap();
        assert_eq!(address.to_string_representation(), "0.1.2");
    }

    #[test]
    fn test_string_parsing() {
        let address = TriangleAddress::from_string_representation("0.1.2").unwrap();
        assert_eq!(address.components(), &[0, 1, 2]);

        let genesis = TriangleAddress::from_string_representation("genesis").unwrap();
        assert!(genesis.is_genesis());
    }

    #[test]
    fn test_parent_child_relationships() {
        let parent = TriangleAddress::new(vec![0, 1]).unwrap();
        let child = parent.child(2).unwrap();
        
        assert!(child.is_child_of(&parent));
        assert!(parent.is_ancestor_of(&child));
        assert_eq!(child.parent().unwrap(), parent);
    }

    #[test]
    fn test_siblings() {
        let address = TriangleAddress::new(vec![0, 1]).unwrap();
        let siblings = address.siblings();
        
        assert_eq!(siblings.len(), 3); // 3 siblings (0.0, 0.2, 0.3)
        assert!(!siblings.contains(&address)); // Should not contain self
    }

    #[test]
    fn test_void_detection() {
        let void_address = TriangleAddress::new(vec![0, 1, 3]).unwrap();
        assert!(void_address.is_void());

        let normal_address = TriangleAddress::new(vec![0, 1, 2]).unwrap();
        assert!(!normal_address.is_void());
    }

    #[test]
    fn test_common_ancestor() {
        let addr1 = TriangleAddress::new(vec![0, 1, 2]).unwrap();
        let addr2 = TriangleAddress::new(vec![0, 1, 0]).unwrap();
        let ancestor = addr1.common_ancestor(&addr2);
        
        assert_eq!(ancestor.components(), &[0, 1]);
    }

    #[test]
    fn test_invalid_components() {
        let result = TriangleAddress::new(vec![0, 1, 4]);
        assert!(matches!(result, Err(SierpinskiError::AddressComponentOutOfRange { component: 4 })));
    }
}
