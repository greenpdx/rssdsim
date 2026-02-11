/// Model Context Protocol (MCP) Implementation
///
/// MCP enables AI agents and LLMs to interact with the simulation system,
/// query model state, run simulations, and analyze results.
///
/// Reference: https://modelcontextprotocol.io/

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// MCP Protocol Version
pub const MCP_VERSION: &str = "2024-11-05";

/// MCP Message Types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum McpMessage {
    /// Initialize connection
    Initialize {
        protocol_version: String,
        capabilities: McpCapabilities,
        client_info: ClientInfo,
    },

    /// Request to list available tools/resources
    ListResources {
        #[serde(skip_serializing_if = "Option::is_none")]
        cursor: Option<String>,
    },

    /// Request to read a resource
    ReadResource {
        uri: String,
    },

    /// List available tools
    ListTools {},

    /// Call a tool
    CallTool {
        name: String,
        arguments: HashMap<String, serde_json::Value>,
    },

    /// Response message
    Response {
        request_id: String,
        result: McpResult,
    },

    /// Error message
    Error {
        request_id: String,
        code: i32,
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        data: Option<serde_json::Value>,
    },

    /// Notification (no response expected)
    Notification {
        method: String,
        params: serde_json::Value,
    },
}

/// MCP Capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<ResourceCapabilities>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolCapabilities>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompts: Option<PromptCapabilities>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sampling: Option<SamplingCapabilities>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceCapabilities {
    pub subscribe: bool,
    pub list_changed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCapabilities {
    pub list_changed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptCapabilities {
    pub list_changed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingCapabilities {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
}

/// MCP Result types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum McpResult {
    Resources {
        resources: Vec<Resource>,
        #[serde(skip_serializing_if = "Option::is_none")]
        next_cursor: Option<String>,
    },
    ResourceContent {
        contents: Vec<ResourceContent>,
    },
    Tools {
        tools: Vec<Tool>,
    },
    ToolResult {
        content: Vec<ToolContent>,
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
    },
}

/// Resource definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub uri: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

/// Resource content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceContent {
    pub uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    #[serde(flatten)]
    pub content: ResourceContentType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResourceContentType {
    Text { text: String },
    Blob { blob: String }, // base64 encoded
}

/// Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value, // JSON Schema
}

/// Tool execution result content
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ToolContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image { data: String, mime_type: String },
    #[serde(rename = "resource")]
    Resource { resource: ResourceContent },
}

/// MCP Server implementation for rsedsim
pub struct McpServer {
    capabilities: McpCapabilities,
    resources: Vec<Resource>,
    tools: Vec<Tool>,
}

impl McpServer {
    /// Create a new MCP server instance
    pub fn new() -> Self {
        Self {
            capabilities: McpCapabilities {
                resources: Some(ResourceCapabilities {
                    subscribe: true,
                    list_changed: true,
                }),
                tools: Some(ToolCapabilities {
                    list_changed: true,
                }),
                prompts: None,
                sampling: None,
            },
            resources: Self::default_resources(),
            tools: Self::default_tools(),
        }
    }

    /// Default resources exposed by rsedsim
    fn default_resources() -> Vec<Resource> {
        vec![
            Resource {
                uri: "rsedsim://models/list".to_string(),
                name: "Available Models".to_string(),
                description: Some("List of all loaded simulation models".to_string()),
                mime_type: Some("application/json".to_string()),
            },
            Resource {
                uri: "rsedsim://simulation/state".to_string(),
                name: "Current Simulation State".to_string(),
                description: Some("Current state of running simulation".to_string()),
                mime_type: Some("application/json".to_string()),
            },
            Resource {
                uri: "rsedsim://results/latest".to_string(),
                name: "Latest Results".to_string(),
                description: Some("Most recent simulation results".to_string()),
                mime_type: Some("application/json".to_string()),
            },
        ]
    }

