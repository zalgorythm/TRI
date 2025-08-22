//! Blockchain block structure for Sierpinski Triangle cryptocurrency

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rust_decimal::Decimal;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::{
    triangle::Triangle,
    fractal::FractalTriangle,
    address::TriangleAddress,
    errors::{SierpinskiError, SierpinskiResult},
};

/// Transaction representing triangle operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TriangleTransaction {
    pub id: Uuid,
    pub from_address: Option<TriangleAddress>,
    pub to_address: TriangleAddress,
    pub operation: TriangleOperation,
    pub triangle_data: Option<Triangle>,
    pub signature: Vec<u8>,
    pub timestamp: u64,
    pub gas_fee: Decimal,
}

/// Types of triangle operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TriangleOperation {
    /// Create a new triangle (genesis or child)
    Create,
    /// Subdivide an existing triangle
    Subdivide,
    /// Transfer triangle ownership
    Transfer,
    /// Merge compatible triangles
    Merge,
    /// Stake tokens on a triangle region
    Stake { amount: Decimal },
    /// Claim mining rewards
    ClaimReward { amount: Decimal },
}

/// Geometric proof for triangle operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeometricProof {
    pub triangle_hash: String,
    pub subdivision_valid: bool,
    pub area_conservation: bool,
    pub merkle_root: String,
    pub nonce: u64,
    pub difficulty: u32,
}

/// Block header containing metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    pub previous_hash: String,
    pub merkle_root: String,
    pub timestamp: u64,
    pub nonce: u64,
    pub difficulty: u32,
    pub version: u32,
    pub triangle_count: usize,
    pub total_area: Decimal,
}

/// Complete block in the Sierpinski blockchain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub triangle_transactions: Vec<TriangleTransaction>,
    pub geometric_proof: GeometricProof,
    pub miner_address: String,
    pub block_reward: Decimal,
    pub height: u64,
}

impl TriangleTransaction {
    /// Create a new triangle transaction
    pub fn new(
        from: Option<TriangleAddress>,
        to: TriangleAddress,
        operation: TriangleOperation,
        triangle: Option<Triangle>,
        gas_fee: Decimal,
    ) -> Self {
        TriangleTransaction {
            id: Uuid::new_v4(),
            from_address: from,
            to_address: to,
            operation,
            triangle_data: triangle,
            signature: Vec::new(), // Will be filled by wallet
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            gas_fee,
        }
    }

    /// Calculate transaction hash
    pub fn hash(&self) -> String {
        let mut hasher = blake3::Hasher::new();
        
        hasher.update(self.id.as_bytes());
        hasher.update(&self.timestamp.to_le_bytes());
        
        if let Some(from) = &self.from_address {
            hasher.update(from.to_string().as_bytes());
        }
        
        hasher.update(self.to_address.to_string().as_bytes());
        
        if let Some(triangle) = &self.triangle_data {
            hasher.update(triangle.hash().as_bytes());
        }
        
        hasher.finalize().to_hex().to_string()
    }

    /// Validate transaction structure
    pub fn validate(&self) -> SierpinskiResult<bool> {
        // Check timestamp is reasonable
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        if self.timestamp > now + 3600 { // Not more than 1 hour in future
            return Err(SierpinskiError::validation("Transaction timestamp too far in future"));
        }

        // Validate gas fee
        if self.gas_fee < Decimal::ZERO {
            return Err(SierpinskiError::validation("Gas fee cannot be negative"));
        }

        // Operation-specific validation
        match &self.operation {
            TriangleOperation::Create => {
                if self.triangle_data.is_none() {
                    return Err(SierpinskiError::validation("Create operation requires triangle data"));
                }
            }
            TriangleOperation::Transfer => {
                if self.from_address.is_none() {
                    return Err(SierpinskiError::validation("Transfer requires from address"));
                }
            }
            TriangleOperation::Stake { amount } => {
                if *amount <= Decimal::ZERO {
                    return Err(SierpinskiError::validation("Stake amount must be positive"));
                }
            }
            _ => {}
        }

        Ok(true)
    }
}

impl Block {
    /// Create a new block
    pub fn new(
        previous_hash: String,
        transactions: Vec<TriangleTransaction>,
        miner_address: String,
        difficulty: u32,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let merkle_root = Self::calculate_merkle_root(&transactions);
        let triangle_count = transactions.len();
        let total_area = Self::calculate_total_area(&transactions);
        let block_reward = Self::calculate_block_reward(difficulty, &transactions);

        let header = BlockHeader {
            previous_hash,
            merkle_root: merkle_root.clone(),
            timestamp,
            nonce: 0,
            difficulty,
            version: 1,
            triangle_count,
            total_area,
        };

        let geometric_proof = GeometricProof {
            triangle_hash: Self::calculate_triangle_hash(&transactions),
            subdivision_valid: true, // Will be validated during mining
            area_conservation: true,
            merkle_root,
            nonce: 0,
            difficulty,
        };

        Block {
            header,
            triangle_transactions: transactions,
            geometric_proof,
            miner_address,
            block_reward,
            height: 0, // Will be set by blockchain
        }
    }

