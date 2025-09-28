//! PoVERA Consensus Validator
//!
//! Implements Proof of Value Economic Randomized Agreement consensus
//! for the Omne blockchain network.

use crate::config::ValidatorConfig;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::{Duration};
use tokio::sync::broadcast;
use tracing::{info, debug, warn};

/// PoVERA consensus validator implementation
pub struct PoVERAValidator {
    config: ValidatorConfig,
    state: ConsensusState,
    performance_metrics: PerformanceMetrics,
    network_metrics: NetworkMetrics,
}

// NOTE: Removed unsafe Send + Sync implementations for security.
// Rust will automatically implement Send + Sync if all fields are Send + Sync.
// This prevents potential data races and undefined behavior.

/// Current consensus state
#[derive(Debug, Clone)]
pub struct ConsensusState {
    /// Current commerce layer epoch
    pub commerce_epoch: u64,
    /// Current security layer epoch  
    pub security_epoch: u64,
    /// Last commerce block height
    pub commerce_height: u64,
    /// Last security block height
    pub security_height: u64,
    /// Validator is actively participating
    pub is_active: bool,
    /// Current validator stake
    pub stake: u64,
    /// Blocks proposed in current epoch
    pub blocks_proposed: u64,
    /// Attestations made in current epoch
    pub attestations_made: u64,
}

/// Consensus status for external queries - INFRASTRUCTURE SERVICES ENHANCED
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusStatus {
    pub is_validator: bool,
    pub is_active: bool,
    pub commerce_epoch: u64,
    pub security_epoch: u64,
    pub commerce_height: u64,
    pub security_height: u64,
    pub stake: u64,
    pub uptime_percentage: f64,
    pub last_block_time: Option<u64>,
    pub infrastructure_services: InfrastructureServiceStatus,
}

/// Infrastructure service status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfrastructureServiceStatus {
    pub oon_enabled: bool,
    pub oon_jobs_completed: u64,
    pub omp_enabled: bool,
    pub omp_storage_served_gb: u64,
    pub orc20_relayer_enabled: bool,
    pub orc20_txs_relayed: u64,
    pub paymaster_enabled: bool,
    pub paymaster_txs_sponsored: u64,
    pub total_monthly_revenue_estimate: u128,
}

/// Performance metrics tracking - INFRASTRUCTURE SERVICES INTEGRATION
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Total uptime in seconds
    pub total_uptime: u64,
    /// Total downtime in seconds  
    pub total_downtime: u64,
    /// Blocks successfully proposed
    pub blocks_proposed: u64,
    /// Blocks missed
    pub blocks_missed: u64,
    /// Average block production time
    pub avg_block_time: Duration,
    /// Computational jobs completed (OON)
    pub oon_jobs_completed: u64,
    /// OMP storage requests served
    pub omp_requests_served: u64,
    /// ORC-20 transactions relayed
    pub orc20_txs_relayed: u64,
    /// Paymaster transactions sponsored
    pub paymaster_txs_sponsored: u64,
    /// Total revenue generated for network
    pub total_revenue_generated: u128,
    /// Revenue breakdown by service
    pub revenue_by_service: ServiceRevenueBreakdown,
    /// Start time for performance tracking
    pub start_time: std::time::Instant,
}

/// Revenue breakdown by infrastructure service
#[derive(Debug, Clone)]
pub struct ServiceRevenueBreakdown {
    /// Revenue from OON computational services
    pub oon_revenue: u128,
    /// Revenue from OMP storage services
    pub omp_revenue: u128,
    /// Revenue from ORC-20 relayer services
    pub orc20_relayer_revenue: u128,
    /// Revenue from OEC-4337 paymaster services
    pub paymaster_revenue: u128,
}

/// Network metrics monitoring - BREAKTHROUGH OPTIMIZATION  
#[derive(Debug, Clone)]
pub struct NetworkMetrics {
    /// Current network utilization
    pub network_utilization: f64,
    /// Total active validators
    pub active_validators: u32,
    /// Current dynamic stake requirement
    pub dynamic_stake_requirement: u64,
    /// Network health score (0-100)
    pub network_health: f64,
    /// Last metrics update time
    pub last_update: std::time::Instant,
}

