/// Agent-to-Agent (A2A) Protocol Implementation
///
/// A2A enables distributed agent communication in hybrid models,
/// allowing agents to discover, message, and coordinate with each other
/// across network boundaries.
///
/// This protocol supports:
/// - Agent discovery and registration
/// - Direct agent-to-agent messaging
/// - Publish/subscribe patterns
/// - State synchronization
/// - Distributed simulation coordination

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;

/// A2A Protocol Version
pub const A2A_VERSION: &str = "0.1.0";

/// Unique agent identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId {
    pub namespace: String,  // e.g., "simulation1"
    pub id: String,         // e.g., "agent_42"
}

impl AgentId {
    pub fn new(namespace: &str, id: &str) -> Self {
        Self {
            namespace: namespace.to_string(),
            id: id.to_string(),
        }
    }

    pub fn to_string(&self) -> String {
        format!("{}:{}", self.namespace, self.id)
    }
}

/// A2A Message envelope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2aMessage {
    /// Message ID for tracking
    pub message_id: String,

    /// Source agent
    pub from: AgentId,

    /// Destination (None for broadcast)
    pub to: Option<AgentId>,

    /// Message timestamp (Unix timestamp in ms)
    pub timestamp: u64,

    /// Time-to-live (hops or seconds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<u32>,

    /// Message payload
    pub payload: A2aPayload,
}

/// A2A Message payload types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum A2aPayload {
    /// Register agent with directory
    Register {
        agent_info: AgentInfo,
    },

    /// Unregister agent
    Unregister,

    /// Discover agents by criteria
    Discover {
        query: DiscoveryQuery,
    },

    /// Discovery response
    DiscoveryResult {
        agents: Vec<AgentInfo>,
    },

    /// Direct message to another agent
    DirectMessage {
        content: serde_json::Value,
    },

    /// Publish to a topic
    Publish {
        topic: String,
        content: serde_json::Value,
    },

    /// Subscribe to a topic
    Subscribe {
        topic: String,
    },

    /// Unsubscribe from a topic
    Unsubscribe {
        topic: String,
    },

    /// State synchronization
    StateSync {
        state: HashMap<String, serde_json::Value>,
    },

    /// Request state from agent
    StateRequest {
        keys: Vec<String>,
    },

    /// State response
    StateResponse {
        state: HashMap<String, serde_json::Value>,
    },

    /// Simulation control messages
    SimControl {
        command: SimControlCommand,
    },

    /// Heartbeat/keepalive
    Heartbeat,

    /// Acknowledgment
    Ack {
        ack_message_id: String,
    },

    /// Error response
    Error {
        code: u32,
        message: String,
    },
}

/// Agent information for discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: AgentId,
    pub agent_type: String,
    pub capabilities: Vec<String>,
    pub attributes: HashMap<String, serde_json::Value>,
    pub endpoint: Option<String>, // Network endpoint if remote
}

/// Discovery query criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_type: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<HashMap<String, serde_json::Value>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
}

/// Simulation control commands
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "cmd")]
pub enum SimControlCommand {
    /// Start simulation
    Start {
        time_config: TimeConfig,
    },

    /// Pause simulation
    Pause,

    /// Resume simulation
    Resume,

    /// Stop simulation
    Stop,

    /// Step simulation by N steps
    Step {
        steps: u32,
    },

    /// Synchronization barrier
    Barrier {
        barrier_id: String,
        required_agents: Vec<AgentId>,
    },

