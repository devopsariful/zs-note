use std::collections::HashMap;

use anyhow::{bail, Result};
use serde_json::Value;
use tracing::info;

pub struct McpToolRouter {
    routes: HashMap<String, RouteConfig>,
    confirmations: Vec<ConfirmationRule>,
}

#[derive(Debug, Clone)]
pub struct RouteConfig {
    pub server_name: String,
    pub tool_name: String,
    pub requires_confirmation: bool,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone)]
pub struct ConfirmationRule {
    pub tool_pattern: String,
    pub action_pattern: String,
    pub require_confirm: bool,
}

impl McpToolRouter {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
            confirmations: Vec::new(),
        }
    }

    pub fn register_route(&mut self, route_key: &str, config: RouteConfig) {
        info!(
            "Registered MCP route: {} -> {}/{}",
            route_key, config.server_name, config.tool_name
        );
        self.routes.insert(route_key.to_string(), config);
    }

    pub fn add_confirmation_rule(&mut self, rule: ConfirmationRule) {
        self.confirmations.push(rule);
    }

    pub fn get_route(&self, server: &str, tool: &str) -> Option<&RouteConfig> {
        let exact = format!("{}/{}", server, tool);
        self.routes.get(&exact).or_else(|| {
            self.routes
                .iter()
                .find(|(key, _)| {
                    if let Some(tool_part) = key.split('/').nth(1) {
                        tool_part == "*" || tool_part == tool
                    } else {
                        false
                    }
                })
                .map(|(_, config)| config)
        })
    }

    pub fn requires_confirmation(&self, server: &str, tool: &str, args: &Value) -> bool {
        for rule in &self.confirmations {
            let tool_matches = tool.contains(rule.tool_pattern.trim_matches('*'));
            if !tool_matches {
                continue;
            }

            if !rule.action_pattern.is_empty() && rule.action_pattern != "*" {
                let args_str = serde_json::to_string(args).unwrap_or_default();
                if !args_str.contains(&rule.action_pattern) {
                    continue;
                }
            }

            return rule.require_confirm;
        }

        if let Some(route) = self.get_route(server, tool) {
            return route.requires_confirmation;
        }

        let sensitive_tools = [
            "write", "delete", "remove", "rm", "exec", "execute", "run", "bash", "shell",
            "command", "sql", "query",
        ];

        sensitive_tools
            .iter()
            .any(|s| tool.to_lowercase().contains(s))
    }

    pub fn format_tool_result(&self, result: Value, _tool_name: &str) -> Result<String> {
        if result.is_null() {
            return Ok("(no output)".to_string());
        }

        if let Some(text) = result["content"].as_array() {
            let parts: Vec<String> = text
                .iter()
                .filter_map(|part| {
                    let text = part["text"].as_str()?;
                    if text.is_empty() {
                        None
                    } else {
                        Some(text.to_string())
                    }
                })
                .collect();

            if !parts.is_empty() {
                return Ok(parts.join("\n"));
            }
        }

        if let Some(text) = result.as_str() {
            return Ok(text.to_string());
        }

        Ok(serde_json::to_string_pretty(&result)?)
    }
}

impl Default for McpToolRouter {
    fn default() -> Self {
        Self::new()
    }
}

pub fn route_ai_tool_call(
    router: &McpToolRouter,
    server: &str,
    tool: &str,
    args: Value,
) -> Result<RouteAction> {
    let route = router
        .get_route(server, tool)
        .cloned()
        .unwrap_or(RouteConfig {
            server_name: server.to_string(),
            tool_name: tool.to_string(),
            requires_confirmation: router.requires_confirmation(server, tool, &args),
            timeout_ms: 30_000,
        });

    Ok(RouteAction {
        config: route,
        args,
    })
}

#[derive(Debug, Clone)]
pub struct RouteAction {
    pub config: RouteConfig,
    pub args: Value,
}

pub async fn execute_with_confirmation<F, Fut>(action: RouteAction, confirm_fn: F) -> Result<Value>
where
    F: FnOnce(RouteAction) -> Fut,
    Fut: std::future::Future<Output = Result<bool>>,
{
    if action.config.requires_confirmation {
        let confirmed = confirm_fn(action.clone()).await?;
        if !confirmed {
            bail!("Tool call rejected by user");
        }
    }

    Ok(serde_json::json!({
        "status": "approved",
        "tool": action.config.tool_name,
        "server": action.config.server_name,
    }))
}
