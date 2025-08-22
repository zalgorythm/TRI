//! Blockchain implementation for TriadChain cryptocurrency

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;

use crate::core::{
    block::{Block, TriangleTransaction, TriangleOperation},
    fractal::{FractalStructure, FractalTriangle},
    address::TriangleAddress,
    errors::{SierpinskiError, SierpinskiResult},
};

/// The main blockchain structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriadChainBlockchain {
    /// Chain of blocks
    pub blocks: Vec<Block>,
    /// Current fractal state representing all triangle ownership
    pub fractal_state: FractalStructure,
    /// Pending transactions waiting to be mined
    pub mempool: Vec<TriangleTransaction>,
    /// Current mining difficulty
    pub difficulty: u32,
    /// Total tokens in circulation
    pub total_supply: Decimal,
    /// Balance tracking by address
    pub balances: HashMap<String, Decimal>,
    /// Triangle ownership mapping
    pub triangle_owners: HashMap<TriangleAddress, String>,
}

impl TriadChainBlockchain {
    /// Create a new blockchain with genesis block
    pub fn new() -> SierpinskiResult<Self> {
        let mut blockchain = TriadChainBlockchain {
            blocks: Vec::new(),
            fractal_state: FractalStructure::new(),
            mempool: Vec::new(),
            difficulty: 4, // Start with 4 leading zeros
            total_supply: Decimal::ZERO,
            balances: HashMap::new(),
            triangle_owners: HashMap::new(),
        };

        blockchain.create_genesis_block()?;
        Ok(blockchain)
    }

    /// Create the genesis block with initial triangle
    fn create_genesis_block(&mut self) -> SierpinskiResult<()> {
        // Create genesis triangle
        let genesis_triangle = crate::core::genesis::genesis_fractal_triangle()?;
        let genesis_address = genesis_triangle.address.clone();
        
        // Set genesis in fractal state
        self.fractal_state.set_genesis(genesis_triangle.clone())?;

        // Create genesis transaction
        let genesis_tx = TriangleTransaction::new(
            None,
            genesis_address.clone(),
            TriangleOperation::Create,
            Some(genesis_triangle.triangle.clone()),
            Decimal::ZERO, // No gas fee for genesis
        );

        // Create genesis block
        let mut genesis_block = Block::new(
            "0".repeat(64), // Previous hash for genesis is all zeros
            vec![genesis_tx],
            "genesis_miner".to_string(),
            self.difficulty,
        );
        
        genesis_block.height = 0;
        
        // Add initial supply
        let genesis_reward = Decimal::new(1000000, 0); // 1 million initial tokens
        self.total_supply = genesis_reward;
        self.balances.insert("genesis_miner".to_string(), genesis_reward);
        self.triangle_owners.insert(genesis_address, "genesis_miner".to_string());

        self.blocks.push(genesis_block);
        Ok(())
    }

    /// Add a transaction to the mempool
    pub fn add_transaction(&mut self, transaction: TriangleTransaction) -> SierpinskiResult<()> {
        // Validate transaction
        transaction.validate()?;
        
        // Check if sender has sufficient balance for gas fee
        if let Some(from_addr) = &transaction.from_address {
            let from_str = from_addr.to_string();
            let balance = self.balances.get(&from_str).unwrap_or(&Decimal::ZERO);
            
            if *balance < transaction.gas_fee {
                return Err(SierpinskiError::validation("Insufficient balance for gas fee"));
            }
        }

        // Add to mempool
        self.mempool.push(transaction);
        Ok(())
    }

