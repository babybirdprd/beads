# Rust Porting Guide

## Current Status (PoC)
We have established the foundational structure for the Rust port of `beads`.
- **Workspace**: `rust/` contains the cargo workspace.
- **beads-core**: Library crate containing domain models (`Issue`) and storage logic (`Store` wrapper around `rusqlite`).
- **beads-cli**: Binary crate using `clap` for CLI parsing. Implements `list` and `create` commands.
- **Interoperability**: The Rust CLI reads and writes to the same SQLite database (`.beads/beads.db`) as the Go implementation. It writes to `dirty_issues` table to ensure the Go `bd export` command picks up changes.

## Next Steps for the Next Agent

Your goal is to continue the port towards full feature parity, focusing on the "Sync" and "Git" capabilities.

### 1. Implement JSONL Export
Currently, `beads-cli create` writes to SQLite but does not update `.beads/issues.jsonl`.
- **Task**: Implement `Store::export_to_jsonl()` in `beads-core`.
- **Logic**: Query all issues (or dirty ones), serialize to JSONL, and write to `.beads/issues.jsonl`.
- **Reference**: See `internal/export/export.go` in the Go codebase.

### 2. Implement Git Integration
The Go version uses `exec.Command("git")`.
- **Task**: Create a `Git` struct in `beads-core` that handles git operations.
- **Strategy**: Start by wrapping `std::process::Command("git")` to match existing behavior. Later, consider `git2` or `gitoxide` for WASM compatibility.
- **Operations needed**: `init`, `add`, `commit`, `push`, `pull`.

### 3. Implement Merge Logic
This is the "brain" of the conflict resolution.
- **Task**: Port `internal/merge/merge.go` to Rust.
- **Logic**: It's a 3-way merge of JSON lists.
- **Destination**: `beads-core::merge`.

### 4. Implement `sync` Command
The `sync` command is the heart of beads.
- **Task**: Implement `bd sync`.
- **Flow**:
    1.  Export DB to JSONL.
    2.  `git add .beads/issues.jsonl`.
    3.  `git commit`.
    4.  `git pull --rebase`. (If conflict, use merge logic).
    5.  `git push`.

## Architecture Notes
- **No Daemon**: We are intentionally dropping the Daemon/RPC architecture. Use SQLite file locking for concurrency safety.
- **WASM Goal**: Keep `beads-core` pure Rust where possible. Abstract IO and Git operations to allow future WASM compilation (e.g. for VSCode web).
- **Testing**: Add unit tests for `beads-core` logic, especially the Merge logic.

## Helpful Commands
- Build: `cd rust && cargo build`
- Run: `cd rust && cargo run -p beads-cli -- <args>`
- Test: `cd rust && cargo test`