    /// Default tools exposed by rsedsim
    fn default_tools() -> Vec<Tool> {
        vec![
            Tool {
                name: "run_simulation".to_string(),
                description: "Run a system dynamics simulation with specified parameters".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "model": {
                            "type": "string",
                            "description": "Model file path or URI"
                        },
                        "parameters": {
                            "type": "object",
                            "description": "Parameter overrides as key-value pairs"
                        },
                        "time_config": {
                            "type": "object",
                            "properties": {
                                "start": {"type": "number"},
                                "stop": {"type": "number"},
                                "dt": {"type": "number"}
                            }
                        }
                    },
                    "required": ["model"]
                }),
            },
            Tool {
                name: "analyze_model".to_string(),
                description: "Analyze model structure, loops, and dependencies".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "model": {
                            "type": "string",
                            "description": "Model file path or URI"
                        },
                        "analysis_type": {
                            "type": "string",
                            "enum": ["structure", "loops", "sensitivity", "equilibrium"],
                            "description": "Type of analysis to perform"
                        }
                    },
                    "required": ["model", "analysis_type"]
                }),
            },
            Tool {
                name: "sensitivity_analysis".to_string(),
                description: "Perform parameter sensitivity analysis".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "model": {"type": "string"},
                        "parameters": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Parameters to vary"
                        },
                        "ranges": {
                            "type": "object",
                            "description": "Min/max ranges for each parameter"
                        },
                        "samples": {
                            "type": "integer",
                            "description": "Number of samples to generate"
                        }
                    },
                    "required": ["model", "parameters"]
                }),
            },
            Tool {
                name: "get_variable_timeseries".to_string(),
                description: "Extract time series data for specific variables".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "simulation_id": {"type": "string"},
                        "variables": {
                            "type": "array",
                            "items": {"type": "string"}
                        }
                    },
                    "required": ["simulation_id", "variables"]
                }),
            },
        ]
    }

    /// Handle incoming MCP message
    pub async fn handle_message(&mut self, message: McpMessage) -> Result<McpMessage, McpError> {
        // TODO: Implement message handling logic
        match message {
            McpMessage::ListResources { .. } => {
                Ok(McpMessage::Response {
                    request_id: "TODO".to_string(),
                    result: McpResult::Resources {
                        resources: self.resources.clone(),
                        next_cursor: None,
                    },
                })
            }
            McpMessage::ListTools { .. } => {
                Ok(McpMessage::Response {
                    request_id: "TODO".to_string(),
                    result: McpResult::Tools {
                        tools: self.tools.clone(),
                    },
                })
            }
            _ => Err(McpError::NotImplemented),
        }
    }

    /// Start MCP server on stdio
    pub async fn serve_stdio(&mut self) -> Result<(), McpError> {
        // TODO: Implement stdio-based JSON-RPC server
        todo!("Implement stdio transport")
    }

    /// Start MCP server on HTTP SSE
    pub async fn serve_http(&mut self, addr: &str) -> Result<(), McpError> {
        // TODO: Implement HTTP SSE transport
        todo!("Implement HTTP SSE transport")
    }
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new()
    }
}

/// MCP Client for connecting to other MCP servers
pub struct McpClient {
    server_info: Option<ClientInfo>,
    capabilities: Option<McpCapabilities>,
}

impl McpClient {
    /// Create new MCP client
    pub fn new() -> Self {
        Self {
            server_info: None,
            capabilities: None,
        }
    }

    /// Connect to MCP server via stdio
    pub async fn connect_stdio(&mut self) -> Result<(), McpError> {
        // TODO: Implement stdio client
        todo!("Implement stdio client")
    }

    /// Connect to MCP server via HTTP SSE
    pub async fn connect_http(&mut self, url: &str) -> Result<(), McpError> {
        // TODO: Implement HTTP SSE client
        todo!("Implement HTTP SSE client")
    }

    /// Send message to server
    pub async fn send(&self, message: McpMessage) -> Result<McpMessage, McpError> {
        // TODO: Implement message sending
        todo!("Implement message sending")
    }
}

impl Default for McpClient {
    fn default() -> Self {
        Self::new()
    }
}

/// MCP Error types
#[derive(Debug, Clone)]
pub enum McpError {
    ParseError(String),
    InvalidRequest(String),
    MethodNotFound(String),
    InvalidParams(String),
    InternalError(String),
    NotImplemented,
    TransportError(String),
}

impl std::fmt::Display for McpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            McpError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            McpError::InvalidRequest(msg) => write!(f, "Invalid request: {}", msg),
            McpError::MethodNotFound(msg) => write!(f, "Method not found: {}", msg),
            McpError::InvalidParams(msg) => write!(f, "Invalid params: {}", msg),
            McpError::InternalError(msg) => write!(f, "Internal error: {}", msg),
            McpError::NotImplemented => write!(f, "Not implemented"),
            McpError::TransportError(msg) => write!(f, "Transport error: {}", msg),
        }
    }
}

impl std::error::Error for McpError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_server_creation() {
        let server = McpServer::new();
        assert!(!server.tools.is_empty());
        assert!(!server.resources.is_empty());
    }

    #[test]
    fn test_message_serialization() {
        let msg = McpMessage::ListTools {};
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("ListTools"));
    }
}