    /// Report ready at barrier
    BarrierReady {
        barrier_id: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeConfig {
    pub start: f64,
    pub stop: f64,
    pub dt: f64,
}

/// Transport layer abstraction
#[async_trait::async_trait]
pub trait A2aTransport: Send + Sync {
    /// Send message
    async fn send(&self, message: A2aMessage) -> Result<(), A2aError>;

    /// Receive message (blocking or with timeout)
    async fn receive(&self) -> Result<A2aMessage, A2aError>;

    /// Broadcast message to all connected agents
    async fn broadcast(&self, message: A2aMessage) -> Result<(), A2aError>;
}

/// A2A Node - manages agent communication
pub struct A2aNode {
    /// Local agent ID
    agent_id: AgentId,

    /// Agent registry (for directory nodes)
    registry: HashMap<AgentId, AgentInfo>,

    /// Topic subscriptions
    subscriptions: HashMap<String, Vec<AgentId>>,

    /// Transport layer
    transport: Option<Box<dyn A2aTransport>>,

    /// Message handlers
    handlers: HashMap<String, Box<dyn Fn(&A2aMessage) + Send + Sync>>,
}

impl A2aNode {
    /// Create new A2A node
    pub fn new(agent_id: AgentId) -> Self {
        Self {
            agent_id,
            registry: HashMap::new(),
            subscriptions: HashMap::new(),
            transport: None,
            handlers: HashMap::new(),
        }
    }

    /// Set transport layer
    pub fn with_transport(mut self, transport: Box<dyn A2aTransport>) -> Self {
        self.transport = Some(transport);
        self
    }

    /// Register message handler
    pub fn register_handler<F>(&mut self, message_type: &str, handler: F)
    where
        F: Fn(&A2aMessage) + Send + Sync + 'static,
    {
        self.handlers.insert(message_type.to_string(), Box::new(handler));
    }

    /// Send message to another agent
    pub async fn send(&self, to: AgentId, payload: A2aPayload) -> Result<(), A2aError> {
        let message = A2aMessage {
            message_id: Self::generate_message_id(),
            from: self.agent_id.clone(),
            to: Some(to),
            timestamp: Self::current_timestamp(),
            ttl: Some(10),
            payload,
        };

        if let Some(transport) = &self.transport {
            transport.send(message).await
        } else {
            Err(A2aError::NoTransport)
        }
    }

    /// Broadcast message
    pub async fn broadcast(&self, payload: A2aPayload) -> Result<(), A2aError> {
        let message = A2aMessage {
            message_id: Self::generate_message_id(),
            from: self.agent_id.clone(),
            to: None,
            timestamp: Self::current_timestamp(),
            ttl: Some(5),
            payload,
        };

        if let Some(transport) = &self.transport {
            transport.broadcast(message).await
        } else {
            Err(A2aError::NoTransport)
        }
    }

    /// Discover agents
    pub async fn discover(&self, query: DiscoveryQuery) -> Result<Vec<AgentInfo>, A2aError> {
        // TODO: Implement discovery logic
        Ok(vec![])
    }

    /// Subscribe to topic
    pub async fn subscribe(&mut self, topic: String) -> Result<(), A2aError> {
        let payload = A2aPayload::Subscribe { topic: topic.clone() };
        self.broadcast(payload).await?;
        Ok(())
    }

    /// Publish to topic
    pub async fn publish(&self, topic: String, content: serde_json::Value) -> Result<(), A2aError> {
        let payload = A2aPayload::Publish { topic, content };
        self.broadcast(payload).await
    }

    /// Process incoming message
    pub async fn process_message(&mut self, message: A2aMessage) -> Result<(), A2aError> {
        // Call registered handlers
        let msg_type = match &message.payload {
            A2aPayload::Register { .. } => "register",
            A2aPayload::DirectMessage { .. } => "direct_message",
            A2aPayload::Publish { .. } => "publish",
            _ => "unknown",
        };

        if let Some(handler) = self.handlers.get(msg_type) {
            handler(&message);
        }

        // Built-in message handling
        match message.payload {
            A2aPayload::Register { agent_info } => {
                self.registry.insert(agent_info.id.clone(), agent_info);
            }
            A2aPayload::Subscribe { topic } => {
                self.subscriptions
                    .entry(topic)
                    .or_insert_with(Vec::new)
                    .push(message.from);
            }
            _ => {}
        }

        Ok(())
    }

    /// Start message processing loop
    pub async fn run(&mut self) -> Result<(), A2aError> {
        // TODO: Implement main event loop
        loop {
            if let Some(transport) = &self.transport {
                match transport.receive().await {
                    Ok(message) => {
                        self.process_message(message).await?;
                    }
                    Err(e) => {
                        eprintln!("Error receiving message: {}", e);
                    }
                }
            } else {
                return Err(A2aError::NoTransport);
            }
        }
    }

    // Helper functions
    fn generate_message_id() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        format!("msg_{}", now)
    }

    fn current_timestamp() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }
}

/// UDP Transport implementation (simple, connectionless)
pub struct UdpTransport {
    socket: std::sync::Arc<tokio::sync::Mutex<Option<tokio::net::UdpSocket>>>,
    peers: std::sync::Arc<tokio::sync::Mutex<Vec<SocketAddr>>>,
}

impl UdpTransport {
    pub fn new() -> Self {
        Self {
            socket: std::sync::Arc::new(tokio::sync::Mutex::new(None)),
            peers: std::sync::Arc::new(tokio::sync::Mutex::new(Vec::new())),
        }
    }

    pub async fn bind(&mut self, addr: &str) -> Result<(), A2aError> {
        // TODO: Implement UDP binding
        todo!("Implement UDP transport")
    }

    pub async fn add_peer(&self, addr: SocketAddr) {
        self.peers.lock().await.push(addr);
    }
}

#[async_trait::async_trait]
impl A2aTransport for UdpTransport {
    async fn send(&self, _message: A2aMessage) -> Result<(), A2aError> {
        // TODO: Implement UDP send
        todo!("Implement UDP send")
    }

    async fn receive(&self) -> Result<A2aMessage, A2aError> {
        // TODO: Implement UDP receive
        todo!("Implement UDP receive")
    }

    async fn broadcast(&self, _message: A2aMessage) -> Result<(), A2aError> {
        // TODO: Broadcast to all peers
        todo!("Implement UDP broadcast")
    }
}

/// A2A Error types
#[derive(Debug, Clone)]
pub enum A2aError {
    NoTransport,
    TransportError(String),
    SerializationError(String),
    TimeoutError,
    NotFound(String),
    InvalidMessage(String),
}

impl std::fmt::Display for A2aError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            A2aError::NoTransport => write!(f, "No transport configured"),
            A2aError::TransportError(msg) => write!(f, "Transport error: {}", msg),
            A2aError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            A2aError::TimeoutError => write!(f, "Timeout"),
            A2aError::NotFound(msg) => write!(f, "Not found: {}", msg),
            A2aError::InvalidMessage(msg) => write!(f, "Invalid message: {}", msg),
        }
    }
}

impl std::error::Error for A2aError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_id() {
        let id = AgentId::new("sim1", "agent_1");
        assert_eq!(id.to_string(), "sim1:agent_1");
    }

    #[test]
    fn test_message_creation() {
        let from = AgentId::new("sim1", "agent_1");
        let to = AgentId::new("sim1", "agent_2");

        let msg = A2aMessage {
            message_id: "test_123".to_string(),
            from,
            to: Some(to),
            timestamp: 1234567890,
            ttl: Some(10),
            payload: A2aPayload::Heartbeat,
        };

        assert_eq!(msg.message_id, "test_123");
    }
}
