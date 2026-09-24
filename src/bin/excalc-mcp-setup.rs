//! Best-effort post-install helper that registers `excalc-mcp` with any
//! MCP-aware CLI tools (GitHub Copilot CLI, Claude Code CLI) found on the
//! machine. Invoked by the Windows MSI installer as an optional, deferred,
//! impersonated custom action, so failures here must never surface to the
//! user or block installation.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
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

/// Resolve a CLI to a runnable path, trying `where`/`which` on PATH first,
/// then falling back to well-known absolute install locations. Both
/// `copilot` and `claude` are commonly installed by npm or a native/winget
/// installer that may not have updated PATH yet at MSI-install time, so the
/// fallback catches the most common real-world Windows layouts.
fn resolve_cli(cli: &str) -> Option<PathBuf> {
    let probe = if cfg!(windows) { "where" } else { "which" };
    if let Ok(out) = Command::new(probe).arg(cli).output() {
        if out.status.success() {
            if let Some(first) = String::from_utf8_lossy(&out.stdout)
                .lines()
                .next()
                .map(str::trim)
            {
                if !first.is_empty() {
                    return Some(PathBuf::from(first));
                }
            }
        }
    }

    known_candidate_paths(cli)
        .into_iter()
        .find(|path| path.is_file())
}

/// Well-known absolute paths where `copilot`/`claude` end up on Windows,
/// covering the npm-global, native-installer, and winget layouts.
fn known_candidate_paths(cli: &str) -> Vec<PathBuf> {
    if !cfg!(windows) {
        return Vec::new();
    }

    let mut candidates = Vec::new();

    // npm global installs (both tools): %APPDATA%\npm\<cli>.cmd
    if let Ok(appdata) = env::var("APPDATA") {
        candidates.push(Path::new(&appdata).join("npm").join(format!("{cli}.cmd")));
    }

    if let Ok(userprofile) = env::var("USERPROFILE") {
        // Claude Code's native installer: %USERPROFILE%\.local\bin\claude.exe
        if cli == "claude" {
            candidates.push(
                Path::new(&userprofile)
                    .join(".local")
                    .join("bin")
                    .join("claude.exe"),
            );
        }
    }

    // GitHub Copilot CLI's winget package layout:
    // %LOCALAPPDATA%\Microsoft\WinGet\Packages\GitHub.Copilot_*\copilot.exe
    if cli == "copilot" {
        if let Ok(localappdata) = env::var("LOCALAPPDATA") {
            let packages_dir = Path::new(&localappdata)
                .join("Microsoft")
                .join("WinGet")
                .join("Packages");
            if let Ok(entries) = fs::read_dir(&packages_dir) {
                for entry in entries.flatten() {
                    let name = entry.file_name();
                    let name = name.to_string_lossy();
                    if name.starts_with("GitHub.Copilot_") {
                        candidates.push(entry.path().join("copilot.exe"));
                    }
                }
            }
        }
    }

    candidates
}

fn register(cli: &str, args: &[&str]) {
    let Some(resolved) = resolve_cli(cli) else {
        return;
    };
    // Ignore the outcome: the server may already be registered, the CLI
    // version may not support `mcp add`, etc. None of that should be fatal.
    let _ = Command::new(resolved).args(args).output();
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
