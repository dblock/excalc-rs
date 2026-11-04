# ExCalc Privacy Policy

Effective date: November 4, 2026.

**In short:** ExCalc evaluates expressions on your device. It does not send your calculations to Vestris or the project maintainers, and it has no telemetry, analytics, advertising, or account requirement. Interactive command history is saved locally by default. AI clients that call ExCalc have their own data handling and privacy policies.

## Scope

This policy covers the ExCalc command-line tools (`excalc` and `calc`), Rust library, MCP server (`excalc-mcp`), and installer helper distributed by this project. It adapts the data-minimization approach of the [Vestris privacy policy](https://www.vestris.com/policies/privacy.txt) to ExCalc. It does not cover the Vestris website, unrelated services, or third-party applications.

## Calculations and local history

Expressions are evaluated locally. ExCalc does not upload expressions, results, variables, or function definitions to the maintainers, collect usage statistics, or send crash reports.

The interactive REPL saves entered commands, which may contain sensitive information, in a local history file. The default is `.excalc_history` in your home directory (normally `~/.excalc_history` on macOS and Linux, or `%USERPROFILE%\.excalc_history` on Windows). You can select another location with `--history-file <path>` or `EXCALC_HISTORY_FILE`.

Run `excalc --no-history` or `calc --no-history` to disable loading and saving persistent history for that session. This does not delete history from previous sessions. To remove it, close running ExCalc sessions and delete the history file, including any copies you created at custom paths. History is not encrypted by ExCalc; access and retention depend on your device permissions, backups, and synchronization settings.

One-shot CLI calculations and MCP requests do not use the REPL history file. Your shell, terminal, calling application, or operating system may independently record commands, output, or diagnostics.

## MCP and AI clients

The MCP server receives expressions and returns results or errors to the calling client over standard input and standard output. It does not contact an external calculation service or send data to the maintainers.

Clients such as GitHub Copilot or Claude may log tool calls or send expressions, results, and other conversation content to their providers. Their settings and privacy policies govern that processing. ExCalc cannot control or delete data retained by those clients or providers.

The Windows installer offers an optional MCP registration feature, enabled by default, that invokes installed Copilot or Claude command-line tools to register the local ExCalc server. You can deselect that feature during installation. Those tools manage their own configuration and behavior.

## Downloads and support

GitHub, crates.io, Microsoft WinGet, Homebrew, and other distribution services handle downloads under their own privacy policies. This policy does not promise that those services, your package manager, or your network provider collect no data.

If you email the maintainers or submit an issue, pull request, or other feedback, we receive the contact information and content you choose to share. We use it to respond and maintain the project, not to sell or rent personal information or use it for advertising. GitHub issues, pull requests, and comments are generally public: do not include private calculations, credentials, or other sensitive information. The relevant service controls its own records and retention.

## Questions and changes

For privacy questions or requests concerning information you have sent to the maintainers, email [dblock@vestris.com](mailto:dblock@vestris.com). We do not have access to your local history or records held by third-party clients.

Policy updates will be published in this file with a revised effective date. Previous versions remain available in the repository history.
