//! JSON-RPC 2.0 protocol types for the Agent API.

use serde::{Deserialize, Serialize};

/// A JSON-RPC 2.0 request.
#[derive(Debug, Deserialize)]
pub struct Request {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    pub method: String,
    #[serde(default)]
    pub params: serde_json::Value,
}

/// A JSON-RPC 2.0 response.
#[derive(Debug, Serialize)]
pub struct Response {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
}

#[derive(Debug, Serialize)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl Response {
    pub fn success(id: serde_json::Value, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: serde_json::Value, code: i32, message: String) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(RpcError {
                code,
                message,
                data: None,
            }),
        }
    }
}

/// Diagnostic returned by diagnostics.get.
#[derive(Debug, Serialize)]
pub struct AgentDiagnostic {
    pub code: String,
    pub severity: String,
    pub message: String,
    pub location: AgentLocation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix: Option<AgentFix>,
    pub suppressed: usize,
}

#[derive(Debug, Serialize)]
pub struct AgentLocation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Serialize)]
pub struct AgentFix {
    pub action: String,
    pub replacement: String,
    pub confidence: f64,
}

/// Completion item returned by completions.get.
#[derive(Debug, Serialize)]
pub struct AgentCompletion {
    pub label: String,
    pub kind: String,
    pub detail: Option<String>,
}
