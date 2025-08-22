//! Token economics and area-based value system for Sierpinski Triangle cryptocurrency

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;

use crate::core::{
    address::TriangleAddress,
    triangle::Triangle,
    fractal::FractalStructure,
    blockchain::TriadChainBlockchain,
    errors::{SierpinskiError, SierpinskiResult},
};

/// Token economics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenEconomics {
    /// Base token supply
    pub initial_supply: Decimal,
    /// Maximum possible supply
    pub max_supply: Decimal,
    /// Current circulating supply
    pub circulating_supply: Decimal,
    /// Inflation rate per block
    pub block_inflation_rate: Decimal,
    /// Deflation rate from subdivisions
    pub subdivision_deflation_rate: Decimal,
    /// Area-based value multipliers
    pub area_value_curve: AreaValueCurve,
}

/// Area-based value calculation curve
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AreaValueCurve {
    /// Base value per unit area
    pub base_value_per_area: Decimal,
    /// Depth multiplier (smaller/deeper triangles are more valuable)
    pub depth_multiplier: Decimal,
    /// Rarity bonus for unique triangle properties
    pub rarity_bonus: Decimal,
    /// Age factor (older triangles may be more/less valuable)
    pub age_factor: Decimal,
}

/// Triangle value assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriangleValue {
    pub address: TriangleAddress,
    pub base_area_value: Decimal,
    pub depth_bonus: Decimal,
    pub rarity_bonus: Decimal,
    pub age_factor: Decimal,
    pub total_estimated_value: Decimal,
    pub market_liquidity: Decimal,
}

/// Staking economics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingPool {
    pub triangle_address: TriangleAddress,
    pub total_staked: Decimal,
    pub staking_reward_rate: Decimal,
    pub minimum_stake: Decimal,
    pub lock_period: u64, // in seconds
    pub participants: HashMap<String, StakePosition>,
}

/// Individual stake position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakePosition {
    pub staker_address: String,
    pub amount_staked: Decimal,
    pub stake_timestamp: u64,
    pub lock_expires: u64,
    pub accumulated_rewards: Decimal,
}

/// Triangle rental economics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriangleRental {
    pub triangle_address: TriangleAddress,
    pub owner_address: String,
    pub rental_rate_per_block: Decimal,
    pub minimum_rental_period: u64,
    pub current_renter: Option<String>,
    pub rental_start_block: u64,
    pub rental_end_block: u64,
    pub security_deposit: Decimal,
}

/// Main economics engine
pub struct EconomicsEngine {
    pub config: TokenEconomics,
    pub staking_pools: HashMap<TriangleAddress, StakingPool>,
    pub rentals: HashMap<TriangleAddress, TriangleRental>,
    pub market_prices: HashMap<TriangleAddress, Decimal>,
}

impl EconomicsEngine {
    /// Create new economics engine
    pub fn new() -> Self {
        let config = TokenEconomics {
            initial_supply: Decimal::new(1_000_000, 0), // 1 million tokens
            max_supply: Decimal::new(21_000_000, 0),    // 21 million max (like Bitcoin)
            circulating_supply: Decimal::new(1_000_000, 0),
            block_inflation_rate: Decimal::new(5, 2), // 0.05% per block
            subdivision_deflation_rate: Decimal::new(1, 2), // 0.01% per subdivision
            area_value_curve: AreaValueCurve {
                base_value_per_area: Decimal::new(100, 0), // 100 tokens per unit area
                depth_multiplier: Decimal::new(2, 0),      // 2x multiplier per depth level
                rarity_bonus: Decimal::new(10, 1),         // Up to 1.0 bonus for rare properties
                age_factor: Decimal::new(1, 3),            // 0.001 bonus per day age
            },
        };

        EconomicsEngine {
            config,
            staking_pools: HashMap::new(),
            rentals: HashMap::new(),
            market_prices: HashMap::new(),
        }
    }

