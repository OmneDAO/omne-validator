//! P2P networking for Omne validator nodes

use crate::config::ValidatorConfig;
use crate::consensus::PoVERAValidator;

use anyhow::Result;
use futures::StreamExt;
use libp2p::{
    gossipsub, identify, kad, mdns, noise, ping, yamux, 
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, Multiaddr, PeerId, Swarm, Transport,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, Mutex};
use tracing::{info, debug, warn};

/// P2P network implementation for Omne validators
pub struct P2PNetwork {
    config: ValidatorConfig,
    consensus: Arc<PoVERAValidator>,
    swarm: Mutex<Option<Swarm<ValidatorNetworkBehaviour>>>,
}

// NOTE: Removed unsafe Send + Sync implementations for security.
// Rust will automatically implement Send + Sync if all fields are Send + Sync.
// This prevents potential data races and undefined behavior.

/// Network behaviour for validator nodes
#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "ValidatorNetworkBehaviourEvent")]
pub struct ValidatorNetworkBehaviour {
    pub ping: ping::Behaviour,
    pub identify: identify::Behaviour,  
    pub kad: kad::Behaviour<kad::store::MemoryStore>,
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
}

/// Events from the validator network behaviour
#[derive(Debug)]
pub enum ValidatorNetworkBehaviourEvent {
    Ping(ping::Event),
    Identify(identify::Event),
    Kad(kad::Event),
    Gossipsub(gossipsub::Event),
    Mdns(mdns::Event),
}

impl From<ping::Event> for ValidatorNetworkBehaviourEvent {
    fn from(event: ping::Event) -> Self {
        ValidatorNetworkBehaviourEvent::Ping(event)
    }
}

impl From<identify::Event> for ValidatorNetworkBehaviourEvent {
    fn from(event: identify::Event) -> Self {
        ValidatorNetworkBehaviourEvent::Identify(event)
    }
}

impl From<kad::Event> for ValidatorNetworkBehaviourEvent {
    fn from(event: kad::Event) -> Self {
        ValidatorNetworkBehaviourEvent::Kad(event)
    }
}

impl From<gossipsub::Event> for ValidatorNetworkBehaviourEvent {
    fn from(event: gossipsub::Event) -> Self {
        ValidatorNetworkBehaviourEvent::Gossipsub(event)
    }
}

impl From<mdns::Event> for ValidatorNetworkBehaviourEvent {
    fn from(event: mdns::Event) -> Self {
        ValidatorNetworkBehaviourEvent::Mdns(event)
    }
}

/// P2P network status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PStatus {
    pub local_peer_id: String,
    pub connected_peers: usize,
    pub listening_addresses: Vec<String>,
    pub network_id: u64,
    pub gossipsub_topics: Vec<String>,
}

impl P2PNetwork {
    /// Create a new P2P network
    pub async fn new(
        config: &ValidatorConfig, 
        consensus: Arc<PoVERAValidator>
    ) -> Result<Self> {
        info!("🌐 Initializing P2P network");
        info!("   Network: {} (ID: {})", config.network.name, config.network.id);
        info!("   P2P Port: {}", config.p2p.port);
        info!("   Max Peers: {}", config.p2p.max_peers);
        info!("   Bootstrap Peers: {}", config.p2p.bootstrap_peers.len());

        Ok(Self {
            config: config.clone(),
            consensus,
            swarm: Mutex::new(None),
        })
    }

    /// Initialize the libp2p swarm
    async fn init_swarm(&self) -> Result<()> {
        // Create a random key for this node
        let local_key = libp2p::identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());

        info!("🆔 Local Peer ID: {}", local_peer_id);

        // Create transport
        let transport = tcp::tokio::Transport::default()
            .upgrade(libp2p::core::upgrade::Version::V1)
            .authenticate(noise::Config::new(&local_key)?)
            .multiplex(yamux::Config::default())
            .boxed();

        // Create network behaviour
        let behaviour = ValidatorNetworkBehaviour {
            ping: ping::Behaviour::new(ping::Config::new()),
            identify: identify::Behaviour::new(identify::Config::new(
                "/omne/validator/1.0.0".to_string(),
                local_key.public(),
            )),
            kad: kad::Behaviour::new(
                local_peer_id,
                kad::store::MemoryStore::new(local_peer_id),
            ),
            gossipsub: self.create_gossipsub_behaviour()?,
            mdns: mdns::tokio::Behaviour::new(
                mdns::Config::default(),
                local_peer_id,
            )?,
        };

