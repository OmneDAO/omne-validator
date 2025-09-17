//! JSON-RPC server for validator queries and control

use crate::config::ValidatorConfig;
use crate::consensus::PoVERAValidator;
use crate::p2p::P2PNetwork;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tracing::{info, debug, warn};

/// JSON-RPC server for validator API
pub struct RpcServer {
    config: ValidatorConfig,
    consensus: Arc<PoVERAValidator>,
    p2p_network: Arc<P2PNetwork>,
    bind_address: SocketAddr,
}

/// RPC method handler
type RpcHandler = fn(&RpcServer, &[Value]) -> Result<Value>;

/// RPC status information  
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcStatus {
    pub is_running: bool,
    pub bind_address: String,
    pub active_connections: usize,
    pub total_requests: u64,
}

/// JSON-RPC request structure
#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Option<Vec<Value>>,
    id: Option<Value>,
}

/// JSON-RPC response structure
#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
    id: Option<Value>,
}

/// JSON-RPC error structure
#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

impl RpcServer {
    /// Create a new RPC server
    pub async fn new(
        config: &ValidatorConfig,
        consensus: Arc<PoVERAValidator>,
        p2p_network: Arc<P2PNetwork>,
    ) -> Result<Self> {
        let bind_address: SocketAddr = format!("{}:{}", config.rpc.bind_address, config.rpc.port)
            .parse()?;

        info!("🌐 Initializing RPC server");
        info!("   Bind Address: {}", bind_address);
        info!("   HTTP Enabled: {}", config.rpc.enable_http);
        info!("   WebSocket Enabled: {}", config.rpc.enable_ws);

        Ok(Self {
            config: config.clone(),
            consensus,
            p2p_network,
            bind_address,
        })
    }

    /// Start the RPC server
    pub async fn start(&self, mut shutdown: broadcast::Receiver<()>) -> Result<()> {
        info!("🚀 Starting RPC server on {}", self.bind_address);

        if !self.config.rpc.enable_http {
            info!("📵 HTTP RPC disabled, waiting for shutdown signal");
            let _ = shutdown.recv().await;
            return Ok(());
        }

        let listener = TcpListener::bind(self.bind_address).await?;
        info!("✅ RPC server listening on {}", self.bind_address);

        loop {
            tokio::select! {
                result = listener.accept() => {
                    match result {
                        Ok((stream, addr)) => {
                            debug!("🔌 New RPC connection from {}", addr);
                            // TODO: Handle HTTP/WebSocket upgrade and process requests
                            let _ = stream; // Placeholder
                        }
                        Err(e) => {
                            warn!("Failed to accept RPC connection: {}", e);
                        }
                    }
                }
                
                _ = shutdown.recv() => {
                    info!("🛑 Shutting down RPC server");
                    break;
                }
            }
        }

        Ok(())
    }

    /// Process a JSON-RPC request
    async fn process_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let id = request.id.clone();