impl PoVERAValidator {
    /// Create a new PoVERA validator
    pub async fn new(config: &ValidatorConfig) -> Result<Self> {
        info!("🔧 Initializing PoVERA consensus validator");
        
        let state = ConsensusState {
            commerce_epoch: 0,
            security_epoch: 0,
            commerce_height: 0,
            security_height: 0,
            is_active: config.validator.is_validator,
            stake: config.validator.validator_stake,
            blocks_proposed: 0,
            attestations_made: 0,
        };

        let performance_metrics = PerformanceMetrics {
            total_uptime: 0,
            total_downtime: 0,
            blocks_proposed: 0,
            blocks_missed: 0,
            avg_block_time: Duration::from_secs(3),
            oon_jobs_completed: 0,
            omp_requests_served: 0,
            orc20_txs_relayed: 0,
            paymaster_txs_sponsored: 0,
            total_revenue_generated: 0,
            revenue_by_service: ServiceRevenueBreakdown {
                oon_revenue: 0,
                omp_revenue: 0,
                orc20_relayer_revenue: 0,
                paymaster_revenue: 0,
            },
            start_time: std::time::Instant::now(),
        };

        let network_metrics = NetworkMetrics {
            network_utilization: 0.5,
            active_validators: 50, // Default assumption
            dynamic_stake_requirement: config.network.chain_spec.min_validator_stake,
            network_health: 95.0,
            last_update: std::time::Instant::now(),
        };

        Ok(Self {
            config: config.clone(),
            state,
            performance_metrics,
            network_metrics,
        })
    }

    /// Start the consensus validator
    pub async fn start(&self, mut shutdown: broadcast::Receiver<()>) -> Result<()> {
        info!("🚀 Starting PoVERA consensus validator");

        if !self.config.validator.is_validator {
            info!("👁️  Running in observer mode - not participating in consensus");
            // Just wait for shutdown signal
            let _ = shutdown.recv().await;
            return Ok(());
        }

        info!("🏛️  Validator active - participating in consensus");
        info!("   Stake: {} OGT", self.state.stake);
        info!("   Commerce block time: {}s", self.config.network.chain_spec.commerce_block_time);
        info!("   Security block time: {}s", self.config.network.chain_spec.security_block_time);

        // Main consensus loop
        let commerce_interval = Duration::from_secs(self.config.network.chain_spec.commerce_block_time);
        let security_interval = Duration::from_secs(self.config.network.chain_spec.security_block_time);

        let mut commerce_timer = tokio::time::interval(commerce_interval);
        let mut security_timer = tokio::time::interval(security_interval);

        loop {
            tokio::select! {
                _ = commerce_timer.tick() => {
                    if let Err(e) = self.process_commerce_slot().await {
                        warn!("Commerce slot processing error: {}", e);
                    }
                }
                
                _ = security_timer.tick() => {
                    if let Err(e) = self.process_security_slot().await {
                        warn!("Security slot processing error: {}", e);
                    }
                }
                
                _ = shutdown.recv() => {
                    info!("🛑 Shutting down consensus validator");
                    break;
                }
            }
        }

        Ok(())
    }

    /// Process a commerce layer consensus slot
    async fn process_commerce_slot(&self) -> Result<()> {
        debug!("⚡ Processing commerce consensus slot");
        
        // TODO: Implement actual consensus logic:
        // 1. Check if we're the proposer for this slot
        // 2. Create and propose commerce block if selected
        // 3. Attest to proposed blocks
        // 4. Handle consensus messages
        
        Ok(())
    }

    /// Process a security layer consensus slot
    async fn process_security_slot(&self) -> Result<()> {
        debug!("🔒 Processing security consensus slot");
        
        // TODO: Implement actual consensus logic:
        // 1. Aggregate commerce layer state
        // 2. Check if we're the proposer for security block
        // 3. Create and propose security block if selected
        // 4. Attest to proposed security blocks
        // 5. Handle finality decisions
        
        Ok(())
    }

    /// Get current consensus status - INFRASTRUCTURE SERVICES ENHANCED
    pub fn get_status(&self) -> ConsensusStatus {
        let uptime_percentage = self.calculate_uptime_percentage();
        
        ConsensusStatus {
            is_validator: self.config.validator.is_validator,
            is_active: self.state.is_active,
            commerce_epoch: self.state.commerce_epoch,
            security_epoch: self.state.security_epoch,
            commerce_height: self.state.commerce_height,
            security_height: self.state.security_height,
            stake: self.state.stake,
            uptime_percentage,
            last_block_time: None, // TODO: Track actual last block time
            infrastructure_services: InfrastructureServiceStatus {
                oon_enabled: self.config.oon.enable_oon,
                oon_jobs_completed: self.performance_metrics.oon_jobs_completed,
                omp_enabled: self.config.omp.enable_omp,
                omp_storage_served_gb: self.performance_metrics.omp_requests_served / 1000, // Rough estimate
                orc20_relayer_enabled: self.config.orc20_relayer.enable_relayer,
                orc20_txs_relayed: self.performance_metrics.orc20_txs_relayed,
                paymaster_enabled: self.config.paymaster.enable_paymaster,
                paymaster_txs_sponsored: self.performance_metrics.paymaster_txs_sponsored,
                total_monthly_revenue_estimate: self.calculate_monthly_revenue_estimate(),
            },
        }
    }