    /// Calculate the intrinsic value of a triangle
    pub fn calculate_triangle_value(&self, 
        triangle: &Triangle, 
        address: &TriangleAddress,
        creation_time: u64
    ) -> SierpinskiResult<TriangleValue> {
        // Base value from area
        let area = triangle.area()?;
        let base_area_value = area * self.config.area_value_curve.base_value_per_area;

        // Depth bonus (exponential increase with depth)
        let depth = address.depth();
        let depth_bonus = base_area_value * {
            let mut multiplier = Decimal::ONE;
            for _ in 0..depth {
                multiplier *= self.config.area_value_curve.depth_multiplier;
            }
            multiplier
        };

        // Rarity bonus based on triangle properties
        let rarity_bonus = self.calculate_rarity_bonus(triangle, address)?;

        // Age factor
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let age_days = (current_time - creation_time) / 86400; // seconds to days
        let age_factor = self.config.area_value_curve.age_factor * Decimal::from(age_days);

        // Total value calculation
        let total_estimated_value = base_area_value + depth_bonus + rarity_bonus + age_factor;

        // Market liquidity factor (how easy it is to trade this triangle)
        let market_liquidity = self.calculate_liquidity_factor(address);

        Ok(TriangleValue {
            address: address.clone(),
            base_area_value,
            depth_bonus,
            rarity_bonus,
            age_factor,
            total_estimated_value,
            market_liquidity,
        })
    }

    /// Calculate rarity bonus for special triangle properties
    fn calculate_rarity_bonus(&self, triangle: &Triangle, address: &TriangleAddress) -> SierpinskiResult<Decimal> {
        let mut bonus = Decimal::ZERO;

        // Perfect equilateral triangles get bonus
        if triangle.is_equilateral()? {
            bonus += self.config.area_value_curve.rarity_bonus * Decimal::new(2, 1); // 0.2 bonus
        }

        // Triangles at special positions get bonus
        if address.is_genesis() {
            bonus += self.config.area_value_curve.rarity_bonus * Decimal::new(5, 1); // 0.5 bonus
        }

        // Void triangles get different valuation
        if address.is_void() {
            bonus += self.config.area_value_curve.rarity_bonus * Decimal::new(15, 2); // 0.15 bonus
        }

        // Triangles with special address patterns
        let address_str = address.to_string();
        if address_str.contains("000") || address_str.contains("111") {
            bonus += self.config.area_value_curve.rarity_bonus * Decimal::new(1, 1); // 0.1 bonus
        }

        Ok(bonus)
    }

    /// Calculate liquidity factor based on trading activity
    fn calculate_liquidity_factor(&self, address: &TriangleAddress) -> Decimal {
        // Higher depth = lower liquidity (harder to find buyers)
        let depth_penalty = Decimal::new(depth_penalty_factor(address.depth()), 2);
        
        // Genesis and low-depth triangles are more liquid
        if address.is_genesis() || address.depth() <= 2 {
            Decimal::new(95, 2) // 95% liquidity
        } else {
            (Decimal::ONE - depth_penalty).max(Decimal::new(10, 2)) // At least 10% liquidity
        }
    }

    /// Create a staking pool for a triangle
    pub fn create_staking_pool(&mut self, 
        triangle_address: TriangleAddress,
        reward_rate: Decimal,
        minimum_stake: Decimal
    ) -> SierpinskiResult<()> {
        if self.staking_pools.contains_key(&triangle_address) {
            return Err(SierpinskiError::validation("Staking pool already exists for this triangle"));
        }

        let pool = StakingPool {
            triangle_address: triangle_address.clone(),
            total_staked: Decimal::ZERO,
            staking_reward_rate: reward_rate,
            minimum_stake,
            lock_period: 7 * 24 * 3600, // 7 days default lock
            participants: HashMap::new(),
        };

        self.staking_pools.insert(triangle_address, pool);
        Ok(())
    }

    /// Stake tokens in a triangle pool
    pub fn stake_tokens(&mut self,
        triangle_address: &TriangleAddress,
        staker_address: String,
        amount: Decimal
    ) -> SierpinskiResult<()> {
        let pool = self.staking_pools.get_mut(triangle_address)
            .ok_or_else(|| SierpinskiError::validation("Staking pool not found"))?;

        if amount < pool.minimum_stake {
            return Err(SierpinskiError::validation("Amount below minimum stake"));
        }

        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let stake_position = StakePosition {
            staker_address: staker_address.clone(),
            amount_staked: amount,
            stake_timestamp: current_time,
            lock_expires: current_time + pool.lock_period,
            accumulated_rewards: Decimal::ZERO,
        };

        pool.participants.insert(staker_address, stake_position);
        pool.total_staked += amount;

        Ok(())
    }

    /// Calculate staking rewards for a position
    pub fn calculate_staking_rewards(&self,
        triangle_address: &TriangleAddress,
        staker_address: &str
    ) -> SierpinskiResult<Decimal> {
        let pool = self.staking_pools.get(triangle_address)
            .ok_or_else(|| SierpinskiError::validation("Staking pool not found"))?;

        let position = pool.participants.get(staker_address)
            .ok_or_else(|| SierpinskiError::validation("Stake position not found"))?;

        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let staking_duration = current_time - position.stake_timestamp;
        let reward_periods = Decimal::from(staking_duration / 3600); // Hourly rewards

        let rewards = position.amount_staked * pool.staking_reward_rate * reward_periods;
        Ok(rewards)
    }

