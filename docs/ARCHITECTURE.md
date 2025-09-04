# Omne Validator Architecture

This document describes the architecture and design of the Omne Validator Node.

## Overview

The Omne Validator is a lightweight, focused implementation designed specifically for validating transactions and participating in the Omne blockchain consensus. It implements the PoVERA (Proof of Value Economic Randomized Agreement) consensus mechanism and supports Omne's unique dual-layer architecture.

## Design Principles

### 1. **Separation of Concerns**
- Clear separation between consensus, networking, and RPC layers
- Modular design allows independent development and testing
- Each component has well-defined responsibilities

### 2. **Minimal Dependencies**
- Focus only on validator functionality
- Exclude genesis block creation and bootstrapping logic
- Lightweight compared to full blockchain nodes

### 3. **Production Ready**
- Built with Rust for memory safety and performance
- Comprehensive error handling and logging
- Configurable for different network environments

### 4. **Economic Efficiency**
- Optional OON (Omne Orchestration Network) integration
- Computational revenue sharing with validators
- Economic incentives aligned with network security

## Architecture Components

```
┌─────────────────────────────────────────────────────────────┐
│                    Omne Validator Node                     │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │     RPC     │  │     CLI     │  │    Utils    │        │
│  │   Server    │  │  Interface  │  │  Helpers    │        │
│  └─────────────┘  └─────────────┘  └─────────────┘        │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │   PoVERA    │  │     P2P     │  │    Config   │        │
│  │  Consensus  │  │  Network    │  │ Management  │        │
│  └─────────────┘  └─────────────┘  └─────────────┘        │
├─────────────────────────────────────────────────────────────┤
│                   Core Dependencies                        │
│  • Tokio (Async Runtime)    • Libp2p (Networking)         │
│  • Serde (Serialization)    • Clap (CLI)                  │
│  • Anyhow (Error Handling)  • Tracing (Logging)           │
└─────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. Main Application (`src/main.rs`)

**Purpose**: CLI interface and application entry point

**Responsibilities**:
- Command-line argument parsing
- Logging configuration
- Component orchestration
- Graceful shutdown handling

**Key Features**:
- `init` - Initialize validator configuration
- `start` - Start validator node
- `status` - Query validator status
- `keys` - Generate cryptographic keys

### 2. Validator Node (`src/validator.rs`)

**Purpose**: Main coordinator for all validator operations

**Responsibilities**:
- Component lifecycle management
- Inter-component communication
- Status aggregation
- Shutdown coordination

**Architecture**:
```rust
pub struct ValidatorNode {
    config: ValidatorConfig,
    consensus: Arc<PoVERAValidator>,
    p2p_network: Arc<P2PNetwork>,
    rpc_server: Arc<RpcServer>,
    shutdown_tx: broadcast::Sender<()>,
}
```

### 3. PoVERA Consensus (`src/consensus.rs`)

**Purpose**: Implements Proof of Value Economic Randomized Agreement

**Components**:
- **Proof of Stake**: Validator selection and stake management
- **Proof of Valuable Computation**: External revenue generation via OON
- **RANDAO**: Cryptographic randomness for fair validator ordering
- **Byzantine Fault Tolerance**: Safety and liveness guarantees

**Dual-Layer Support**:
- **Commerce Layer**: 3-second block times for fast transactions
- **Security Layer**: 9-minute block times for finality and security

**Key Operations**:
```rust
// Process commerce layer consensus
async fn process_commerce_slot(&self) -> Result<()>

// Process security layer consensus  
async fn process_security_slot(&self) -> Result<()>