    /// Calculate estimated monthly revenue from all infrastructure services
    fn calculate_monthly_revenue_estimate(&self) -> u128 {
        let mut total = 0u128;
        
        // OMP revenue estimate (based on storage served)
        if self.config.omp.enable_omp {
            let gb_served = self.performance_metrics.omp_requests_served / 1000;
            total += gb_served as u128 * self.config.omp.pricing_per_gb_quar;
        }
        
        // ORC-20 relayer revenue estimate (based on transactions relayed)
        if self.config.orc20_relayer.enable_relayer {
            let avg_fee_per_tx = 1_000_000_000_000_000u128; // 0.001 OMC per transaction
            total += self.performance_metrics.orc20_txs_relayed as u128 * avg_fee_per_tx;
        }
        
        // Paymaster revenue estimate (based on sponsorship usage)
        if self.config.paymaster.enable_paymaster {
            let avg_sponsorship_fee = 500_000_000_000_000u128; // 0.0005 OMC per sponsored tx
            total += self.performance_metrics.paymaster_txs_sponsored as u128 * avg_sponsorship_fee;
        }
        
        // OON revenue estimate
        if self.config.oon.enable_oon {
            let avg_revenue_per_job = 10_000_000_000_000_000u128; // 0.01 OMC per job
            total += self.performance_metrics.oon_jobs_completed as u128 * avg_revenue_per_job;
        }
        
        total
    }

    /// Calculate uptime percentage - BREAKTHROUGH OPTIMIZATION
    fn calculate_uptime_percentage(&self) -> f64 {
        let total_time = self.performance_metrics.total_uptime + self.performance_metrics.total_downtime;
        if total_time == 0 {
            return 100.0;
        }
        (self.performance_metrics.total_uptime as f64 / total_time as f64) * 100.0
    }

    /// Calculate performance bonus based on metrics - BREAKTHROUGH OPTIMIZATION
    pub fn calculate_performance_bonus(&self, base_reward: u128) -> u128 {
        let uptime_score = self.calculate_uptime_percentage() / 100.0;
        let block_accuracy = if self.performance_metrics.blocks_proposed + self.performance_metrics.blocks_missed > 0 {
            self.performance_metrics.blocks_proposed as f64 / 
            (self.performance_metrics.blocks_proposed + self.performance_metrics.blocks_missed) as f64
        } else { 1.0 };
        
        let performance_score = uptime_score * block_accuracy;
        
        // Performance bonus: up to 20% for excellent performance
        let bonus_multiplier = if performance_score >= 0.95 { 0.20 }
                              else if performance_score >= 0.90 { 0.15 }
                              else if performance_score >= 0.85 { 0.10 }
                              else if performance_score >= 0.75 { 0.05 }
                              else { 0.0 };
        
        (base_reward as f64 * bonus_multiplier) as u128
    }

    /// Update network metrics periodically - BREAKTHROUGH OPTIMIZATION
    pub async fn update_network_metrics(&mut self) -> Result<()> {
        // In a real implementation, this would query the network for actual metrics
        // For now, we simulate some basic metric updates
        
        let now = std::time::Instant::now();
        if now.duration_since(self.network_metrics.last_update) > Duration::from_secs(30) {
            // Simulate network metrics update
            self.network_metrics.network_utilization = 0.6; // Would be calculated from actual network data
            self.network_metrics.active_validators = 55; // Would be queried from network
            
            // Calculate dynamic stake requirement
            self.network_metrics.dynamic_stake_requirement = self.calculate_dynamic_stake_requirement();
            
            // Update network health based on performance
            self.network_metrics.network_health = self.calculate_network_health();
            
            self.network_metrics.last_update = now;
            
            debug!("📊 Network metrics updated: utilization={:.1}%, validators={}, stake_req={} OGT, health={:.1}%",
                self.network_metrics.network_utilization * 100.0,
                self.network_metrics.active_validators,
                self.network_metrics.dynamic_stake_requirement,
                self.network_metrics.network_health
            );
        }
        
        Ok(())
    }

    /// Calculate dynamic stake requirement for current network conditions
    fn calculate_dynamic_stake_requirement(&self) -> u64 {
        let base_stake = self.config.network.chain_spec.min_validator_stake;
        let utilization_factor = (0.5_f64).max((2.0_f64).min(1.0 + self.network_metrics.network_utilization));
        let validator_density = (0.8_f64).max((1.5_f64).min(self.network_metrics.active_validators as f64 / 100.0));
        
        let dynamic_stake = base_stake as f64 * utilization_factor * validator_density;
        (15_u64).max((150_u64).min(dynamic_stake as u64)) // Hard stability limits
    }

    /// Calculate overall network health score
    fn calculate_network_health(&self) -> f64 {
        let uptime_score = self.calculate_uptime_percentage();
        let validator_count_score = if self.network_metrics.active_validators >= 21 { 100.0 } else { 50.0 };
        let utilization_score = (1.0 - (self.network_metrics.network_utilization - 0.5).abs()) * 100.0;
        
        (uptime_score + validator_count_score + utilization_score) / 3.0
    }
}
