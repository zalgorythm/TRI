//! Geometric proof-of-work mining engine for Sierpinski Triangle cryptocurrency

use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;

use crate::core::{
    block::{Block, TriangleTransaction, TriangleOperation, GeometricProof},
    blockchain::TriadChainBlockchain,
    fractal::{FractalTriangle, FractalStructure},
    subdivision::{subdivide_triangle, SubdivisionResult, validate_subdivision},
    triangle::Triangle,
    address::TriangleAddress,
    geometry::Point,
    errors::{SierpinskiError, SierpinskiResult},
};

/// Mining challenge based on geometric operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeometricChallenge {
    pub target_triangle: Triangle,
    pub difficulty: u32,
    pub required_subdivisions: u8,
    pub area_constraint: Option<Decimal>,
    pub timestamp: u64,
    pub challenge_id: String,
}

/// Result of a geometric mining operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningResult {
    pub nonce: u64,
    pub subdivision_proof: SubdivisionResult,
    pub geometric_hash: String,
    pub computation_time: Duration,
    pub triangles_generated: usize,
    pub total_area_preserved: bool,
}

/// Mining configuration and settings
#[derive(Debug, Clone)]
pub struct MinerConfig {
    pub miner_id: String,
    pub max_threads: usize,
    pub target_block_time: Duration,
    pub max_nonce: u64,
    pub geometric_precision: u32,
}

impl Default for MinerConfig {
    fn default() -> Self {
        MinerConfig {
            miner_id: format!("miner_{}", uuid::Uuid::new_v4()),
            max_threads: num_cpus::get(),
            target_block_time: Duration::from_secs(60), // 1 minute blocks
            max_nonce: 1_000_000,
            geometric_precision: 10,
        }
    }
}

/// Main mining engine
pub struct GeometricMiner {
    config: MinerConfig,
    is_mining: Arc<AtomicBool>,
    current_challenge: Option<GeometricChallenge>,
    hashrate: f64,
}

impl GeometricMiner {
    /// Create a new geometric miner
    pub fn new(config: MinerConfig) -> Self {
        GeometricMiner {
            config,
            is_mining: Arc::new(AtomicBool::new(false)),
            current_challenge: None,
            hashrate: 0.0,
        }
    }

    /// Start mining process
    pub fn start_mining(
        &mut self,
        blockchain: Arc<Mutex<TriadChainBlockchain>>,
        reward_address: String,
    ) -> SierpinskiResult<()> {
        self.is_mining.store(true, Ordering::Relaxed);
        
        let is_mining = Arc::clone(&self.is_mining);
        let config = self.config.clone();
        
        // Spawn mining thread
        thread::spawn(move || {
            let mut nonce = 0u64;
            let mut last_stats = Instant::now();
            let mut operations_count = 0u64;
            
            while is_mining.load(Ordering::Relaxed) {
                // Get current mining target
                let (challenge, transactions) = {
                    let blockchain_guard = blockchain.lock().unwrap();
                    let challenge = Self::generate_challenge(&blockchain_guard, config.geometric_precision);
                    let transactions = blockchain_guard.mempool.clone();
                    (challenge, transactions)
                };
                
                if transactions.is_empty() {
                    thread::sleep(Duration::from_secs(1));
                    continue;
                }
                
                // Attempt to mine block
                match Self::mine_geometric_block(
                    &challenge,
                    &transactions,
                    &reward_address,
                    nonce,
                    config.max_nonce,
                ) {
                    Ok(block) => {
                        // Successfully mined block
                        let mut blockchain_guard = blockchain.lock().unwrap();
                        match blockchain_guard.mine_block(reward_address.clone(), transactions.len()) {
                            Ok(mined_block) => {
                                println!("✅ Block mined! Height: {}, Hash: {}", 
                                        mined_block.height, 
                                        mined_block.hash()[..16].to_string());
                            }
                            Err(e) => {
                                println!("❌ Failed to add block to chain: {}", e);
                            }
                        }
                        nonce = 0; // Reset nonce for next block
                    }
                    Err(_) => {
                        nonce = nonce.wrapping_add(1);
                        operations_count += 1;
                        
                        // Print hashrate stats every 10 seconds
                        if last_stats.elapsed() >= Duration::from_secs(10) {
                            let hashrate = operations_count as f64 / last_stats.elapsed().as_secs_f64();
                            println!("⛏️  Mining... Hashrate: {:.2} H/s, Nonce: {}", hashrate, nonce);
                            operations_count = 0;
                            last_stats = Instant::now();
                        }
                    }
                }
                
                // Small delay to prevent excessive CPU usage in demo
                thread::sleep(Duration::from_millis(1));
            }
        });
        
        Ok(())
    }