// Handle consensus messages from peers
async fn handle_consensus_message(&self, msg: ConsensusMessage) -> Result<()>
```

### 4. P2P Networking (`src/p2p.rs`)

**Purpose**: Peer-to-peer communication using libp2p

**Network Behaviours**:
- **Ping**: Connection health monitoring
- **Identify**: Peer identification and capability exchange
- **Kademlia**: Distributed hash table for peer discovery
- **GossipSub**: Pub/sub messaging for consensus
- **mDNS**: Local network discovery (devnet only)

**Message Types**:
- Commerce block proposals and attestations
- Security block proposals and attestations
- Transaction broadcasts
- Validator announcements

**Topics**:
```
omne/consensus/commerce/{network_id}
omne/consensus/security/{network_id}  
omne/transactions/{network_id}
omne/attestations/{network_id}
```

### 5. RPC Server (`src/rpc.rs`)

**Purpose**: JSON-RPC API for external queries and control

**Supported Methods**:
- `validator_status` - Current validator state
- `consensus_status` - Consensus participation info
- `p2p_status` - Network connectivity status
- `network_info` - Network configuration details
- `latest_block` - Most recent block information
- `health` - Node health check

**Transport Options**:
- HTTP JSON-RPC (default port 9944)
- WebSocket JSON-RPC (planned)
- IPC (planned)

### 6. Configuration (`src/config.rs`)

**Purpose**: Comprehensive configuration management

**Configuration Sections**:
- **Network**: Chain specification and network parameters
- **P2P**: Networking and peer management settings
- **RPC**: API server configuration
- **Validator**: Consensus participation settings
- **OON**: Computational service settings

**Network Presets**:
- **Mainnet**: Production network with full security
- **Testnet**: Testing network with relaxed parameters
- **Devnet**: Development network for local testing

## Data Flow

### 1. Startup Sequence

```
1. Parse CLI arguments
2. Load/generate configuration
3. Initialize validator keys
4. Start consensus validator
5. Start P2P network
6. Start RPC server
7. Begin consensus participation
```

### 2. Consensus Participation

```
Commerce Layer (every 3 seconds):
1. Check if selected as proposer
2. Create commerce block proposal
3. Broadcast via gossipsub
4. Collect attestations
5. Update local state

Security Layer (every 9 minutes):
1. Aggregate commerce layer state
2. Check if selected as proposer  
3. Create security block proposal
4. Broadcast via gossipsub
5. Achieve finality
```

### 3. Message Processing

```
P2P Message → GossipSub → Message Handler → Consensus Validator
                                      ↓
RPC Query ← RPC Server ← Status Aggregator ← Consensus State
```

## Security Considerations

### 1. **Slashing Protection**
- Prevents double voting and conflicting attestations
- Maintains validator reputation and stake safety
- Configurable safety margins for attestations

### 2. **Key Management**
- Separation of validator keys and network identity keys
- Secure key generation and storage
- Optional hardware security module support (planned)

### 3. **Network Security**
- Authenticated P2P connections using noise protocol
- Message validation and spam protection
- DDoS mitigation via connection limits

### 4. **Economic Security**
- Minimum stake requirements for validators
- Revenue sharing incentives for honest behavior
- Slashing penalties for malicious actions

## Performance Characteristics

### **Target Metrics**
- **Latency**: < 100ms consensus message processing
- **Throughput**: 1000+ transactions per second on commerce layer
- **Memory**: < 2GB RAM usage under normal operation
- **CPU**: < 50% utilization during active validation
- **Network**: < 10MB/hour bandwidth per peer

### **Optimizations**
- Async/await throughout for non-blocking operations
- Efficient serialization with bincode/serde
- Connection pooling and message batching
- Memory-mapped storage for large datasets

## Deployment Scenarios

### 1. **Validator Node**
```bash
omne-validator start \
  --validator \
  --stake 100 \
  --network mainnet \
  --enable-oon
```

### 2. **Observer Node**
```bash
omne-validator start \
  --network mainnet \
  --rpc-port 9944
```

### 3. **Development Node**
```bash
omne-validator start \
  --validator \
  --stake 1 \
  --network devnet
```

## Future Enhancements

### **Phase 1**: Core Functionality
- [ ] Complete PoVERA consensus implementation
- [ ] Full P2P message handling
- [ ] Comprehensive RPC API
- [ ] Slashing protection

### **Phase 2**: Advanced Features
- [ ] OON computational integration
- [ ] Hardware security module support
- [ ] Advanced monitoring and metrics
- [ ] Automatic updates

### **Phase 3**: Ecosystem Integration
- [ ] Multi-language RPC client libraries
- [ ] Grafana dashboard templates
- [ ] Docker and Kubernetes deployments
- [ ] Cloud provider integrations

## Comparison with Full Node

| Feature | Omne Validator | Full Blockchain Node |
|---------|----------------|----------------------|
| Genesis creation | ❌ No | ✅ Yes |
| Transaction pool | ❌ No | ✅ Yes |
| Block production | ✅ Yes | ✅ Yes |
| State management | ⚠️ Minimal | ✅ Complete |
| RPC APIs | ⚠️ Validator-focused | ✅ Complete |
| P2P networking | ✅ Yes | ✅ Yes |
| Binary size | ⚠️ ~20MB | ❌ ~100MB+ |
| Memory usage | ✅ < 2GB | ❌ 8GB+ |
| Startup time | ✅ < 30s | ❌ 5+ minutes |

This architecture provides a clean, focused implementation that validators can run efficiently while maintaining full compatibility with the Omne network.
