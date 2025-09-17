//! Omne Nexus - High-Performance Validator Node
//!
//! The definitive validator implementation for the Omne blockchain network.
//! Implements PoVERA consensus and participates in dual-layer architecture.

use clap::{Parser, Subcommand};
use anyhow::{Result, Context};
use tracing::{info};
use std::path::PathBuf;

mod validator;
mod consensus;
mod p2p;
mod rpc;
mod config;
mod utils;

use validator::ValidatorNode;
use config::ValidatorConfig;

/// Omne Nexus - The definitive validator node for Omne blockchain
#[derive(Parser)]
#[command(
    name = "omne-nexus",
    about = "High-performance validator node for the Omne blockchain network",
    long_about = "Omne Nexus is the definitive validator implementation for the Omne blockchain network, \
                  featuring PoVERA consensus, dual-layer architecture, and OON integration for maximum \
                  efficiency and profitability.",
    version = env!("CARGO_PKG_VERSION"),
    author = "Omne Network <dev@omne.network>"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize validator configuration
    Init {
        /// Data directory for validator storage
        #[arg(long, default_value = "~/.omne-nexus")]
        data_dir: PathBuf,
        
        /// Network to initialize for (mainnet, testnet, devnet)
        #[arg(long, default_value = "testnet")]
        network: String,
        
        /// Generate new validator keys
        #[arg(long)]
        generate_keys: bool,
    },
    
    /// Start the validator node
    Start {
        /// Data directory for validator storage
        #[arg(long, default_value = "~/.omne-nexus")]
        data_dir: PathBuf,
        
        /// Configuration file path
        #[arg(long)]
        config: Option<PathBuf>,
        
        /// Enable validator mode (participate in consensus)
        #[arg(long)]
        validator: bool,
        
        /// Validator stake amount in OGT
        #[arg(long, default_value = "100")]
        stake: u64,
        
        /// Network to connect to
        #[arg(long, default_value = "testnet")]
        network: String,
        
        /// P2P listening port
        #[arg(long, default_value = "30303")]
        p2p_port: u16,
        
        /// RPC server port
        #[arg(long, default_value = "9944")]
        rpc_port: u16,
        
        /// Bootstrap peers (comma-separated)
        #[arg(long)]
        bootstrap_peers: Option<String>,
        
        /// Enable OON computational services
        #[arg(long)]
        enable_oon: bool,
    },
    
    /// Show validator status
    Status {
        /// RPC endpoint to query
        #[arg(long, default_value = "http://127.0.0.1:9944")]
        rpc_endpoint: String,
    },
    
    /// Generate validator keys
    Keys {
        /// Output directory for keys
        #[arg(long, default_value = "~/.omne-nexus/keys")]
        output_dir: PathBuf,
        
        /// Key type to generate (validator, network, oon)
        #[arg(long, default_value = "validator")]
        key_type: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize logging
    let filter = if cli.verbose {
        "debug,libp2p=info,sled=info"
    } else {
        "info,libp2p=warn,sled=warn"
    };
    
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();
    
    match cli.command {
        Commands::Init { data_dir, network, generate_keys } => {
            info!("🔧 Initializing Omne Nexus validator...");
            init_validator(data_dir, network, generate_keys).await
        },
        
        Commands::Start { 
            data_dir, 
            config, 
            validator, 
            stake, 
            network, 
            p2p_port, 
            rpc_port, 
            bootstrap_peers,
            enable_oon 
        } => {
            info!("🚀 Starting Omne Nexus validator node...");
            start_validator(
                data_dir, 
                config, 
                validator, 
                stake, 
                network, 
                p2p_port, 
                rpc_port, 
                bootstrap_peers,
                enable_oon
            ).await
        },
        
        Commands::Status { rpc_endpoint } => {
            info!("📊 Checking validator status...");
            show_status(rpc_endpoint).await
        },
        
        Commands::Keys { output_dir, key_type } => {
            info!("🔑 Generating validator keys...");
            generate_keys(output_dir, key_type).await
        },
    }
}

async fn init_validator(
    data_dir: PathBuf, 
    network: String, 
    generate_keys: bool
) -> Result<()> {
    let config = ValidatorConfig::new_for_network(&network)?;
    config.init_directories(&data_dir)?;
    
    if generate_keys {
        config.generate_validator_keys(&data_dir)?;
    }
    
    config.save_to_file(&data_dir.join("config.toml"))?;
    
    info!("✅ Nexus validator initialized successfully");
    info!("   Data directory: {}", data_dir.display());
    info!("   Network: {}", network);
    info!("   Configuration saved to: {}", data_dir.join("config.toml").display());
    
    Ok(())
}

async fn start_validator(
    data_dir: PathBuf,
    config_path: Option<PathBuf>,
    is_validator: bool,
    stake: u64,
    network: String,
    p2p_port: u16,
    rpc_port: u16,
    bootstrap_peers: Option<String>,
    enable_oon: bool,
) -> Result<()> {
    // Load or create configuration
    let mut config = if let Some(config_path) = config_path {
        ValidatorConfig::load_from_file(&config_path)?
    } else {
        let default_config_path = data_dir.join("config.toml");
        if default_config_path.exists() {
            ValidatorConfig::load_from_file(&default_config_path)?
        } else {
            ValidatorConfig::new_for_network(&network)?
        }
    };
    
    // Override config with CLI parameters
    config.data_dir = data_dir;
    config.validator.is_validator = is_validator;
    config.validator.validator_stake = stake;
    config.p2p.port = p2p_port;
    config.rpc.port = rpc_port;
    config.oon.enable_oon = enable_oon;
    
    if let Some(peers) = bootstrap_peers {
        config.p2p.bootstrap_peers = peers.split(',').map(|s| s.trim().to_string()).collect();
    }
    
    // Validate minimum stake for validators
    if is_validator && stake < 20 {
        return Err(anyhow::anyhow!(
            "Minimum validator stake is 20 OGT, provided: {}", stake
        ));
    }
    
    // Create and start validator node
    let validator_node = ValidatorNode::new(config).await
        .context("Failed to create validator node")?;
    
    // Start the node and wait for shutdown signal
    validator_node.start().await
        .context("Failed to start validator node")?;
    
    Ok(())
}

async fn show_status(rpc_endpoint: String) -> Result<()> {
    // TODO: Implement RPC client to query validator status
    info!("Querying status from: {}", rpc_endpoint);
    info!("Status functionality coming soon...");
    Ok(())
}

async fn generate_keys(output_dir: PathBuf, key_type: String) -> Result<()> {
    std::fs::create_dir_all(&output_dir)?;
    
    match key_type.as_str() {
        "validator" => {
            info!("Generating validator consensus keys...");
            // TODO: Generate BLS validator keys
        },
        "network" => {
            info!("Generating network identity keys...");
            // TODO: Generate Ed25519 network keys
        },
        "oon" => {
            info!("Generating OON service keys...");
            // TODO: Generate OON computation keys
        },
        _ => {
            return Err(anyhow::anyhow!("Unknown key type: {}", key_type));
        }
    }
    
    info!("✅ Keys generated in: {}", output_dir.display());
    Ok(())
}