    /// Calculate Merkle root of transactions
    fn calculate_merkle_root(transactions: &[TriangleTransaction]) -> String {
        if transactions.is_empty() {
            return "0".repeat(64);
        }

        let mut hashes: Vec<String> = transactions
            .iter()
            .map(|tx| tx.hash())
            .collect();

        while hashes.len() > 1 {
            let mut next_level = Vec::new();
            
            for chunk in hashes.chunks(2) {
                let mut hasher = blake3::Hasher::new();
                hasher.update(chunk[0].as_bytes());
                if chunk.len() > 1 {
                    hasher.update(chunk[1].as_bytes());
                } else {
                    hasher.update(chunk[0].as_bytes()); // Duplicate if odd number
                }
                next_level.push(hasher.finalize().to_hex().to_string());
            }
            
            hashes = next_level;
        }

        hashes[0].clone()
    }

    /// Calculate total area involved in transactions
    fn calculate_total_area(transactions: &[TriangleTransaction]) -> Decimal {
        transactions
            .iter()
            .filter_map(|tx| tx.triangle_data.as_ref())
            .filter_map(|triangle| triangle.area().ok())
            .sum()
    }

    /// Calculate combined hash of all triangle data
    fn calculate_triangle_hash(transactions: &[TriangleTransaction]) -> String {
        let mut hasher = blake3::Hasher::new();
        
        for tx in transactions {
            if let Some(triangle) = &tx.triangle_data {
                hasher.update(triangle.hash().as_bytes());
            }
        }
        
        hasher.finalize().to_hex().to_string()
    }

    /// Calculate block reward based on difficulty and triangle operations
    fn calculate_block_reward(difficulty: u32, transactions: &[TriangleTransaction]) -> Decimal {
        let base_reward = Decimal::new(50, 0); // Base 50 tokens
        let difficulty_multiplier = Decimal::new(difficulty as i64, 0) / Decimal::new(100, 0);
        let transaction_bonus = Decimal::new(transactions.len() as i64, 1); // 0.1 per transaction
        
        base_reward + difficulty_multiplier + transaction_bonus
    }

    /// Calculate block hash
    pub fn hash(&self) -> String {
        let mut hasher = blake3::Hasher::new();
        
        hasher.update(self.header.previous_hash.as_bytes());
        hasher.update(self.header.merkle_root.as_bytes());
        hasher.update(&self.header.timestamp.to_le_bytes());
        hasher.update(&self.header.nonce.to_le_bytes());
        hasher.update(&self.header.difficulty.to_le_bytes());
        hasher.update(self.geometric_proof.triangle_hash.as_bytes());
        
        hasher.finalize().to_hex().to_string()
    }

    /// Validate block structure and proofs
    pub fn validate(&self) -> SierpinskiResult<bool> {
        // Validate all transactions
        for tx in &self.triangle_transactions {
            tx.validate()?;
        }

        // Validate Merkle root
        let calculated_merkle = Self::calculate_merkle_root(&self.triangle_transactions);
        if calculated_merkle != self.header.merkle_root {
            return Err(SierpinskiError::validation("Invalid Merkle root"));
        }

        // Validate geometric proof
        if !self.geometric_proof.subdivision_valid {
            return Err(SierpinskiError::validation("Invalid subdivision proof"));
        }

        // Validate timestamp
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        if self.header.timestamp > now + 7200 { // Not more than 2 hours in future
            return Err(SierpinskiError::validation("Block timestamp too far in future"));
        }

        Ok(true)
    }

    /// Check if block meets difficulty target
    pub fn meets_difficulty_target(&self) -> bool {
        let hash = self.hash();
        let leading_zeros = hash.chars().take_while(|&c| c == '0').count();
        leading_zeros >= self.header.difficulty as usize
    }

    /// Set the nonce (used during mining)
    pub fn set_nonce(&mut self, nonce: u64) {
        self.header.nonce = nonce;
        self.geometric_proof.nonce = nonce;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::geometry::Point;

    fn create_test_transaction() -> TriangleTransaction {
        let triangle = Triangle::new(
            Point::from_f64(0.0, 0.0).unwrap(),
            Point::from_f64(1.0, 0.0).unwrap(),
            Point::from_f64(0.5, 0.866).unwrap(),
        ).unwrap();

        TriangleTransaction::new(
            None,
            TriangleAddress::genesis(),
            TriangleOperation::Create,
            Some(triangle),
            Decimal::new(1, 2), // 0.01 gas fee
        )
    }

    #[test]
    fn test_transaction_creation() {
        let tx = create_test_transaction();
        assert!(!tx.hash().is_empty());
        assert!(tx.validate().unwrap());
    }

    #[test]
    fn test_block_creation() {
        let transactions = vec![create_test_transaction()];
        let block = Block::new(
            "previous_hash".to_string(),
            transactions,
            "miner_address".to_string(),
            4,
        );
        
        assert!(!block.hash().is_empty());
        assert!(block.validate().unwrap());
        assert_eq!(block.header.triangle_count, 1);
    }

    #[test]
    fn test_merkle_root_calculation() {
        let tx1 = create_test_transaction();
        let tx2 = create_test_transaction();
        
        let root1 = Block::calculate_merkle_root(&[tx1.clone()]);
        let root2 = Block::calculate_merkle_root(&[tx1, tx2]);
        
        assert_ne!(root1, root2);
        assert!(!root1.is_empty());
        assert!(!root2.is_empty());
    }
}