    /// Stop mining
    pub fn stop_mining(&mut self) {
        self.is_mining.store(false, Ordering::Relaxed);
    }

    /// Generate a geometric mining challenge
    fn generate_challenge(blockchain: &TriadChainBlockchain, precision: u32) -> GeometricChallenge {
        // Use the latest block's geometry as basis for challenge
        let latest_block = blockchain.blocks.last().unwrap();
        
        // Create challenge triangle based on current fractal state
        let target_triangle = if let Some(genesis) = blockchain.fractal_state.genesis() {
            genesis.triangle.clone()
        } else {
            // Fallback triangle
            Triangle::new(
                Point::from_f64(0.0, 0.0).unwrap(),
                Point::from_f64(1.0, 0.0).unwrap(),
                Point::from_f64(0.5, 0.866).unwrap(),
            ).unwrap()
        };

        let challenge_id = format!("{}-{}", 
                                  latest_block.hash()[..8].to_string(),
                                  SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs());

        GeometricChallenge {
            target_triangle,
            difficulty: blockchain.difficulty,
            required_subdivisions: std::cmp::min(blockchain.difficulty / 2, 10) as u8,
            area_constraint: Some(Decimal::new(1, precision)),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            challenge_id,
        }
    }

    /// Attempt to mine a block using geometric proof-of-work
    fn mine_geometric_block(
        challenge: &GeometricChallenge,
        transactions: &[TriangleTransaction],
        miner_address: &str,
        start_nonce: u64,
        max_iterations: u64,
    ) -> SierpinskiResult<Block> {
        let start_time = Instant::now();
        
        for nonce_offset in 0..max_iterations {
            let nonce = start_nonce.wrapping_add(nonce_offset);
            
            // Create candidate block
            let mut block = Block::new(
                "previous_hash".to_string(), // Will be set properly in real implementation
                transactions.to_vec(),
                miner_address.to_string(),
                challenge.difficulty,
            );
            
            block.set_nonce(nonce);
            
            // Perform geometric proof-of-work
            match Self::verify_geometric_work(challenge, &block, nonce) {
                Ok(mining_result) => {
                    if mining_result.total_area_preserved && mining_result.triangles_generated > 0 {
                        // Update block with geometric proof
                        block.geometric_proof = GeometricProof {
                            triangle_hash: mining_result.geometric_hash,
                            subdivision_valid: true,
                            area_conservation: mining_result.total_area_preserved,
                            merkle_root: block.header.merkle_root.clone(),
                            nonce,
                            difficulty: challenge.difficulty,
                        };
                        
                        // Check if block meets difficulty target
                        if block.meets_difficulty_target() {
                            return Ok(block);
                        }
                    }
                }
                Err(_) => {
                    // Invalid geometric proof, continue with next nonce
                    continue;
                }
            }
        }
        
        Err(SierpinskiError::subdivision("Failed to find valid geometric proof".to_string()))
    }