    /// Mine a new block with pending transactions
    pub fn mine_block(&mut self, miner_address: String, max_transactions: usize) -> SierpinskiResult<Block> {
        if self.blocks.is_empty() {
            return Err(SierpinskiError::validation("Cannot mine without genesis block"));
        }

        // Select transactions from mempool
        let transactions: Vec<TriangleTransaction> = self.mempool
            .iter()
            .take(max_transactions)
            .cloned()
            .collect();

        if transactions.is_empty() {
            return Err(SierpinskiError::validation("No transactions to mine"));
        }

        // Get previous block hash
        let previous_hash = self.blocks.last().unwrap().hash();

        // Create new block
        let mut new_block = Block::new(
            previous_hash,
            transactions.clone(),
            miner_address.clone(),
            self.difficulty,
        );
        
        new_block.height = self.blocks.len() as u64;

        // Perform proof-of-work (simplified for demo)
        let mut nonce = 0u64;
        loop {
            new_block.set_nonce(nonce);
            if new_block.meets_difficulty_target() {
                break;
            }
            nonce += 1;
            
            // Prevent infinite loop in demo
            if nonce > 100000 {
                return Err(SierpinskiError::validation("Mining timeout"));
            }
        }

        // Validate block
        new_block.validate()?;

        // Apply block to blockchain state
        self.apply_block(&new_block)?;

        // Remove mined transactions from mempool
        let mined_tx_ids: Vec<_> = transactions.iter().map(|tx| tx.id).collect();
        self.mempool.retain(|tx| !mined_tx_ids.contains(&tx.id));

        // Add block to chain
        self.blocks.push(new_block.clone());

        Ok(new_block)
    }

    /// Apply a block's effects to the blockchain state
    fn apply_block(&mut self, block: &Block) -> SierpinskiResult<()> {
        // Process each transaction
        for transaction in &block.triangle_transactions {
            self.apply_transaction(transaction)?;
        }

        // Award mining reward
        let current_balance = self.balances
            .get(&block.miner_address)
            .unwrap_or(&Decimal::ZERO);
        
        self.balances.insert(
            block.miner_address.clone(),
            current_balance + block.block_reward,
        );
        
        self.total_supply += block.block_reward;

        // Adjust difficulty every 10 blocks
        if block.height % 10 == 0 && block.height > 0 {
            self.adjust_difficulty();
        }

        Ok(())
    }

    /// Apply a transaction's effects
    fn apply_transaction(&mut self, transaction: &TriangleTransaction) -> SierpinskiResult<()> {
        match &transaction.operation {
            TriangleOperation::Create => {
                if let Some(triangle_data) = &transaction.triangle_data {
                    // Create new fractal triangle
                    let fractal_triangle = FractalTriangle::new(
                        triangle_data.clone(),
                        crate::core::state::TriangleState::Active,
                        transaction.to_address.clone(),
                        transaction.to_address.depth(),
                    );

                    self.fractal_state.add_triangle(fractal_triangle)?;
                    
                    // Set ownership
                    if let Some(from_addr) = &transaction.from_address {
                        self.triangle_owners.insert(
                            transaction.to_address.clone(),
                            from_addr.to_string(),
                        );
                    }
                }
            }
            
            TriangleOperation::Subdivide => {
                // Find parent triangle and subdivide it
                if let Some(parent_triangle) = self.fractal_state.get_triangle_mut(&uuid::Uuid::new_v4()) {
                    // Subdivide logic would go here
                    parent_triangle.change_state(crate::core::state::TriangleState::Subdivided)?;
                }
            }
            
            TriangleOperation::Transfer => {
                // Transfer triangle ownership
                if let (Some(from), to) = (&transaction.from_address, &transaction.to_address) {
                    self.triangle_owners.insert(to.clone(), from.to_string());
                }
            }
            
            TriangleOperation::Stake { amount } => {
                // Handle staking
                if let Some(from_addr) = &transaction.from_address {
                    let from_str = from_addr.to_string();
                    let balance = self.balances.get(&from_str).unwrap_or(&Decimal::ZERO);
                    
                    if *balance >= *amount {
                        self.balances.insert(from_str, balance - amount);
                        // Staking logic would track staked amounts
                    }
                }
            }
            
            _ => {} // Handle other operations
        }

        // Deduct gas fees
        if let Some(from_addr) = &transaction.from_address {
            let from_str = from_addr.to_string();
            let balance = self.balances.get(&from_str).unwrap_or(&Decimal::ZERO);
            self.balances.insert(from_str, balance - transaction.gas_fee);
        }

        Ok(())
    }

