//! Integration tests that exercise the `excalc-mcp` binary end-to-end over
//! stdio using raw JSON-RPC, covering `src/bin/excalc-mcp.rs`. Only built
//! with `--features mcp` (see the `[[test]]` entry in `Cargo.toml`).

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Command, Stdio};

struct McpProcess {
    child: Child,
    stdin: Option<ChildStdin>,
    stdout: BufReader<std::process::ChildStdout>,
}

impl McpProcess {
    fn spawn() -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_excalc-mcp"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let stdin = child.stdin.take().unwrap();
        let stdout = BufReader::new(child.stdout.take().unwrap());
        Self {
            child,
            stdin: Some(stdin),
            stdout,
        }
    }

    /// Sends a JSON-RPC request/notification and, if it has an `id` (i.e.
    /// isn't a notification), reads and parses the matching response line.
    fn send(&mut self, request: serde_json::Value) -> Option<serde_json::Value> {
        let mut line = request.to_string();
        line.push('\n');
        let stdin = self.stdin.as_mut().unwrap();
        stdin.write_all(line.as_bytes()).unwrap();
        stdin.flush().unwrap();

        request.get("id")?;
        let mut response = String::new();
        self.stdout.read_line(&mut response).unwrap();
        Some(serde_json::from_str(&response).unwrap())
    }
}

impl Drop for McpProcess {
    fn drop(&mut self) {
        // Closing stdin makes the server's stdio transport see EOF and shut
        // down normally (rather than being killed), so the process exits
        // cleanly and, under `cargo llvm-cov`, flushes its coverage profile.
        self.stdin.take();
        let _ = self.child.wait();
    }
}

#[test]
fn initialize_lists_and_calls_the_evaluate_tool() {
    let mut mcp = McpProcess::spawn();

    let init = mcp
        .send(serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-06-18",
                "capabilities": {},
                "clientInfo": {"name": "test", "version": "1.0"}
            }
        }))
        .unwrap();
    assert_eq!(
        init["result"]["serverInfo"]["name"],
        serde_json::json!("excalc-mcp")
    );

    mcp.send(serde_json::json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    }));

    let list = mcp
        .send(serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list",
            "params": {}
        }))
        .unwrap();
    let tools = list["result"]["tools"].as_array().unwrap();
    assert!(tools.iter().any(|t| t["name"] == "evaluate"));

    let ok = mcp
        .send(serde_json::json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {"name": "evaluate", "arguments": {"expression": "2 + 2 * 3"}}
        }))
        .unwrap();
    assert_eq!(ok["result"]["isError"], serde_json::json!(false));
    assert_eq!(ok["result"]["content"][0]["text"], serde_json::json!("8"));

    let err = mcp
        .send(serde_json::json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "tools/call",
            "params": {"name": "evaluate", "arguments": {"expression": "1/0"}}
        }))
        .unwrap();
    assert_eq!(err["result"]["isError"], serde_json::json!(true));
    assert_eq!(
        err["result"]["content"][0]["text"],
        serde_json::json!("division by zero")
    );
}

/// Keeps the complex example in the README's MCP Server section in sync
/// with the real evaluator, the same way `tests/readme_examples.rs` does
/// for the CLI `### Examples` section: parses the `> what's ... ?` line and
/// the result line right after it straight out of README.md rather than
/// hardcoding them here, so a doc edit that goes out of sync fails this test.
#[test]
fn readme_mcp_example_matches_evaluator() {
    let readme_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("README.md");
    let readme = std::fs::read_to_string(&readme_path).expect("README.md should be readable");

    let mut lines = readme.lines();
    let expression = lines
        .by_ref()
        .find_map(|line| {
            line.strip_prefix("> what's ")
                .and_then(|s| s.strip_suffix(" ?"))
        })
        .expect("MCP example (\"> what's ... ?\") not found in README.md");
    let expected: f64 = lines
        .next()
        .expect("MCP example result line not found in README.md")
        .trim()
        .parse()
        .expect("MCP example result line isn't a number");

    assert_eq!(excalc::evaluate(expression).unwrap(), expected);

    let mut mcp = McpProcess::spawn();
    mcp.send(serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-06-18",
            "capabilities": {},
            "clientInfo": {"name": "test", "version": "1.0"}
        }
    }));
    mcp.send(serde_json::json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    }));

    let result = mcp
        .send(serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {"name": "evaluate", "arguments": {"expression": expression}}
        }))
        .unwrap();
    assert_eq!(
        result["result"]["content"][0]["text"],
        serde_json::json!(expected.to_string())
    );
}
