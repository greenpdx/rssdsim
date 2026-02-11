/// Protocol module for external communication
///
/// This module provides protocol implementations for:
/// - MCP (Model Context Protocol): For LLM/AI agent integration
/// - A2A (Agent-to-Agent): For distributed agent communication

pub mod mcp;
pub mod a2a;

pub use mcp::{McpServer, McpClient, McpMessage};
pub use a2a::{A2aNode, A2aMessage, A2aTransport};