        // Create swarm
        let mut swarm = Swarm::new(
            transport,
            behaviour,
            local_peer_id,
            libp2p::swarm::Config::with_tokio_executor()
        );

        // Listen on configured port
        let listen_addr: Multiaddr = format!("/ip4/0.0.0.0/tcp/{}", self.config.p2p.port)
            .parse()?;
        swarm.listen_on(listen_addr)?;

        // Connect to bootstrap peers
        for peer_addr in &self.config.p2p.bootstrap_peers {
            if let Ok(addr) = peer_addr.parse::<Multiaddr>() {
                if let Err(e) = swarm.dial(addr) {
                    warn!("Failed to dial bootstrap peer {}: {}", peer_addr, e);
                }
            }
        }

        *self.swarm.lock().await = Some(swarm);
        Ok(())
    }

    /// Create gossipsub behaviour for consensus messages
    fn create_gossipsub_behaviour(&self) -> Result<gossipsub::Behaviour> {
        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(Duration::from_millis(1000))
            .validation_mode(gossipsub::ValidationMode::Strict)
            .message_id_fn(|message| {
                use sha2::{Digest, Sha256};
                let mut hasher = Sha256::new();
                hasher.update(&message.data);
                let hash = hasher.finalize();
                gossipsub::MessageId::from(format!("{:x}", hash))
            })
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to build gossipsub config: {:?}", e))?;

        let mut gossipsub = gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(libp2p::identity::Keypair::generate_ed25519()),
            gossipsub_config,
        )
        .map_err(|e| anyhow::anyhow!("Failed to create gossipsub behaviour: {:?}", e))?;

        // Subscribe to consensus topics
        let network_id = self.config.network.id;
        let topics = vec![
            format!("omne/consensus/commerce/{}", network_id),
            format!("omne/consensus/security/{}", network_id),
            format!("omne/transactions/{}", network_id),
            format!("omne/attestations/{}", network_id),
        ];

        for topic_name in topics {
            let topic = gossipsub::IdentTopic::new(&topic_name);
            gossipsub.subscribe(&topic)?;
            info!("📡 Subscribed to topic: {}", topic_name);
        }

        Ok(gossipsub)
    }

    /// Start the P2P network
    pub async fn start(&self, mut shutdown: broadcast::Receiver<()>) -> Result<()> {
        info!("🚀 Starting P2P network");

        // Initialize swarm
        self.init_swarm().await?;
        
        // Main network event loop
        loop {
            tokio::select! {
                event = async {
                    let mut swarm_guard = self.swarm.lock().await;
                    if let Some(swarm) = swarm_guard.as_mut() {
                        swarm.next().await
                    } else {
                        None
                    }
                } => {
                    if let Some(event) = event {
                        if let Err(e) = Self::handle_swarm_event_static(event, &self.consensus).await {
                            warn!("Error handling swarm event: {}", e);
                        }
                    }
                }
                
                _ = shutdown.recv() => {
                    info!("🛑 Shutting down P2P network");
                    break;
                }
            }
        }

        Ok(())
    }

    /// Handle libp2p swarm events (static version to avoid borrow issues)
    async fn handle_swarm_event_static(
        event: SwarmEvent<ValidatorNetworkBehaviourEvent, impl std::error::Error>, 
        consensus: &Arc<PoVERAValidator>
    ) -> Result<()> {
        match event {
            SwarmEvent::NewListenAddr { address, .. } => {
                info!("🎧 Listening on: {}", address);
            }
            SwarmEvent::Behaviour(ValidatorNetworkBehaviourEvent::Ping(event)) => {
                debug!("🏓 Ping event: {:?}", event);
            }
            SwarmEvent::Behaviour(ValidatorNetworkBehaviourEvent::Identify(event)) => {
                debug!("🆔 Identify event: {:?}", event);
            }
            SwarmEvent::Behaviour(ValidatorNetworkBehaviourEvent::Kad(event)) => {
                debug!("🗺️  Kademlia event: {:?}", event);
            }
            SwarmEvent::Behaviour(ValidatorNetworkBehaviourEvent::Gossipsub(event)) => {
                Self::handle_gossipsub_event_static(event, consensus).await?;
            }
            SwarmEvent::Behaviour(ValidatorNetworkBehaviourEvent::Mdns(event)) => {
                debug!("🔍 mDNS event: {:?}", event);
            }
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                info!("🤝 Connected to peer: {}", peer_id);
            }
            SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                info!("👋 Disconnected from peer: {} (cause: {:?})", peer_id, cause);
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle libp2p swarm events
    async fn handle_swarm_event(&self, event: SwarmEvent<ValidatorNetworkBehaviourEvent, impl std::error::Error>) -> Result<()> {
        match event {
            SwarmEvent::NewListenAddr { address, .. } => {
                info!("🎧 Listening on: {}", address);
            }
            SwarmEvent::Behaviour(ValidatorNetworkBehaviourEvent::Ping(event)) => {
                debug!("🏓 Ping event: {:?}", event);
            }
            SwarmEvent::Behaviour(ValidatorNetworkBehaviourEvent::Identify(event)) => {
                debug!("🆔 Identify event: {:?}", event);
            }
            SwarmEvent::Behaviour(ValidatorNetworkBehaviourEvent::Kad(event)) => {
                debug!("🗺️  Kademlia event: {:?}", event);
            }
            SwarmEvent::Behaviour(ValidatorNetworkBehaviourEvent::Gossipsub(event)) => {
                self.handle_gossipsub_event(event).await?;
            }
            SwarmEvent::Behaviour(ValidatorNetworkBehaviourEvent::Mdns(event)) => {
                debug!("🔍 mDNS event: {:?}", event);
            }
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                info!("🤝 Connected to peer: {}", peer_id);
            }
            SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                info!("👋 Disconnected from peer: {} (cause: {:?})", peer_id, cause);
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle gossipsub events (consensus messages) - static version
    async fn handle_gossipsub_event_static(
        event: gossipsub::Event, 
        _consensus: &Arc<PoVERAValidator>
    ) -> Result<()> {
        match event {
            gossipsub::Event::Message { 
                propagation_source, 
                message_id, 
                message 
            } => {
                debug!(
                    "📨 Received message {} from {} on topic {}",
                    message_id,
                    propagation_source,
                    message.topic
                );
                
                // TODO: Route message to consensus validator for processing
                // This is where we'd handle:
                // - Commerce block proposals
                // - Security block proposals  
                // - Attestations
                // - Transaction broadcasts
            }
            gossipsub::Event::Subscribed { peer_id, topic } => {
                debug!("📡 Peer {} subscribed to topic {}", peer_id, topic);
            }
            gossipsub::Event::Unsubscribed { peer_id, topic } => {
                debug!("📡 Peer {} unsubscribed from topic {}", peer_id, topic);
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle gossipsub events (consensus messages)
    async fn handle_gossipsub_event(&self, event: gossipsub::Event) -> Result<()> {
        match event {
            gossipsub::Event::Message { 
                propagation_source, 
                message_id, 
                message 
            } => {
                debug!(
                    "📨 Received message {} from {} on topic {}",
                    message_id,
                    propagation_source,
                    message.topic
                );
                
                // TODO: Route message to consensus validator for processing
                // This is where we'd handle:
                // - Commerce block proposals
                // - Security block proposals  
                // - Attestations
                // - Transaction broadcasts
            }
            gossipsub::Event::Subscribed { peer_id, topic } => {
                debug!("📡 Peer {} subscribed to topic {}", peer_id, topic);
            }
            gossipsub::Event::Unsubscribed { peer_id, topic } => {
                debug!("📡 Peer {} unsubscribed from topic {}", peer_id, topic);
            }
            _ => {}
        }
        Ok(())
    }

    /// Get P2P network status
    pub async fn status(&self) -> Result<P2PStatus> {
        // TODO: Implement actual status collection from swarm
        Ok(P2PStatus {
            local_peer_id: "placeholder_peer_id".to_string(),
            connected_peers: 0,
            listening_addresses: vec!["placeholder_address".to_string()],
            network_id: self.config.network.id,
            gossipsub_topics: vec![
                format!("omne/consensus/commerce/{}", self.config.network.id),
                format!("omne/consensus/security/{}", self.config.network.id),
            ],
        })
    }
}
