//! Best-effort post-install helper that registers `excalc-mcp` with any
//! MCP-aware CLI tools (GitHub Copilot CLI, Claude Code CLI) found on the
//! machine. Invoked by the Windows MSI installer as an optional, deferred,
//! impersonated custom action, so failures here must never surface to the
//! user or block installation.

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn mcp_binary_path() -> Option<PathBuf> {
    let exe = env::current_exe().ok()?;
    let dir = exe.parent()?;
    let name = if cfg!(windows) {
        "excalc-mcp.exe"
    } else {
        "excalc-mcp"
    };
    let candidate = dir.join(name);
    candidate.exists().then_some(candidate)
}

fn command_exists(cli: &str) -> bool {
    let probe = if cfg!(windows) { "where" } else { "which" };
    Command::new(probe)
        .arg(cli)
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false)
}

fn register(cli: &str, args: &[&str]) {
    if !command_exists(cli) {
        return;
    }
    // Ignore the outcome: the server may already be registered, the CLI
    // version may not support `mcp add`, etc. None of that should be fatal.
    let _ = Command::new(cli).args(args).output();
}

fn main() {
    let Some(mcp_path) = mcp_binary_path() else {
        return;
    };
    let mcp_path = mcp_path.to_string_lossy().into_owned();

    register("copilot", &["mcp", "add", "excalc", "--", &mcp_path]);
    register(
        "claude",
        &["mcp", "add", "excalc", "--scope", "user", "--", &mcp_path],
    );
}
