//! Validator node implementation
//!
//! This module contains the core validator node logic that orchestrates
//! consensus participation, P2P networking, and RPC services.

use crate::config::ValidatorConfig;
use crate::consensus::PoVERAValidator;
use crate::p2p::P2PNetwork;
use crate::rpc::RpcServer;

use anyhow::{Result, Context};
use tokio::signal;
use tokio::sync::broadcast;
use tracing::{info, warn, error};
use std::sync::Arc;

/// Main validator node structure
pub struct ValidatorNode {
    /// Node configuration
    config: ValidatorConfig,
    /// PoVERA consensus validator
    consensus: Arc<PoVERAValidator>,
    /// P2P networking layer
    p2p_network: Arc<P2PNetwork>,
    /// RPC server
    rpc_server: Arc<RpcServer>,
    /// Shutdown signal broadcaster
    shutdown_tx: broadcast::Sender<()>,
}

impl ValidatorNode {
    /// Create a new validator node
    pub async fn new(config: ValidatorConfig) -> Result<Self> {
        info!("🔧 Initializing Omne Validator Node");
        info!("   Network: {} (ID: {})", config.network.name, config.network.id);
        info!("   Data Directory: {}", config.data_dir.display());
        info!("   Validator Mode: {}", config.validator.is_validator);
        
        if config.validator.is_validator {
            info!("   Validator Stake: {} OGT", config.validator.validator_stake);
            
            // Validate dynamic stake requirements - BREAKTHROUGH OPTIMIZATION
            let network_utilization = 0.5; // Default assumption - should be fetched from network
            let validator_count = 50; // Default assumption - should be fetched from network
            let required_stake = calculate_dynamic_stake_requirement(
                network_utilization,
                validator_count,
                config.network.chain_spec.min_validator_stake
            );
            
            if config.validator.validator_stake < required_stake {
                return Err(anyhow::anyhow!(
                    "Validator stake {} OGT is below dynamic minimum required {} OGT (based on network utilization: {:.1}%, {} validators)",
                    config.validator.validator_stake,
                    required_stake,
                    network_utilization * 100.0,
                    validator_count
                ));
            }
        }
        
        if config.oon.enable_oon {
            info!("   OON Services: Enabled ({} max jobs)", config.oon.max_concurrent_jobs);
        }

        // Initialize directories
        config.init_directories(&config.data_dir)?;

        // Create shutdown channel
        let (shutdown_tx, _) = broadcast::channel(1);

        // Initialize consensus validator
        let consensus = Arc::new(
            PoVERAValidator::new(&config).await
                .context("Failed to initialize consensus validator")?
        );

        // Initialize P2P network
        let p2p_network = Arc::new(
            P2PNetwork::new(&config, consensus.clone()).await
                .context("Failed to initialize P2P network")?
        );

        // Initialize RPC server
        let rpc_server = Arc::new(
            RpcServer::new(&config, consensus.clone(), p2p_network.clone()).await
                .context("Failed to initialize RPC server")?
        );

        Ok(Self {
            config,
            consensus,
            p2p_network,
            rpc_server,
            shutdown_tx,
        })
    }

    /// Start the validator node
    pub async fn start(self) -> Result<()> {
        info!("🚀 Starting Omne Validator Node...");

        let mut shutdown_rx = self.shutdown_tx.subscribe();

        // Start consensus validator
        let consensus_handle = {
            let consensus = self.consensus.clone();
            let shutdown_rx = self.shutdown_tx.subscribe();
            tokio::spawn(async move {
                if let Err(e) = consensus.start(shutdown_rx).await {
                    error!("Consensus validator error: {}", e);
                }
            })
        };

        // Start P2P network
        let p2p_handle = {
            let p2p_network = self.p2p_network.clone();
            let shutdown_rx = self.shutdown_tx.subscribe();
            tokio::spawn(async move {
                if let Err(e) = p2p_network.start(shutdown_rx).await {
                    error!("P2P network error: {}", e);
                }
            })
        };

        // Start RPC server
        let rpc_handle = {
            let rpc_server = self.rpc_server.clone();
            let shutdown_rx = self.shutdown_tx.subscribe();
            tokio::spawn(async move {
                if let Err(e) = rpc_server.start(shutdown_rx).await {
                    error!("RPC server error: {}", e);
                }
            })
        };

        info!("✅ Validator node started successfully");
        info!("   P2P Port: {}", self.config.p2p.port);
        info!("   RPC Port: {}", self.config.rpc.port);
        
        if self.config.validator.is_validator {
            info!("🏛️  Validator is active and participating in consensus");
        } else {
            info!("👁️  Running in observer mode (not validating)");
        }

        // Wait for shutdown signal
        tokio::select! {
            _ = signal::ctrl_c() => {
                info!("🛑 Received shutdown signal (Ctrl+C)");
            }
            _ = shutdown_rx.recv() => {
                info!("🛑 Received internal shutdown signal");
            }
        }

        // Initiate shutdown
        info!("🔄 Shutting down validator node...");
        let _ = self.shutdown_tx.send(());

        // Wait for all components to shut down
        let shutdown_timeout = tokio::time::Duration::from_secs(30);
        
        tokio::select! {
            _ = consensus_handle => info!("✅ Consensus validator shut down"),
            _ = tokio::time::sleep(shutdown_timeout) => warn!("⚠️  Consensus validator shutdown timeout"),
        }

        tokio::select! {
            _ = p2p_handle => info!("✅ P2P network shut down"),
            _ = tokio::time::sleep(shutdown_timeout) => warn!("⚠️  P2P network shutdown timeout"),
        }

        tokio::select! {
            _ = rpc_handle => info!("✅ RPC server shut down"),
            _ = tokio::time::sleep(shutdown_timeout) => warn!("⚠️  RPC server shutdown timeout"),
        }

        info!("👋 Omne Validator Node shut down successfully");
        Ok(())
    }

    /// Get validator node status
    pub async fn status(&self) -> Result<ValidatorStatus> {
        let consensus_status = self.consensus.status().await?;
        let p2p_status = self.p2p_network.status().await?;
        let rpc_status = self.rpc_server.status().await?;

        Ok(ValidatorStatus {
            consensus: consensus_status,
            p2p: p2p_status,
            rpc: rpc_status,
            config: self.config.clone(),
        })
    }
}

/// Dynamic stake calculation function - BREAKTHROUGH OPTIMIZATION
/// Replaces static minimum stake requirements with network-responsive calculation
fn calculate_dynamic_stake_requirement(
    network_utilization: f64,
    validator_count: u32,
    base_stake: u64
) -> u64 {
    let utilization_factor = (0.5_f64).max((2.0_f64).min(1.0 + network_utilization));
    let validator_density = (0.8_f64).max((1.5_f64).min(validator_count as f64 / 100.0));
    
    let dynamic_stake = base_stake as f64 * utilization_factor * validator_density;
    (15_u64).max((150_u64).min(dynamic_stake as u64)) // Hard stability limits: 15-150 OGT
}

/// Validator node status information
#[derive(Debug, Clone)]
pub struct ValidatorStatus {
    pub consensus: crate::consensus::ConsensusStatus,
    pub p2p: crate::p2p::P2PStatus,
    pub rpc: crate::rpc::RpcStatus,
    pub config: ValidatorConfig,
}
