//! MCP (Model Context Protocol) server exposing excalc's expression
//! evaluator as an `evaluate` tool over stdio, so AI agents that speak MCP
//! can call it directly instead of shelling out to the `excalc`/`calc` CLI.
//!
//! Built only with `--features mcp` (see the `[[bin]]` entry in
//! `Cargo.toml`); the plain `cargo install excalc` CLI install doesn't pull
//! in the extra async-runtime/MCP dependencies this binary needs.

use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, ContentBlock, Implementation, ServerCapabilities, ServerConfig},
    schemars, tool, tool_handler, tool_router,
    transport::stdio,
    ErrorData as McpError, ServerHandler, ServiceExt,
};

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct EvaluateRequest {
    #[schemars(
        description = "The math expression to evaluate, e.g. \"2 + 2 * 3\" or \"sqrt(16) + sin(pi/2)\""
    )]
    expression: String,
}

#[derive(Clone)]
struct ExcalcServer {
    // Read by macro-generated code in `#[tool_handler]`/`#[tool_router]`,
    // not directly by hand-written code, so rustc's dead-code analysis
    // (which only sees direct field reads) flags it as unused.
    #[allow(dead_code)]
    tool_router: ToolRouter<ExcalcServer>,
}

#[tool_router]
impl ExcalcServer {
    fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(
        description = "Evaluate a math expression and return the result. Supports arithmetic operators, trig/hyperbolic/log functions, statistics functions, and base-conversion functions (hex/oct/bin) returning text; see excalc's README for the full list."
    )]
    fn evaluate(
        &self,
        Parameters(EvaluateRequest { expression }): Parameters<EvaluateRequest>,
    ) -> Result<CallToolResult, McpError> {
        match excalc::evaluate_value_formatted(&expression) {
            Ok(text) => Ok(CallToolResult::success(vec![ContentBlock::text(text)])),
            // A malformed/undefined expression is the caller's input, not a
            // protocol-level error, so report it as a tool error rather than
            // an McpError (which would surface as a JSON-RPC error instead).
            Err(e) => Ok(CallToolResult::error(vec![ContentBlock::text(
                e.to_string(),
            )])),
        }
    }
}

#[tool_handler]
impl ServerHandler for ExcalcServer {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("excalc-mcp", env!("CARGO_PKG_VERSION")))
            .with_instructions(
                "Call `evaluate` with a math expression, e.g. \"2 + 2 * 3\" or \"sqrt(16) + sin(pi/2)\".",
            )
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let service = ExcalcServer::new().serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
