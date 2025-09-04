//! Validator configuration management

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

/// Validator node configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorConfig {
    /// Data directory for validator storage
    pub data_dir: PathBuf,
    
    /// Network configuration
    pub network: NetworkConfig,
    
    /// P2P networking configuration
    pub p2p: P2PConfig,
    
    /// RPC server configuration
    pub rpc: RpcConfig,
    
    /// Validator-specific settings
    pub validator: ValidatorSettings,
    
    /// OON computational services settings
    pub oon: OonConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Network identifier (mainnet, testnet, devnet)
    pub name: String,
    /// Network ID for protocol identification
    pub id: u64,
    /// Genesis block hash for network validation
    pub genesis_hash: String,
    /// Chain specification
    pub chain_spec: ChainSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainSpec {
    /// Commerce layer block time in seconds
    pub commerce_block_time: u64,
    /// Security layer block time in seconds  
    pub security_block_time: u64,
    /// Minimum validator stake in OGT
    pub min_validator_stake: u64,
    /// Maximum number of validators
    pub max_validators: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PConfig {
    /// P2P listening port
    pub port: u16,
    /// Maximum number of peers
    pub max_peers: usize,
    /// Bootstrap peer addresses
    pub bootstrap_peers: Vec<String>,
    /// Connection timeout in seconds
    #[serde(with = "duration_serde")]
    pub connection_timeout: Duration,
    /// Enable mDNS discovery
    pub enable_mdns: bool,
    /// Enable Kademlia DHT
    pub enable_kad: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcConfig {
    /// RPC server listening port
    pub port: u16,
    /// Bind address for RPC server
    pub bind_address: String,
    /// Enable HTTP RPC
    pub enable_http: bool,
    /// Enable WebSocket RPC
    pub enable_ws: bool,
    /// Maximum number of concurrent RPC connections
    pub max_connections: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorSettings {
    /// Enable validator mode
    pub is_validator: bool,
    /// Validator stake amount in OGT
    pub validator_stake: u64,
    /// Validator identity key path
    pub validator_key_path: Option<PathBuf>,
    /// Network identity key path
    pub network_key_path: Option<PathBuf>,
    /// Enable automatic re-staking of rewards
    pub auto_restake: bool,
    /// Slashing protection settings
    pub slashing_protection: SlashingProtectionConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlashingProtectionConfig {
    /// Enable slashing protection
    pub enabled: bool,
    /// Minimum attestation source epoch difference
    pub min_source_epoch_diff: u64,
    /// Minimum attestation target epoch difference
    pub min_target_epoch_diff: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OonConfig {
    /// Enable OON computational services
    pub enable_oon: bool,
    /// Maximum computational jobs to accept
    pub max_concurrent_jobs: usize,
    /// Computational resource allocation (percentage)
    pub resource_allocation: f64,
    /// Supported computation types
    pub supported_services: Vec<String>,
    /// Revenue sharing percentage with validators
    pub revenue_share_percentage: f64,
}

mod duration_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_secs())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}

impl ValidatorConfig {
    /// Create a new configuration for the specified network
    pub fn new_for_network(network_name: &str) -> Result<Self> {
        let (network_id, genesis_hash, chain_spec, bootstrap_peers) = match network_name {
            "mainnet" => (
                1,
                "0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
                ChainSpec {
                    commerce_block_time: 3,
                    security_block_time: 540, // 9 minutes
                    min_validator_stake: 20,
                    max_validators: 1000,
                },
                vec![
                    "/dns4/mainnet-boot1.omne.network/tcp/30303".to_string(),
                    "/dns4/mainnet-boot2.omne.network/tcp/30303".to_string(),
                ]
            ),
            "testnet" => (
                2,
                "0x1111111111111111111111111111111111111111111111111111111111111111".to_string(),
                ChainSpec {
                    commerce_block_time: 3,
                    security_block_time: 540,
                    min_validator_stake: 10, // Lower for testing
                    max_validators: 100,
                },
                vec![
                    "/dns4/testnet-boot1.omne.network/tcp/30303".to_string(),
                    "/dns4/testnet-boot2.omne.network/tcp/30303".to_string(),
                ]
            ),
            "devnet" => (
                3,
                "0x2222222222222222222222222222222222222222222222222222222222222222".to_string(),
                ChainSpec {
                    commerce_block_time: 3,
                    security_block_time: 60, // 1 minute for faster testing
                    min_validator_stake: 1,
                    max_validators: 10,
                },
                vec![
                    "/ip4/127.0.0.1/tcp/30303".to_string(),
                ]
            ),
            _ => return Err(anyhow::anyhow!("Unknown network: {}", network_name)),
        };

        Ok(Self {
            data_dir: PathBuf::from("~/.omne-nexus"),
            network: NetworkConfig {
                name: network_name.to_string(),
                id: network_id,
                genesis_hash,
                chain_spec,
            },
            p2p: P2PConfig {
                port: 30303,
                max_peers: 50,
                bootstrap_peers,
                connection_timeout: Duration::from_secs(10),
                enable_mdns: network_name == "devnet",
                enable_kad: true,
            },
            rpc: RpcConfig {
                port: 9944,
                bind_address: "127.0.0.1".to_string(),
                enable_http: true,
                enable_ws: true,
                max_connections: 100,
            },
            validator: ValidatorSettings {
                is_validator: false,
                validator_stake: 20,
                validator_key_path: None,
                network_key_path: None,
                auto_restake: true,
                slashing_protection: SlashingProtectionConfig {
                    enabled: true,
                    min_source_epoch_diff: 1,
                    min_target_epoch_diff: 1,
                },
            },
            oon: OonConfig {
                enable_oon: false,
                max_concurrent_jobs: 4,
                resource_allocation: 0.5, // 50% of available resources
                supported_services: vec![
                    "ai-inference".to_string(),
                    "scientific-computation".to_string(),
                    "data-processing".to_string(),
                ],
                revenue_share_percentage: 0.8, // 80% to validators, 20% to network
            },
        })
    }

    /// Initialize validator directories
    pub fn init_directories(&self, data_dir: &PathBuf) -> Result<()> {
        fs::create_dir_all(data_dir)?;
        fs::create_dir_all(data_dir.join("keys"))?;
        fs::create_dir_all(data_dir.join("db"))?;
        fs::create_dir_all(data_dir.join("logs"))?;
        Ok(())
    }

    /// Generate validator keys
    pub fn generate_validator_keys(&self, data_dir: &PathBuf) -> Result<()> {
        let keys_dir = data_dir.join("keys");
        
        // TODO: Generate actual cryptographic keys
        // For now, create placeholder files
        fs::write(keys_dir.join("validator.key"), "placeholder_validator_key")?;
        fs::write(keys_dir.join("network.key"), "placeholder_network_key")?;
        fs::write(keys_dir.join("oon.key"), "placeholder_oon_key")?;
        
        Ok(())
    }

    /// Save configuration to file
    pub fn save_to_file(&self, path: &PathBuf) -> Result<()> {
        let toml_content = toml::to_string_pretty(self)
            .context("Failed to serialize configuration")?;
        fs::write(path, toml_content)
            .context("Failed to write configuration file")?;
        Ok(())
    }

    /// Load configuration from file
    pub fn load_from_file(path: &PathBuf) -> Result<Self> {
        let content = fs::read_to_string(path)
            .context("Failed to read configuration file")?;
        let config: Self = toml::from_str(&content)
            .context("Failed to parse configuration file")?;
        Ok(config)
    }
}
