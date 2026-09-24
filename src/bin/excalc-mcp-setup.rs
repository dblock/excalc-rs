//! Best-effort post-install helper that registers `excalc-mcp` with any
//! MCP-aware CLI tools (GitHub Copilot CLI, Claude Code CLI) found on the
//! machine. Invoked by the Windows MSI installer as an optional, deferred,
//! impersonated custom action, so failures here must never surface to the
//! user or block installation.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn mcp_binary_name() -> &'static str {
    if cfg!(windows) {
        "excalc-mcp.exe"
    } else {
        "excalc-mcp"
    }
}

fn mcp_binary_path() -> Option<PathBuf> {
    let exe = env::current_exe().ok()?;
    let dir = exe.parent()?;
    let candidate = dir.join(mcp_binary_name());
    candidate.exists().then_some(candidate)
}

fn probe_command() -> &'static str {
    if cfg!(windows) {
        "where"
    } else {
        "which"
    }
}

/// Extract the first non-empty line from `where`/`which` output as a path.
fn parse_probe_output(stdout: &str) -> Option<PathBuf> {
    stdout
        .lines()
        .next()
        .map(str::trim)
        .filter(|first| !first.is_empty())
        .map(PathBuf::from)
}

/// Resolve a CLI to a runnable path, trying `where`/`which` on PATH first,
/// then falling back to well-known absolute install locations. Both
/// `copilot` and `claude` are commonly installed by npm or a native/winget
/// installer that may not have updated PATH yet at MSI-install time, so the
/// fallback catches the most common real-world Windows layouts.
fn resolve_cli(cli: &str) -> Option<PathBuf> {
    if let Ok(out) = Command::new(probe_command()).arg(cli).output() {
        if out.status.success() {
            if let Some(path) = parse_probe_output(&String::from_utf8_lossy(&out.stdout)) {
                return Some(path);
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

    windows_candidate_paths(
        cli,
        env::var("APPDATA").ok(),
        env::var("USERPROFILE").ok(),
        env::var("LOCALAPPDATA").ok(),
        winget_copilot_dirs,
    )
}

/// Lists the subdirectories of `packages_dir` matching GitHub Copilot CLI's
/// winget package naming convention (`GitHub.Copilot_*`).
fn winget_copilot_dirs(packages_dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(packages_dir) else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with("GitHub.Copilot_")
        })
        .map(|entry| entry.path())
        .collect()
}

/// Pure, platform-independent implementation of [`known_candidate_paths`],
/// taking environment values and the winget-directory lookup as parameters
/// so it can be exercised deterministically by unit tests without touching
/// real process environment variables or the filesystem.
fn windows_candidate_paths(
    cli: &str,
    appdata: Option<String>,
    userprofile: Option<String>,
    localappdata: Option<String>,
    list_winget_copilot_dirs: impl Fn(&Path) -> Vec<PathBuf>,
) -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    // npm global installs (both tools): %APPDATA%\npm\<cli>.cmd
    if let Some(appdata) = appdata {
        candidates.push(Path::new(&appdata).join("npm").join(format!("{cli}.cmd")));
    }

    // Claude Code's native installer: %USERPROFILE%\.local\bin\claude.exe
    if cli == "claude" {
        if let Some(userprofile) = userprofile {
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
        if let Some(localappdata) = localappdata {
            let packages_dir = Path::new(&localappdata)
                .join("Microsoft")
                .join("WinGet")
                .join("Packages");
            for dir in list_winget_copilot_dirs(&packages_dir) {
                candidates.push(dir.join("copilot.exe"));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mcp_binary_name_matches_platform() {
        let expected = if cfg!(windows) {
            "excalc-mcp.exe"
        } else {
            "excalc-mcp"
        };
        assert_eq!(mcp_binary_name(), expected);
    }

    #[test]
    fn mcp_binary_path_is_none_when_sibling_missing() {
        // In test binaries, current_exe() lives in target/.../deps, where no
        // excalc-mcp binary sits alongside, so this should reliably be None.
        assert!(mcp_binary_path().is_none());
    }

    #[test]
    fn probe_command_matches_platform() {
        let expected = if cfg!(windows) { "where" } else { "which" };
        assert_eq!(probe_command(), expected);
    }

    #[test]
    fn parse_probe_output_returns_first_nonempty_line() {
        assert_eq!(
            parse_probe_output("/usr/bin/copilot\n/usr/local/bin/copilot\n"),
            Some(PathBuf::from("/usr/bin/copilot"))
        );
    }

    #[test]
    fn parse_probe_output_trims_whitespace() {
        assert_eq!(
            parse_probe_output("  /usr/bin/claude  \n"),
            Some(PathBuf::from("/usr/bin/claude"))
        );
    }

    #[test]
    fn parse_probe_output_none_when_empty() {
        assert_eq!(parse_probe_output(""), None);
        assert_eq!(parse_probe_output("\n\n"), None);
    }

    #[test]
    fn windows_candidate_paths_includes_npm_global_cmd() {
        let paths = windows_candidate_paths(
            "copilot",
            Some("C:\\Users\\dblock\\AppData\\Roaming".to_string()),
            None,
            None,
            |_| Vec::new(),
        );
        assert!(paths.contains(&PathBuf::from(
            "C:\\Users\\dblock\\AppData\\Roaming/npm/copilot.cmd"
        )));
    }

    #[test]
    fn windows_candidate_paths_includes_claude_native_installer_path() {
        let paths = windows_candidate_paths(
            "claude",
            None,
            Some("C:\\Users\\dblock".to_string()),
            None,
            |_| Vec::new(),
        );
        assert!(paths.contains(&PathBuf::from("C:\\Users\\dblock/.local/bin/claude.exe")));
    }

    #[test]
    fn windows_candidate_paths_skips_claude_native_installer_for_copilot() {
        let paths = windows_candidate_paths(
            "copilot",
            None,
            Some("C:\\Users\\dblock".to_string()),
            None,
            |_| Vec::new(),
        );
        assert!(!paths.iter().any(|p| p.to_string_lossy().contains(".local")));
    }

    #[test]
    fn windows_candidate_paths_includes_winget_copilot_packages() {
        let paths = windows_candidate_paths(
            "copilot",
            None,
            None,
            Some("C:\\Users\\dblock\\AppData\\Local".to_string()),
            |_dir| vec![PathBuf::from("C:\\...\\GitHub.Copilot_8wekyb3d8bbwe")],
        );
        assert!(paths.contains(&PathBuf::from(
            "C:\\...\\GitHub.Copilot_8wekyb3d8bbwe/copilot.exe"
        )));
    }

    #[test]
    fn windows_candidate_paths_skips_winget_lookup_for_claude() {
        let lookup_called = std::cell::Cell::new(false);
        let paths = windows_candidate_paths(
            "claude",
            None,
            None,
            Some("C:\\Users\\dblock\\AppData\\Local".to_string()),
            |_dir| {
                lookup_called.set(true);
                Vec::new()
            },
        );
        assert!(!lookup_called.get());
        assert!(paths.is_empty());
    }

    #[test]
    fn windows_candidate_paths_empty_without_env_vars() {
        let paths = windows_candidate_paths("copilot", None, None, None, |_| Vec::new());
        assert!(paths.is_empty());
    }

    #[test]
    fn known_candidate_paths_empty_on_non_windows() {
        if !cfg!(windows) {
            assert!(known_candidate_paths("copilot").is_empty());
            assert!(known_candidate_paths("claude").is_empty());
        }
    }

    #[test]
    fn winget_copilot_dirs_filters_by_prefix_and_ignores_missing_dir() {
        assert!(winget_copilot_dirs(Path::new("/does/not/exist")).is_empty());

        let tmp = std::env::temp_dir().join(format!(
            "excalc-mcp-setup-test-{}-{}",
            std::process::id(),
            "winget_copilot_dirs_filters_by_prefix_and_ignores_missing_dir"
        ));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(tmp.join("GitHub.Copilot_abc")).unwrap();
        fs::create_dir_all(tmp.join("Some.Other.Package")).unwrap();

        let dirs = winget_copilot_dirs(&tmp);
        assert_eq!(dirs, vec![tmp.join("GitHub.Copilot_abc")]);

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn resolve_cli_returns_none_for_nonexistent_cli() {
        assert!(resolve_cli("definitely-not-a-real-cli-xyz").is_none());
    }

    #[test]
    fn register_is_a_noop_for_nonexistent_cli() {
        // Should not panic and should simply do nothing.
        register("definitely-not-a-real-cli-xyz", &["mcp", "add", "excalc"]);
    }
}