    /// Verify geometric proof-of-work
    fn verify_geometric_work(
        challenge: &GeometricChallenge,
        block: &Block,
        nonce: u64,
    ) -> SierpinskiResult<MiningResult> {
        let start_time = Instant::now();
        
        // Create fractal triangle from challenge
        let fractal_triangle = FractalTriangle::new(
            challenge.target_triangle.clone(),
            crate::core::state::TriangleState::Active,
            TriangleAddress::genesis(),
            0,
        );
        
        // Perform subdivision as proof-of-work
        let subdivision_result = subdivide_triangle(&fractal_triangle)?;
        
        // Validate subdivision
        let is_valid = validate_subdivision(&subdivision_result)?;
        if !is_valid {
            return Err(SierpinskiError::validation("Invalid subdivision proof"));
        }
        
        // Calculate geometric hash incorporating nonce
        let geometric_hash = Self::calculate_geometric_hash(&subdivision_result, nonce);
        
        // Check area conservation
        let parent_area = subdivision_result.parent.area()?;
        let children_area: Decimal = subdivision_result.children
            .iter()
            .map(|child| child.area().unwrap_or(Decimal::ZERO))
            .sum();
        let void_area = subdivision_result.void_triangle.area()?;
        
        let total_area_preserved = (parent_area - (children_area + void_area)).abs() < Decimal::new(1, 10);
        
        Ok(MiningResult {
            nonce,
            subdivision_proof: subdivision_result,
            geometric_hash,
            computation_time: start_time.elapsed(),
            triangles_generated: 4, // 3 children + 1 void
            total_area_preserved,
        })
    }

    /// Calculate hash that incorporates geometric properties
    fn calculate_geometric_hash(subdivision: &SubdivisionResult, nonce: u64) -> String {
        let mut hasher = blake3::Hasher::new();
        
        // Hash parent triangle
        hasher.update(subdivision.parent.hash().as_bytes());
        
        // Hash children triangles
        for child in &subdivision.children {
            hasher.update(child.hash().as_bytes());
        }
        
        // Hash void triangle
        hasher.update(subdivision.void_triangle.hash().as_bytes());
        
        // Include nonce
        hasher.update(&nonce.to_le_bytes());
        
        hasher.finalize().to_hex().to_string()
    }

    /// Get current mining statistics
    pub fn get_stats(&self) -> MiningStats {
        MiningStats {
            is_mining: self.is_mining.load(Ordering::Relaxed),
            miner_id: self.config.miner_id.clone(),
            hashrate: self.hashrate,
            threads: self.config.max_threads,
        }
    }
}

/// Mining statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningStats {
    pub is_mining: bool,
    pub miner_id: String,
    pub hashrate: f64,
    pub threads: usize,
}

/// Mining pool for collaborative mining (future enhancement)
pub struct MiningPool {
    pub pool_id: String,
    pub miners: Vec<String>,
    pub total_hashrate: f64,
    pub reward_distribution: HashMap<String, Decimal>,
}

use std::collections::HashMap;

impl MiningPool {
    pub fn new(pool_id: String) -> Self {
        MiningPool {
            pool_id,
            miners: Vec::new(),
            total_hashrate: 0.0,
            reward_distribution: HashMap::new(),
        }
    }

    pub fn add_miner(&mut self, miner_id: String, hashrate: f64) {
        self.miners.push(miner_id.clone());
        self.total_hashrate += hashrate;
        self.reward_distribution.insert(miner_id, Decimal::ZERO);
    }

    pub fn distribute_rewards(&mut self, total_reward: Decimal) {
        if self.total_hashrate == 0.0 {
            return;
        }

        for miner_id in &self.miners {
            // In a real implementation, we'd track each miner's contribution
            let share = total_reward / Decimal::try_from(self.miners.len()).unwrap();
            self.reward_distribution.insert(miner_id.clone(), share);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_miner_creation() {
        let config = MinerConfig::default();
        let miner = GeometricMiner::new(config);
        assert!(!miner.is_mining.load(Ordering::Relaxed));
    }

    #[test]
    fn test_geometric_challenge_generation() {
        let blockchain = TriadChainBlockchain::new().unwrap();
        let challenge = GeometricMiner::generate_challenge(&blockchain, 10);
        
        assert!(!challenge.challenge_id.is_empty());
        assert!(challenge.difficulty > 0);
    }

    #[test]
    fn test_mining_pool() {
        let mut pool = MiningPool::new("test_pool".to_string());
        pool.add_miner("miner1".to_string(), 100.0);
        pool.add_miner("miner2".to_string(), 200.0);
        
        assert_eq!(pool.miners.len(), 2);
        assert_eq!(pool.total_hashrate, 300.0);
    }
}