//! MCP (Model Context Protocol) server exposing Klask's code search to AI agents.
//!
//! Implements a stateless, tools-only MCP server over the Streamable HTTP
//! transport: a single `POST /mcp` endpoint answering each JSON-RPC message
//! with a plain JSON response (no SSE streams, no sessions). Authentication
//! reuses the standard `Authorization: Bearer <token>` flow.
//!
//! See `docs/MCP_SERVER_PLAN.md` for the design rationale.

pub mod protocol;
pub mod tools;

use crate::auth::extractors::{AppState, AuthenticatedUser};
use axum::{
    Router,
    extract::State,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::post,
};
use protocol::{INVALID_PARAMS, INVALID_REQUEST, JsonRpcRequest, JsonRpcResponse, METHOD_NOT_FOUND, PARSE_ERROR};
use serde_json::{Value, json};

/// Router exposing the MCP endpoint at `/mcp`.
pub fn create_router() -> Router<AppState> {
    Router::new().route("/mcp", post(handle_post).fallback(handle_unsupported_method))
}

/// Non-POST methods: this stateless server has no server-initiated streams (GET)
/// and no sessions to terminate (DELETE).
async fn handle_unsupported_method() -> Response {
    (
        StatusCode::METHOD_NOT_ALLOWED,
        [(header::ALLOW, "POST")],
        "MCP endpoint only accepts POST",
    )
        .into_response()
}

async fn handle_post(_auth: AuthenticatedUser, State(state): State<AppState>, body: String) -> Response {
    // -32700 is reserved for malformed JSON; a well-formed body that is not a
    // valid Request object (missing method/jsonrpc, wrong types) is -32600.
    let value: Value = match serde_json::from_str(&body) {
        Ok(value) => value,
        Err(e) => {
            return json_response(JsonRpcResponse::error(
                Value::Null,
                PARSE_ERROR,
                format!("Parse error: {e}"),
            ));
        }
    };

    let request: JsonRpcRequest = match serde::Deserialize::deserialize(&value) {
        Ok(request) => request,
        Err(e) => {
            let id = value.get("id").cloned().unwrap_or(Value::Null);
            return json_response(JsonRpcResponse::error(
                id,
                INVALID_REQUEST,
                format!("Invalid request: {e}"),
            ));
        }
    };

    if request.jsonrpc != protocol::JSONRPC_VERSION {
        // Never answer an id-less message, even an invalid one
        let Some(id) = request.id.clone() else {
            return StatusCode::ACCEPTED.into_response();
        };
        return json_response(JsonRpcResponse::error(
            id,
            INVALID_REQUEST,
            "Unsupported JSON-RPC version",
        ));
    }

    // Notifications (no id) must not receive a JSON-RPC response
    if request.is_notification() {
        return StatusCode::ACCEPTED.into_response();
    }

    let id = request.id.clone().unwrap_or(Value::Null);
    tracing::debug!("MCP request: method={}", request.method);

    let response = match request.method.as_str() {
        "initialize" => {
            let requested_version = request.params.get("protocolVersion").and_then(Value::as_str);
            JsonRpcResponse::success(id, protocol::initialize_result(requested_version))
        }
        "ping" => JsonRpcResponse::success(id, json!({})),
        "tools/list" => JsonRpcResponse::success(id, json!({ "tools": tools::tool_definitions() })),
        "tools/call" => handle_tools_call(&state, id, &request.params).await,
        method => JsonRpcResponse::error(id, METHOD_NOT_FOUND, format!("Method not found: {method}")),
    };

    json_response(response)
}

async fn handle_tools_call(state: &AppState, id: Value, params: &Value) -> JsonRpcResponse {
    let Some(name) = params.get("name").and_then(Value::as_str) else {
        return JsonRpcResponse::error(id, INVALID_PARAMS, "Missing tool 'name' in params");
    };

    let default_arguments = Value::Null;
    let arguments = params.get("arguments").unwrap_or(&default_arguments);

    match tools::call_tool(state, name, arguments).await {
        Ok(result) => JsonRpcResponse::success(id, result),
        Err(tools::ToolCallError::UnknownTool(tool)) => {
            JsonRpcResponse::error(id, INVALID_PARAMS, format!("Unknown tool: {tool}"))
        }
        Err(tools::ToolCallError::InvalidParams(message)) => JsonRpcResponse::error(id, INVALID_PARAMS, message),
    }
}

fn json_response(response: JsonRpcResponse) -> Response {
    (StatusCode::OK, axum::Json(response)).into_response()
}
