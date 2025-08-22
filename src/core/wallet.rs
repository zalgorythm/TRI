//! Wallet system for managing triangle ownership and transactions

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};

use crate::core::{
    address::TriangleAddress,
    block::{TriangleTransaction, TriangleOperation},
    triangle::Triangle,
    blockchain::TriadChainBlockchain,
    errors::{SierpinskiError, SierpinskiResult},
};

/// Wallet for managing cryptocurrency and triangle ownership
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriadChainWallet {
    /// Wallet identifier
    pub wallet_id: String,
    /// Public key for the wallet
    #[serde(with = "verifying_key_serde")]
    pub public_key: VerifyingKey,
    /// Encrypted private key (in real implementation)
    #[serde(skip_serializing)]
    signing_key: Option<SigningKey>,
    /// Owned triangle addresses
    pub owned_triangles: HashMap<TriangleAddress, TriangleOwnership>,
    /// Transaction history
    pub transaction_history: Vec<String>, // Transaction IDs
    /// Cached balance
    pub balance: Decimal,
    /// Staked amounts
    pub staked_balance: Decimal,
    /// Wallet creation time
    pub created_at: u64,
}

/// Information about owned triangle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriangleOwnership {
    pub address: TriangleAddress,
    pub triangle_data: Option<Triangle>,
    pub acquisition_time: u64,
    pub is_staked: bool,
    pub staked_amount: Decimal,
    pub estimated_value: Decimal,
}

/// Transaction builder for creating signed transactions
#[allow(dead_code)]
pub struct TransactionBuilder {
    wallet: TriadChainWallet,
    gas_price: Decimal,
}