        // Validate JSON-RPC version
        if request.jsonrpc != "2.0" {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(JsonRpcError {
                    code: -32600,
                    message: "Invalid Request".to_string(),
                    data: None,
                }),
                id,
            };
        }

        // Get method handler
        let methods = self.get_rpc_methods();
        let params = request.params.unwrap_or_default();

        let result = match methods.get(&request.method) {
            Some(handler) => match handler(self, &params) {
                Ok(result) => Some(result),
                Err(e) => {
                    return JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32603,
                            message: format!("Internal error: {}", e),
                            data: None,
                        }),
                        id,
                    };
                }
            },
            None => {
                return JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32601,
                        message: "Method not found".to_string(),
                        data: None,
                    }),
                    id,
                };
            }
        };

        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result,
            error: None,
            id,
        }
    }

    /// Get available RPC methods
    fn get_rpc_methods(&self) -> HashMap<String, RpcHandler> {
        let mut methods: HashMap<String, RpcHandler> = HashMap::new();

        // Validator status methods
        methods.insert("validator_status".to_string(), Self::rpc_validator_status);
        methods.insert("consensus_status".to_string(), Self::rpc_consensus_status);
        methods.insert("p2p_status".to_string(), Self::rpc_p2p_status);
        
        // Network methods
        methods.insert("network_info".to_string(), Self::rpc_network_info);
        methods.insert("peer_list".to_string(), Self::rpc_peer_list);
        
        // Block and transaction methods
        methods.insert("latest_block".to_string(), Self::rpc_latest_block);
        methods.insert("block_by_height".to_string(), Self::rpc_block_by_height);
        
        // Utility methods
        methods.insert("health".to_string(), Self::rpc_health);
        methods.insert("version".to_string(), Self::rpc_version);

        methods
    }

    /// RPC method: validator_status
    fn rpc_validator_status(&self, _params: &[Value]) -> Result<Value> {
        // TODO: Get actual validator status
        Ok(json!({
            "is_validator": self.config.validator.is_validator,
            "stake": self.config.validator.validator_stake,
            "is_active": true,
            "uptime": "99.9%"
        }))
    }

    /// RPC method: consensus_status
    fn rpc_consensus_status(&self, _params: &[Value]) -> Result<Value> {
        // TODO: Get actual consensus status from consensus module
        Ok(json!({
            "commerce_epoch": 0,
            "security_epoch": 0,
            "commerce_height": 0,
            "security_height": 0
        }))
    }

    /// RPC method: p2p_status
    fn rpc_p2p_status(&self, _params: &[Value]) -> Result<Value> {
        // TODO: Get actual P2P status
        Ok(json!({
            "connected_peers": 0,
            "network_id": self.config.network.id,
            "local_peer_id": "placeholder"
        }))
    }

    /// RPC method: network_info
    fn rpc_network_info(&self, _params: &[Value]) -> Result<Value> {
        Ok(json!({
            "network_name": self.config.network.name,
            "network_id": self.config.network.id,
            "genesis_hash": self.config.network.genesis_hash,
            "commerce_block_time": self.config.network.chain_spec.commerce_block_time,
            "security_block_time": self.config.network.chain_spec.security_block_time
        }))
    }

    /// RPC method: peer_list
    fn rpc_peer_list(&self, _params: &[Value]) -> Result<Value> {
        Ok(json!({
            "peers": []
        }))
    }

    /// RPC method: latest_block
    fn rpc_latest_block(&self, _params: &[Value]) -> Result<Value> {
        Ok(json!({
            "height": 0,
            "hash": "bh_000000000000000000000000000000000000000000000000000000000000000000",
            "timestamp": chrono::Utc::now().timestamp()
        }))
    }

    /// RPC method: block_by_height
    fn rpc_block_by_height(&self, params: &[Value]) -> Result<Value> {
        if params.is_empty() {
            return Err(anyhow::anyhow!("Missing height parameter"));
        }

        let _height = params[0].as_u64()
            .ok_or_else(|| anyhow::anyhow!("Invalid height parameter"))?;

        Ok(json!({
            "height": _height,
            "hash": "bh_000000000000000000000000000000000000000000000000000000000000000000",
            "timestamp": chrono::Utc::now().timestamp()
        }))
    }

    /// RPC method: health
    fn rpc_health(&self, _params: &[Value]) -> Result<Value> {
        Ok(json!({
            "status": "healthy",
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }

    /// RPC method: version
    fn rpc_version(&self, _params: &[Value]) -> Result<Value> {
        Ok(json!({
            "version": env!("CARGO_PKG_VERSION"),
            "name": env!("CARGO_PKG_NAME"),
            "git_commit": "unknown" // TODO: Include git commit hash
        }))
    }

    /// Get RPC server status
    pub async fn status(&self) -> Result<RpcStatus> {
        Ok(RpcStatus {
            is_running: true,
            bind_address: self.bind_address.to_string(),
            active_connections: 0, // TODO: Track actual connections
            total_requests: 0,     // TODO: Track actual requests
        })
    }
}