    /// Adjust mining difficulty based on block times
    fn adjust_difficulty(&mut self) {
        if self.blocks.len() < 10 {
            return;
        }

        let recent_blocks = &self.blocks[self.blocks.len() - 10..];
        let time_span = recent_blocks.last().unwrap().header.timestamp 
                       - recent_blocks.first().unwrap().header.timestamp;

        let target_time = 600; // 10 minutes per block * 10 blocks = 600 seconds
        
        if time_span < target_time / 2 {
            // Too fast, increase difficulty
            self.difficulty = std::cmp::min(self.difficulty + 1, 20);
        } else if time_span > target_time * 2 {
            // Too slow, decrease difficulty
            self.difficulty = std::cmp::max(self.difficulty.saturating_sub(1), 1);
        }
    }

    /// Validate the entire blockchain
    pub fn validate_chain(&self) -> SierpinskiResult<bool> {
        if self.blocks.is_empty() {
            return Err(SierpinskiError::validation("Empty blockchain"));
        }

        // Validate genesis block
        if self.blocks[0].header.previous_hash != "0".repeat(64) {
            return Err(SierpinskiError::validation("Invalid genesis block"));
        }

        // Validate chain links
        for i in 1..self.blocks.len() {
            let prev_hash = self.blocks[i - 1].hash();
            if self.blocks[i].header.previous_hash != prev_hash {
                return Err(SierpinskiError::validation("Broken chain link"));
            }
            
            // Validate individual block
            self.blocks[i].validate()?;
        }

        Ok(true)
    }

    /// Get current blockchain statistics
    pub fn stats(&self) -> BlockchainStats {
        BlockchainStats {
            total_blocks: self.blocks.len(),
            total_transactions: self.blocks.iter().map(|b| b.triangle_transactions.len()).sum(),
            total_supply: self.total_supply,
            current_difficulty: self.difficulty,
            mempool_size: self.mempool.len(),
            total_triangles: self.fractal_state.total_triangles(),
            unique_addresses: self.balances.len(),
        }
    }

    /// Get balance for an address
    pub fn get_balance(&self, address: &str) -> Decimal {
        self.balances.get(address).unwrap_or(&Decimal::ZERO).clone()
    }

    /// Get triangles owned by an address
    pub fn get_owned_triangles(&self, owner: &str) -> Vec<TriangleAddress> {
        self.triangle_owners
            .iter()
            .filter(|(_, addr)| *addr == owner)
            .map(|(triangle_addr, _)| triangle_addr.clone())
            .collect()
    }
}

/// Blockchain statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainStats {
    pub total_blocks: usize,
    pub total_transactions: usize,
    pub total_supply: Decimal,
    pub current_difficulty: u32,
    pub mempool_size: usize,
    pub total_triangles: usize,
    pub unique_addresses: usize,
}

impl Default for TriadChainBlockchain {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blockchain_creation() {
        let blockchain = TriadChainBlockchain::new().unwrap();
        assert_eq!(blockchain.blocks.len(), 1); // Genesis block
        assert!(blockchain.total_supply > Decimal::ZERO);
    }

    #[test]
    fn test_blockchain_validation() {
        let blockchain = TriadChainBlockchain::new().unwrap();
        assert!(blockchain.validate_chain().unwrap());
    }

    #[test]
    fn test_mempool_operations() {
        let mut blockchain = SierpinskiBlockchain::new().unwrap();
        
        let tx = TriangleTransaction::new(
            None,
            TriangleAddress::genesis(),
            TriangleOperation::Create,
            None,
            Decimal::new(1, 2),
        );
        
        blockchain.add_transaction(tx).unwrap();
        assert_eq!(blockchain.mempool.len(), 1);
    }
}