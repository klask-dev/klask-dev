//! JSON-RPC 2.0 and MCP protocol types for the Klask MCP server.
//!
//! Implements the minimal protocol surface needed by a stateless, tools-only
//! MCP server over Streamable HTTP: `initialize`, `notifications/initialized`,
//! `tools/list`, `tools/call` and `ping`.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub const JSONRPC_VERSION: &str = "2.0";

/// Latest protocol revision this server implements.
pub const MCP_PROTOCOL_VERSION: &str = "2025-06-18";

/// Older revisions we can serve as-is (the subset of the protocol used by a
/// stateless tools-only server is identical across these revisions).
pub const SUPPORTED_PROTOCOL_VERSIONS: &[&str] = &["2025-06-18", "2025-03-26", "2024-11-05"];

// JSON-RPC 2.0 error codes
pub const PARSE_ERROR: i32 = -32700;
pub const INVALID_REQUEST: i32 = -32600;
pub const METHOD_NOT_FOUND: i32 = -32601;
pub const INVALID_PARAMS: i32 = -32602;

#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    #[serde(default)]
    pub params: Value,
    /// Absent for notifications.
    #[serde(default)]
    pub id: Option<Value>,
}

impl JsonRpcRequest {
    /// A request without an id is a notification and must not receive a response.
    pub fn is_notification(&self) -> bool {
        self.id.is_none()
    }
}

#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    pub id: Value,
}

impl JsonRpcResponse {
    pub fn success(id: Value, result: Value) -> Self {
        Self { jsonrpc: JSONRPC_VERSION, result: Some(result), error: None, id }
    }

    pub fn error(id: Value, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION,
            result: None,
            error: Some(JsonRpcError { code, message: message.into(), data: None }),
            id,
        }
    }
}

/// Negotiate the protocol version: echo the client's requested revision when we
/// support it, otherwise answer with our latest revision (per MCP spec).
pub fn negotiate_protocol_version(requested: Option<&str>) -> &'static str {
    match requested {
        Some(v) => SUPPORTED_PROTOCOL_VERSIONS.iter().find(|s| **s == v).copied().unwrap_or(MCP_PROTOCOL_VERSION),
        None => MCP_PROTOCOL_VERSION,
    }
}

/// Build the `initialize` result payload.
pub fn initialize_result(requested_version: Option<&str>) -> Value {
    json!({
        "protocolVersion": negotiate_protocol_version(requested_version),
        "capabilities": {
            "tools": {}
        },
        "serverInfo": {
            "name": "klask",
            "title": "Klask Code Search",
            "version": env!("CARGO_PKG_VERSION")
        },
        "instructions": "Klask indexes all the organization's Git repositories and branches. \
            Use search_code to find code across every indexed repository, get_file to read a \
            file found in search results (via its doc_address), list_repositories to see what \
            is indexed, and get_search_facets to discover available projects, branches and \
            file extensions before narrowing a search."
    })
}

/// Build a `tools/call` result carrying a JSON payload as text content.
pub fn tool_result(payload: &Value) -> Value {
    json!({
        "content": [{
            "type": "text",
            "text": payload.to_string()
        }],
        "isError": false
    })
}

/// Build a `tools/call` execution-error result (per MCP spec, tool failures are
/// reported in-band with `isError: true`, not as JSON-RPC errors).
pub fn tool_error(message: impl AsRef<str>) -> Value {
    json!({
        "content": [{
            "type": "text",
            "text": message.as_ref()
        }],
        "isError": true
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_request_with_id() {
        let req: JsonRpcRequest =
            serde_json::from_str(r#"{"jsonrpc":"2.0","method":"tools/list","id":1}"#).expect("valid request");
        assert_eq!(req.method, "tools/list");
        assert!(!req.is_notification());
        assert_eq!(req.id, Some(json!(1)));
    }

    #[test]
    fn test_parse_notification_without_id() {
        let req: JsonRpcRequest =
            serde_json::from_str(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#).expect("valid request");
        assert!(req.is_notification());
    }

    #[test]
    fn test_parse_request_with_string_id_and_params() {
        let req: JsonRpcRequest = serde_json::from_str(
            r#"{"jsonrpc":"2.0","method":"tools/call","id":"abc","params":{"name":"search_code"}}"#,
        )
        .expect("valid request");
        assert_eq!(req.id, Some(json!("abc")));
        assert_eq!(req.params["name"], "search_code");
    }

    #[test]
    fn test_success_response_serialization() {
        let resp = JsonRpcResponse::success(json!(1), json!({"ok": true}));
        let serialized = serde_json::to_value(&resp).expect("serializable");
        assert_eq!(serialized["jsonrpc"], "2.0");
        assert_eq!(serialized["result"]["ok"], true);
        assert!(serialized.get("error").is_none());
    }

    #[test]
    fn test_error_response_serialization() {
        let resp = JsonRpcResponse::error(json!(2), METHOD_NOT_FOUND, "Method not found");
        let serialized = serde_json::to_value(&resp).expect("serializable");
        assert_eq!(serialized["error"]["code"], METHOD_NOT_FOUND);
        assert!(serialized.get("result").is_none());
    }

    #[test]
    fn test_negotiate_protocol_version() {
        assert_eq!(negotiate_protocol_version(Some("2025-03-26")), "2025-03-26");
        assert_eq!(negotiate_protocol_version(Some("1999-01-01")), MCP_PROTOCOL_VERSION);
        assert_eq!(negotiate_protocol_version(None), MCP_PROTOCOL_VERSION);
    }

    #[test]
    fn test_tool_result_wraps_payload_as_text() {
        let result = tool_result(&json!({"total": 3}));
        assert_eq!(result["isError"], false);
        let text = result["content"][0]["text"].as_str().expect("text content");
        let parsed: Value = serde_json::from_str(text).expect("payload is valid JSON");
        assert_eq!(parsed["total"], 3);
    }

    #[test]
    fn test_tool_error_sets_is_error() {
        let result = tool_error("boom");
        assert_eq!(result["isError"], true);
        assert_eq!(result["content"][0]["text"], "boom");
    }
}