    /// Create triangle rental listing
    pub fn create_rental(&mut self,
        triangle_address: TriangleAddress,
        owner_address: String,
        rental_rate: Decimal
    ) -> SierpinskiResult<()> {
        if self.rentals.contains_key(&triangle_address) {
            return Err(SierpinskiError::validation("Triangle already listed for rental"));
        }

        let rental = TriangleRental {
            triangle_address: triangle_address.clone(),
            owner_address,
            rental_rate_per_block: rental_rate,
            minimum_rental_period: 100, // 100 blocks minimum
            current_renter: None,
            rental_start_block: 0,
            rental_end_block: 0,
            security_deposit: rental_rate * Decimal::new(10, 0), // 10x rate as deposit
        };

        self.rentals.insert(triangle_address, rental);
        Ok(())
    }

    /// Update token supply after block mining
    pub fn update_supply_after_block(&mut self, 
        new_triangles_created: u32,
        subdivisions_performed: u32
    ) -> SierpinskiResult<()> {
        // Add inflation from block rewards
        let inflation = self.config.circulating_supply * self.config.block_inflation_rate;
        
        // Subtract deflation from subdivisions (tokens burned)
        let deflation = Decimal::from(subdivisions_performed) * 
            self.config.circulating_supply * self.config.subdivision_deflation_rate;

        // Update circulating supply
        let new_supply = self.config.circulating_supply + inflation - deflation;
        self.config.circulating_supply = new_supply.min(self.config.max_supply);

        Ok(())
    }

    /// Get economics statistics
    pub fn get_economics_stats(&self) -> EconomicsStats {
        EconomicsStats {
            circulating_supply: self.config.circulating_supply,
            max_supply: self.config.max_supply,
            inflation_rate: self.config.block_inflation_rate,
            deflation_rate: self.config.subdivision_deflation_rate,
            active_staking_pools: self.staking_pools.len(),
            total_staked_value: self.staking_pools.values()
                .map(|pool| pool.total_staked)
                .sum(),
            active_rentals: self.rentals.len(),
            average_triangle_value: self.calculate_average_triangle_value(),
        }
    }

    /// Calculate average triangle value across known triangles
    fn calculate_average_triangle_value(&self) -> Decimal {
        if self.market_prices.is_empty() {
            return Decimal::ZERO;
        }

        let total_value: Decimal = self.market_prices.values().sum();
        total_value / Decimal::from(self.market_prices.len())
    }
}

/// Helper function for depth penalty calculation
fn depth_penalty_factor(depth: u8) -> i64 {
    match depth {
        0..=2 => 5,   // 5% penalty
        3..=5 => 15,  // 15% penalty
        6..=8 => 30,  // 30% penalty
        _ => 50,      // 50% penalty for very deep triangles
    }
}

/// Economics statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicsStats {
    pub circulating_supply: Decimal,
    pub max_supply: Decimal,
    pub inflation_rate: Decimal,
    pub deflation_rate: Decimal,
    pub active_staking_pools: usize,
    pub total_staked_value: Decimal,
    pub active_rentals: usize,
    pub average_triangle_value: Decimal,
}

impl Default for EconomicsEngine {
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
    fn test_economics_engine_creation() {
        let engine = EconomicsEngine::new();
        assert!(engine.config.initial_supply > Decimal::ZERO);
        assert!(engine.config.max_supply > engine.config.initial_supply);
    }

    #[test]
    fn test_triangle_value_calculation() {
        let engine = EconomicsEngine::new();
        let triangle = create_test_triangle();
        let address = TriangleAddress::genesis();
        
        let value = engine.calculate_triangle_value(&triangle, &address, 0).unwrap();
        assert!(value.total_estimated_value > Decimal::ZERO);
        assert!(value.base_area_value > Decimal::ZERO);
    }

    #[test]
    fn test_staking_pool_creation() {
        let mut engine = EconomicsEngine::new();
        let address = TriangleAddress::genesis();
        
        let result = engine.create_staking_pool(
            address.clone(),
            Decimal::new(5, 2), // 5% APR
            Decimal::new(100, 0) // 100 token minimum
        );
        
        assert!(result.is_ok());
        assert!(engine.staking_pools.contains_key(&address));
    }
}