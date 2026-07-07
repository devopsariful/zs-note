//! Integration tests for the MCP (Model Context Protocol) subsystem.
//!
//! Covers JSON‑RPC message parsing, creation helpers and the tool router.

use serde_json::json;
use zs_note_lib::mcp::protocol::{
    create_call_tool_request, create_initialize_request, create_list_tools_request,
    parse_jsonrpc_message, JsonRpcMessage,
};
use zs_note_lib::mcp::router::{route_ai_tool_call, ConfirmationRule, McpToolRouter, RouteConfig};

// ---------------------------------------------------------------------------
// Parse JSON‑RPC requests
// ---------------------------------------------------------------------------

#[test]
fn test_parse_jsonrpc_request() {
    let raw = r#"{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}"#;
    let msg = parse_jsonrpc_message(raw).expect("valid request should parse");

    match msg {
        JsonRpcMessage::Request {
            jsonrpc,
            id,
            method,
            params,
        } => {
            assert_eq!(jsonrpc, "2.0");
            assert_eq!(id, 1);
            assert_eq!(method, "tools/list");
            assert!(params.is_some());
        }
        other => panic!("expected Request, got {:?}", other),
    }
}

#[test]
fn test_parse_jsonrpc_request_with_params() {
    let raw = r#"{"jsonrpc":"2.0","id":42,"method":"tools/call","params":{"name":"echo","arguments":{"text":"hello"}}}"#;
    let msg = parse_jsonrpc_message(raw).expect("should parse");

    match msg {
        JsonRpcMessage::Request { id, method, .. } => {
            assert_eq!(id, 42);
            assert_eq!(method, "tools/call");
        }
        other => panic!("expected Request, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// Parse JSON‑RPC responses
// ---------------------------------------------------------------------------

#[test]
fn test_parse_jsonrpc_response() {
    let raw = r#"{"jsonrpc":"2.0","id":1,"result":{"content":["done"]}}"#;
    let msg = parse_jsonrpc_message(raw).expect("valid response should parse");

    match msg {
        JsonRpcMessage::Success {
            jsonrpc,
            id,
            ref result,
        } => {
            assert_eq!(jsonrpc, "2.0");
            assert_eq!(id, 1);
            assert_eq!(result["content"][0], "done");
        }
        other => panic!("expected Success, got {:?}", other),
    }
}

#[test]
fn test_parse_jsonrpc_response_with_array_result() {
    let raw = r#"{"jsonrpc":"2.0","id":2,"result":[1,2,3]}"#;
    let msg = parse_jsonrpc_message(raw).expect("should parse");
    match msg {
        JsonRpcMessage::Success { id, .. } => assert_eq!(id, 2),
        other => panic!("expected Success, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// Parse JSON‑RPC error responses
// ---------------------------------------------------------------------------

#[test]
fn test_parse_jsonrpc_error_response() {
    let raw = r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32601,"message":"Method not found"}}"#;
    let msg = parse_jsonrpc_message(raw).expect("error response should parse");

    match msg {
        JsonRpcMessage::Error { id, ref error, .. } => {
            assert_eq!(id, 1);
            assert_eq!(error.code, -32601);
            assert_eq!(error.message, "Method not found");
        }
        other => panic!("expected Error, got {:?}", other),
    }
}

#[test]
fn test_parse_jsonrpc_error_with_data() {
    let raw = r#"{"jsonrpc":"2.0","id":5,"error":{"code":-32000,"message":"Something broke","data":{"detail":"out of memory"}}}"#;
    let msg = parse_jsonrpc_message(raw).expect("should parse");

    match msg {
        JsonRpcMessage::Error { ref error, .. } => {
            assert_eq!(error.code, -32000);
            assert!(error.data.is_some());
            assert_eq!(error.data.as_ref().unwrap()["detail"], "out of memory");
        }
        other => panic!("expected Error, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// Parse JSON‑RPC notifications
// ---------------------------------------------------------------------------

#[test]
fn test_jsonrpc_notification() {
    let raw = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
    let msg = parse_jsonrpc_message(raw).expect("notification should parse");

    match msg {
        JsonRpcMessage::Notification {
            jsonrpc,
            method,
            params,
        } => {
            assert_eq!(jsonrpc, "2.0");
            assert_eq!(method, "notifications/initialized");
            assert!(params.is_none());
        }
        other => panic!("expected Notification, got {:?}", other),
    }
}

#[test]
fn test_jsonrpc_notification_with_params() {
    let raw = r#"{"jsonrpc":"2.0","method":"notifications/progress","params":{"progressToken":1,"progress":0.5}}"#;
    let msg = parse_jsonrpc_message(raw).expect("should parse");

    match msg {
        JsonRpcMessage::Notification { method, params, .. } => {
            assert_eq!(method, "notifications/progress");
            let p = params.expect("params should be present");
            assert_eq!(p["progressToken"], 1);
            assert_eq!(p["progress"], 0.5);
        }
        other => panic!("expected Notification, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// Parse invalid messages
// ---------------------------------------------------------------------------

#[test]
fn test_jsonrpc_invalid_message_not_json() {
    let result = parse_jsonrpc_message("this is not json");
    assert!(result.is_err(), "invalid JSON should fail");
}

#[test]
fn test_jsonrpc_invalid_message_empty_object() {
    // Empty object has no id, method, result or error
    let result = parse_jsonrpc_message("{}");
    assert!(result.is_err(), "empty object should be unrecognized");
}

#[test]
fn test_jsonrpc_invalid_message_both_result_and_error() {
    let raw = r#"{"jsonrpc":"2.0","id":1,"result":"ok","error":{"code":-1,"message":"err"}}"#;
    let result = parse_jsonrpc_message(raw);
    // The current parser uses first-match semantics; both is ambiguous
    assert!(
        result.is_ok(),
        "parser should handle this gracefully (may match Error first)"
    );
}

#[test]
fn test_jsonrpc_invalid_message_null() {
    let result = parse_jsonrpc_message("null");
    assert!(result.is_err(), "JSON null should fail");
}

// ---------------------------------------------------------------------------
// Helper: create request
// ---------------------------------------------------------------------------

#[test]
fn test_create_initialize_request() {
    let req = create_initialize_request(1);
    assert_eq!(req.jsonrpc, "2.0");
    assert_eq!(req.id, 1);
    assert_eq!(req.method, "initialize");
    assert!(req.params.is_some());
    let p = req.params.unwrap();
    assert_eq!(p["protocolVersion"], "2024-11-05");
    assert_eq!(p["clientInfo"]["name"], "zarishnote");
}

#[test]
fn test_create_list_tools_request() {
    let req = create_list_tools_request(10);
    assert_eq!(req.id, 10);
    assert_eq!(req.method, "tools/list");
}

#[test]
fn test_create_call_tool_request() {
    let req = create_call_tool_request(3, "echo", json!({"text": "hello"}));
    assert_eq!(req.id, 3);
    assert_eq!(req.method, "tools/call");
    let p = req.params.unwrap();
    assert_eq!(p["name"], "echo");
    assert_eq!(p["arguments"]["text"], "hello");
}

// ---------------------------------------------------------------------------
// MCP tool router
// ---------------------------------------------------------------------------

#[test]
fn test_mcp_tool_router_register_and_get() {
    let mut router = McpToolRouter::new();

    router.register_route(
        "server1/tool1",
        RouteConfig {
            server_name: "server1".into(),
            tool_name: "tool1".into(),
            requires_confirmation: false,
            timeout_ms: 5000,
        },
    );

    let route = router
        .get_route("server1", "tool1")
        .expect("route should be found");
    assert_eq!(route.server_name, "server1");
    assert_eq!(route.tool_name, "tool1");
    assert!(!route.requires_confirmation);
}

#[test]
fn test_mcp_tool_router_route_not_found() {
    let router = McpToolRouter::new();
    assert!(
        router.get_route("nowhere", "nothing").is_none(),
        "unregistered route should return None"
    );
}

#[test]
fn test_mcp_tool_router_wildcard_tool() {
    let mut router = McpToolRouter::new();
    router.register_route(
        "server1/*",
        RouteConfig {
            server_name: "server1".into(),
            tool_name: "*".into(),
            requires_confirmation: true,
            timeout_ms: 10_000,
        },
    );

    // Any tool on server1 should match the wildcard
    let route = router
        .get_route("server1", "any_tool")
        .expect("wildcard should match");
    assert!(route.requires_confirmation);
}

#[test]
fn test_mcp_tool_router_exact_beats_wildcard() {
    let mut router = McpToolRouter::new();
    router.register_route(
        "server1/*",
        RouteConfig {
            server_name: "server1".into(),
            tool_name: "*".into(),
            requires_confirmation: true,
            timeout_ms: 1000,
        },
    );
    router.register_route(
        "server1/safe_tool",
        RouteConfig {
            server_name: "server1".into(),
            tool_name: "safe_tool".into(),
            requires_confirmation: false,
            timeout_ms: 1000,
        },
    );

    let route = router
        .get_route("server1", "safe_tool")
        .expect("exact route should match");
    assert!(
        !route.requires_confirmation,
        "exact route should take precedence"
    );
}

// ---------------------------------------------------------------------------
// Route AI tool call
// ---------------------------------------------------------------------------

#[test]
fn test_route_ai_tool_call_registered() {
    let mut router = McpToolRouter::new();
    router.register_route(
        "fs/read",
        RouteConfig {
            server_name: "fs".into(),
            tool_name: "read".into(),
            requires_confirmation: false,
            timeout_ms: 5000,
        },
    );

    let action = route_ai_tool_call(&router, "fs", "read", json!({"path": "/tmp"}))
        .expect("routing should succeed");
    assert_eq!(action.config.server_name, "fs");
    assert_eq!(action.config.tool_name, "read");
    assert!(!action.config.requires_confirmation);
}

#[test]
fn test_route_ai_tool_call_unregistered() {
    let router = McpToolRouter::new();
    let action = route_ai_tool_call(&router, "unknown", "unknown_tool", json!({}))
        .expect("unregistered tools should still route with defaults");
    assert_eq!(action.config.server_name, "unknown");
    assert_eq!(action.config.timeout_ms, 30_000);
}

// ---------------------------------------------------------------------------
// Requires confirmation – sensitive tools
// ---------------------------------------------------------------------------

#[test]
fn test_requires_confirmation_sensitive() {
    let router = McpToolRouter::new();

    // Sensitive tools should require confirmation by default
    assert!(router.requires_confirmation("any", "write", &json!({})));
    assert!(router.requires_confirmation("any", "delete", &json!({})));
    assert!(router.requires_confirmation("any", "execute", &json!({})));
    assert!(router.requires_confirmation("any", "run", &json!({})));
    assert!(router.requires_confirmation("any", "bash", &json!({})));
    assert!(router.requires_confirmation("any", "sql", &json!({})));
    assert!(router.requires_confirmation("any", "rm", &json!({})));
}

#[test]
fn test_requires_confirmation_non_sensitive() {
    let router = McpToolRouter::new();

    // Non‑sensitive tools should NOT require confirmation by default
    assert!(!router.requires_confirmation("any", "read", &json!({})));
    assert!(!router.requires_confirmation("any", "list", &json!({})));
    assert!(!router.requires_confirmation("any", "search", &json!({})));
    assert!(!router.requires_confirmation("any", "ping", &json!({})));
}

#[test]
fn test_requires_confirmation_tool_name_case_insensitive() {
    let router = McpToolRouter::new();

    assert!(router.requires_confirmation("srv", "DELETE", &json!({})));
    assert!(router.requires_confirmation("srv", "Write", &json!({})));
}

#[test]
fn test_requires_confirmation_with_rules() {
    let mut router = McpToolRouter::new();

    router.add_confirmation_rule(ConfirmationRule {
        tool_pattern: "read".into(),
        action_pattern: "sensitive".into(),
        require_confirm: true,
    });

    // read + "sensitive" in args → needs confirmation
    assert!(router.requires_confirmation("any", "read", &json!({"path": "sensitive_file"})));
    // read + no sensitive args → no confirmation
    assert!(!router.requires_confirmation("any", "read", &json!({"path": "normal"})));
}

#[test]
fn test_requires_confirmation_route_config_overrides() {
    let mut router = McpToolRouter::new();
    router.register_route(
        "safe/write",
        RouteConfig {
            server_name: "safe".into(),
            tool_name: "write".into(),
            requires_confirmation: false,
            timeout_ms: 5000,
        },
    );

    // The explicit route says no confirmation needed despite tool name "write"
    assert!(
        !router.requires_confirmation("safe", "write", &json!({})),
        "explicit route config should override sensitive default"
    );
}
