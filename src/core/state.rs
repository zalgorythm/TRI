//! Triangle state management for the fractal system

use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents the various states a triangle can be in within the fractal system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TriangleState {
    /// The initial genesis triangle - the root of the fractal
    Genesis,
    /// An active triangle that can be subdivided
    Active,
    /// A triangle that has been subdivided into child triangles
    Subdivided,
    /// The central void triangle created during subdivision
    Void,
    /// A triangle that has been marked as inactive
    Inactive,
}

impl TriangleState {
    /// Check if the triangle can be subdivided in its current state
    pub fn can_subdivide(&self) -> bool {
        matches!(self, TriangleState::Genesis | TriangleState::Active)
    }

    /// Check if the triangle can transition to the target state
    pub fn can_transition_to(&self, target: TriangleState) -> bool {
        use TriangleState::*;
        
        match (self, target) {
            // Genesis can only become subdivided
            (Genesis, Subdivided) => true,
            
            // Active can become subdivided or inactive
            (Active, Subdivided) => true,
            (Active, Inactive) => true,
            
            // Subdivided triangles cannot change state
            (Subdivided, _) => false,
            
            // Void triangles cannot change state
            (Void, _) => false,
            
            // Inactive triangles can become active again
            (Inactive, Active) => true,
            
            // No other transitions allowed
            _ => false,
        }
    }

    /// Get the next logical state after subdivision
    pub fn after_subdivision(&self) -> Option<TriangleState> {
        if self.can_subdivide() {
            Some(TriangleState::Subdivided)
        } else {
            None
        }
    }

    /// Check if this state represents a terminal state (cannot change)
    pub fn is_terminal(&self) -> bool {
        matches!(self, TriangleState::Subdivided | TriangleState::Void)
    }

    /// Get a human-readable description of the state
    pub fn description(&self) -> &'static str {
        match self {
            TriangleState::Genesis => "The root triangle of the fractal system",
            TriangleState::Active => "An active triangle that can be subdivided",
            TriangleState::Subdivided => "A triangle that has been divided into child triangles",
            TriangleState::Void => "The central void created during subdivision",
            TriangleState::Inactive => "An inactive triangle that is not currently processing",
        }
    }

    /// Get all possible states
    pub fn all_states() -> &'static [TriangleState] {
        &[
            TriangleState::Genesis,
            TriangleState::Active,
            TriangleState::Subdivided,
            TriangleState::Void,
            TriangleState::Inactive,
        ]
    }
}

impl fmt::Display for TriangleState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            TriangleState::Genesis => "Genesis",
            TriangleState::Active => "Active",
            TriangleState::Subdivided => "Subdivided",
            TriangleState::Void => "Void",
            TriangleState::Inactive => "Inactive",
        };
        write!(f, "{}", name)
    }
}

/// Represents a state transition with metadata
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateTransition {
    pub from: TriangleState,
    pub to: TriangleState,
    pub timestamp: u64,
    pub reason: String,
}

impl StateTransition {
    /// Create a new state transition
    pub fn new(from: TriangleState, to: TriangleState, reason: String) -> Self {
        StateTransition {
            from,
            to,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            reason,
        }
    }

    /// Check if this transition is valid
    pub fn is_valid(&self) -> bool {
        self.from.can_transition_to(self.to)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_transitions() {
        assert!(TriangleState::Genesis.can_transition_to(TriangleState::Subdivided));
        assert!(TriangleState::Active.can_transition_to(TriangleState::Subdivided));
        assert!(TriangleState::Active.can_transition_to(TriangleState::Inactive));
        assert!(TriangleState::Inactive.can_transition_to(TriangleState::Active));
        
        assert!(!TriangleState::Subdivided.can_transition_to(TriangleState::Active));
        assert!(!TriangleState::Void.can_transition_to(TriangleState::Active));
    }

    #[test]
    fn test_can_subdivide() {
        assert!(TriangleState::Genesis.can_subdivide());
        assert!(TriangleState::Active.can_subdivide());
        assert!(!TriangleState::Subdivided.can_subdivide());
        assert!(!TriangleState::Void.can_subdivide());
        assert!(!TriangleState::Inactive.can_subdivide());
    }

    #[test]
    fn test_terminal_states() {
        assert!(!TriangleState::Genesis.is_terminal());
        assert!(!TriangleState::Active.is_terminal());
        assert!(TriangleState::Subdivided.is_terminal());
        assert!(TriangleState::Void.is_terminal());
        assert!(!TriangleState::Inactive.is_terminal());
    }

    #[test]
    fn test_state_transition_creation() {
        let transition = StateTransition::new(
            TriangleState::Genesis,
            TriangleState::Subdivided,
            "Initial subdivision".to_string(),
        );
        assert!(transition.is_valid());
        assert_eq!(transition.from, TriangleState::Genesis);
        assert_eq!(transition.to, TriangleState::Subdivided);
    }
}
