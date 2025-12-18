# Rust Porting Guide

## Current Status (Sync, Export, Import, Config Implemented)
We have achieved the major milestones: **Sync, Export, Import, Config**.
- **beads-core**:
    - `Store::import_from_jsonl` implements strict sync (clearing child tables to match JSONL source of truth).
    - `Store` schema initialization is handled in `open()`.
    - `merge` module has unit tests covering conflicts and tombstones.
    - `tracing` is used for logging.
- **beads-cli** (binary `bd`):
    - Implements `sync`, `export`, `import`, `list`, `create`, `config`, `merge`.
    - `bd config` manages key-value pairs in SQLite.

## Continuation Prompt / Next Steps

Your goal is to implement the user-facing issue management commands to reach feature parity with the Go tool's daily workflow.

### 1. Implement Issue Management Commands
The CLI lacks commands for viewing and modifying existing issues.
- **Task**: Implement `bd show <id>`, `bd update <id>`, `bd close <id>`.
- **Logic**:
    - `show`: Fetch issue by ID (or short hash) and display details (including comments/deps).
    - `update`: Modify fields (title, desc, status, etc.). Ensure update marks as dirty so it exports.
    - `close`: Shortcut for update status=closed.
- **Reference**: See `cmd/bd/show.go`, `cmd/bd/update.go` in Go.

### 2. Implement `bd ready` and Filtering
`bd list` currently dumps everything.
- **Task**: Implement `bd list --status <status> --assignee <user>` filtering.
- **Task**: Implement `bd ready` (alias for listing open issues assigned to user or unassigned?).

### 3. Implement `bd onboard`
- **Task**: Create a wizard to set up `.beads` repo, git config, and initial user config.

## Architecture Notes
- **No Daemon**: We are intentionally dropping the Daemon/RPC architecture. Use SQLite file locking for concurrency safety.
- **WASM Goal**: Keep `beads-core` pure Rust where possible. Abstract IO and Git operations to allow future WASM compilation.

## Helpful Commands
- Build: `cd rust && cargo build`
- Run: `cd rust && cargo run -p beads-cli -- <args>`
- Test: `cd rust && cargo test`