impl TriadChainWallet {
    /// Create a new wallet with generated keypair
    pub fn new() -> SierpinskiResult<Self> {
        let signing_key = SigningKey::from_bytes(&rand::random::<[u8; 32]>());
        let public_key = signing_key.verifying_key();
        
        let wallet_id = Self::derive_wallet_address(&public_key);
        
        Ok(TriadChainWallet {
            wallet_id,
            public_key,
            signing_key: Some(signing_key),
            owned_triangles: HashMap::new(),
            transaction_history: Vec::new(),
            balance: Decimal::ZERO,
            staked_balance: Decimal::ZERO,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
    }

    /// Create wallet from existing signing key (for recovery)
    pub fn from_signing_key(signing_key: SigningKey) -> Self {
        let public_key = signing_key.verifying_key();
        let wallet_id = Self::derive_wallet_address(&public_key);
        
        TriadChainWallet {
            wallet_id,
            public_key,
            signing_key: Some(signing_key),
            owned_triangles: HashMap::new(),
            transaction_history: Vec::new(),
            balance: Decimal::ZERO,
            staked_balance: Decimal::ZERO,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    /// Derive wallet address from public key
    fn derive_wallet_address(public_key: &VerifyingKey) -> String {
        let mut hasher = blake3::Hasher::new();
        hasher.update(public_key.as_bytes());
        let hash = hasher.finalize();
        format!("ST{}", &hash.to_hex()[..32]) // ST prefix for Sierpinski Triangle
    }

    /// Sign a transaction
    pub fn sign_transaction(&self, transaction: &mut TriangleTransaction) -> SierpinskiResult<()> {
        let signing_key = self.signing_key.as_ref()
            .ok_or_else(|| SierpinskiError::validation("Wallet signing key not available"))?;

        // Create message to sign
        let message = format!(
            "{}:{}:{}:{}",
            transaction.id,
            transaction.to_address,
            serde_json::to_string(&transaction.operation).unwrap(),
            transaction.timestamp
        );

        // Sign the message
        let signature = signing_key.sign(message.as_bytes());
        transaction.signature = signature.to_bytes().to_vec();

        Ok(())
    }

    /// Verify a transaction signature
    pub fn verify_transaction_signature(
        transaction: &TriangleTransaction,
        public_key: &VerifyingKey,
    ) -> bool {
        let message = format!(
            "{}:{}:{}:{}",
            transaction.id,
            transaction.to_address,
            serde_json::to_string(&transaction.operation).unwrap(),
            transaction.timestamp
        );

        if transaction.signature.len() == 64 {
            if let Ok(signature_bytes) = transaction.signature.as_slice().try_into() {
                let signature = Signature::from_bytes(signature_bytes);
                return public_key.verify(message.as_bytes(), &signature).is_ok();
            }
        }
        false
    }

    /// Update wallet state from blockchain
    pub fn sync_with_blockchain(&mut self, blockchain: &TriadChainBlockchain) -> SierpinskiResult<()> {
        // Update balance
        self.balance = blockchain.get_balance(&self.wallet_id);

        // Update owned triangles
        let owned_addresses = blockchain.get_owned_triangles(&self.wallet_id);
        
        for address in owned_addresses {
            if !self.owned_triangles.contains_key(&address) {
                // Get triangle data from fractal state
                let triangle_data = blockchain.fractal_state
                    .triangles_at_depth(address.depth())
                    .iter()
                    .find(|t| t.address == address)
                    .map(|t| t.triangle.clone());

                let ownership = TriangleOwnership {
                    address: address.clone(),
                    triangle_data,
                    acquisition_time: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    is_staked: false,
                    staked_amount: Decimal::ZERO,
                    estimated_value: self.estimate_triangle_value(&address, blockchain),
                };

                self.owned_triangles.insert(address, ownership);
            }
        }

        Ok(())
    }

    /// Estimate the value of a triangle based on its properties
    fn estimate_triangle_value(&self, address: &TriangleAddress, _blockchain: &TriadChainBlockchain) -> Decimal {
        // Value increases with depth (rarity) and decreases with age
        let depth_multiplier = {
            let mut result = Decimal::new(2, 0);
            for _ in 0..address.depth() {
                result *= Decimal::new(2, 0);
            }
            result
        };
        let base_value = Decimal::new(10, 0); // 10 tokens base
        
        // Area-based value (smaller triangles are more valuable)
        let area_multiplier = if let Some(triangle) = self.owned_triangles.get(address)
            .and_then(|ownership| ownership.triangle_data.as_ref()) 
        {
            if let Ok(area) = triangle.area() {
                Decimal::ONE / (area + Decimal::new(1, 3)) // Prevent division by zero
            } else {
                Decimal::ONE
            }
        } else {
            Decimal::ONE
        };

        base_value * depth_multiplier * area_multiplier
    }

    /// Create a transaction to transfer triangle ownership
    pub fn create_transfer_transaction(
        &self,
        to_address: &str,
        triangle_address: TriangleAddress,
        gas_fee: Decimal,
    ) -> SierpinskiResult<TriangleTransaction> {
        // Check if we own this triangle
        if !self.owned_triangles.contains_key(&triangle_address) {
            return Err(SierpinskiError::validation("Triangle not owned by this wallet"));
        }

        // Check sufficient balance for gas
        if self.balance < gas_fee {
            return Err(SierpinskiError::validation("Insufficient balance for gas fee"));
        }

        let mut transaction = TriangleTransaction::new(
            Some(triangle_address.clone()),
            TriangleAddress::from_string_representation(to_address)?,
            TriangleOperation::Transfer,
            self.owned_triangles.get(&triangle_address)
                .and_then(|ownership| ownership.triangle_data.clone()),
            gas_fee,
        );

        // Sign the transaction
        self.sign_transaction(&mut transaction)?;

        Ok(transaction)
    }

    /// Create a staking transaction
    pub fn create_stake_transaction(
        &self,
        triangle_address: TriangleAddress,
        stake_amount: Decimal,
        gas_fee: Decimal,
    ) -> SierpinskiResult<TriangleTransaction> {
        // Check ownership and sufficient balance
        if !self.owned_triangles.contains_key(&triangle_address) {
            return Err(SierpinskiError::validation("Triangle not owned by this wallet"));
        }

        if self.balance < stake_amount + gas_fee {
            return Err(SierpinskiError::validation("Insufficient balance for stake and gas"));
        }

        let mut transaction = TriangleTransaction::new(
            Some(triangle_address.clone()),
            triangle_address,
            TriangleOperation::Stake { amount: stake_amount },
            None,
            gas_fee,
        );

        self.sign_transaction(&mut transaction)?;
        Ok(transaction)
    }

    /// Create a subdivision transaction (mining)
    pub fn create_subdivision_transaction(
        &self,
        triangle_address: TriangleAddress,
        gas_fee: Decimal,
    ) -> SierpinskiResult<TriangleTransaction> {
        // Check ownership
        let triangle_data = self.owned_triangles.get(&triangle_address)
            .ok_or_else(|| SierpinskiError::validation("Triangle not owned by this wallet"))?
            .triangle_data.clone();

        if self.balance < gas_fee {
            return Err(SierpinskiError::validation("Insufficient balance for gas fee"));
        }

        let mut transaction = TriangleTransaction::new(
            Some(triangle_address.clone()),
            triangle_address,
            TriangleOperation::Subdivide,
            triangle_data,
            gas_fee,
        );

        self.sign_transaction(&mut transaction)?;
        Ok(transaction)
    }

    /// Get wallet statistics
    pub fn get_stats(&self) -> WalletStats {
        let total_triangles = self.owned_triangles.len();
        let staked_triangles = self.owned_triangles.values()
            .filter(|ownership| ownership.is_staked)
            .count();
        
        let estimated_portfolio_value: Decimal = self.owned_triangles.values()
            .map(|ownership| ownership.estimated_value)
            .sum();

        WalletStats {
            wallet_id: self.wallet_id.clone(),
            total_balance: self.balance,
            staked_balance: self.staked_balance,
            available_balance: self.balance - self.staked_balance,
            total_triangles,
            staked_triangles,
            estimated_portfolio_value,
            transaction_count: self.transaction_history.len(),
        }
    }

    /// Export wallet (without private key)
    pub fn export_public(&self) -> PublicWalletData {
        PublicWalletData {
            wallet_id: self.wallet_id.clone(),
            public_key: self.public_key,
            owned_triangles: self.owned_triangles.clone(),
            balance: self.balance,
            created_at: self.created_at,
        }
    }
}

/// Public wallet data for sharing/display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicWalletData {
    pub wallet_id: String,
    #[serde(with = "verifying_key_serde")]
    pub public_key: VerifyingKey,
    pub owned_triangles: HashMap<TriangleAddress, TriangleOwnership>,
    pub balance: Decimal,
    pub created_at: u64,
}

/// Wallet statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletStats {
    pub wallet_id: String,
    pub total_balance: Decimal,
    pub staked_balance: Decimal,
    pub available_balance: Decimal,
    pub total_triangles: usize,
    pub staked_triangles: usize,
    pub estimated_portfolio_value: Decimal,
    pub transaction_count: usize,
}

/// Serde helper for VerifyingKey
mod verifying_key_serde {
    use super::*;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(key: &VerifyingKey, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        key.as_bytes().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<VerifyingKey, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes: [u8; 32] = Deserialize::deserialize(deserializer)?;
        VerifyingKey::from_bytes(&bytes).map_err(serde::de::Error::custom)
    }
}

impl Default for TriadChainWallet {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_creation() {
        let wallet = TriadChainWallet::new().unwrap();
        assert!(!wallet.wallet_id.is_empty());
        assert!(wallet.wallet_id.starts_with("ST"));
        assert_eq!(wallet.balance, Decimal::ZERO);
    }

    #[test]
    fn test_transaction_signing() {
        let wallet = TriadChainWallet::new().unwrap();
        
        let mut transaction = TriangleTransaction::new(
            None,
            TriangleAddress::genesis(),
            TriangleOperation::Create,
            None,
            Decimal::new(1, 2),
        );

        wallet.sign_transaction(&mut transaction).unwrap();
        assert!(!transaction.signature.is_empty());
        
        // Verify signature
        assert!(TriadChainWallet::verify_transaction_signature(&transaction, &wallet.public_key));
    }

    #[test]
    fn test_wallet_stats() {
        let wallet = TriadChainWallet::new().unwrap();
        let stats = wallet.get_stats();
        
        assert_eq!(stats.total_triangles, 0);
        assert_eq!(stats.total_balance, Decimal::ZERO);
    }
}