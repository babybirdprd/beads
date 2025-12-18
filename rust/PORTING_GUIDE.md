# Rust Porting Guide

## Current Status (Sync & Export Implemented)
We have achieved the first major milestone: **Sync & Export**.
- **beads-core**:
    - `Store::export_to_jsonl` implements full-fidelity export (including labels, dependencies, comments, tombstones).
    - `merge` module ports the Go 3-way merge logic exactly.
    - `git` module wraps git commands.
    - `sync` module orchestrates the sync workflow (Export -> Commit -> Pull Rebase -> Merge -> Push).
    - `Issue` struct matches Go's JSON schema (Base36 IDs, `Vec<String>` for `relates_to`).
- **beads-cli** (binary `bd`):
    - Implements `sync`, `export`, `merge` (merge driver), `list`, `create`.
    - Renamed to `bd` in Cargo.toml.

## Continuation Prompt / Next Steps

Your goal is to complete the "Import" side of the equation and robustify the CLI.

### 1. Implement JSONL Import
Currently, `bd sync` pulls changes but does **not** import them into the local SQLite database.
- **Task**: Implement `Store::import_from_jsonl(path)` in `beads-core`.
- **Logic**: Read JSONL, upsert issues into SQLite (`INSERT OR REPLACE` or smart update). Handle deletions (tombstones).
- **Reference**: See `cmd/bd/import.go` and `internal/importer` in Go.
- **Integration**: Update `beads-core::sync::run_sync` to call `import_from_jsonl` after `git.pull_rebase` (and after merge resolution).

### 2. Add Unit Tests for Merge Logic
The `merge` logic is ported but lacks granular unit tests.
- **Task**: Add tests in `rust/beads-core/src/merge.rs` covering:
    - Conflict resolution (Closed vs Open, Priority).
    - Tombstone handling (expiry, resurrection).
    - 3-way merge scenarios.

### 3. Implement Config Command
The CLI currently has hardcoded defaults or simple paths.
- **Task**: Implement `bd config` to read/write `.beads/config` (or SQLite `config` table).
- **Goal**: Support `user.email`, `sync.remote`, etc.

### 4. Improve Error Handling & Logging
- Use `tracing` or `log` crate instead of `println!`.
- Refine error types in `beads-core`.

## Architecture Notes
- **No Daemon**: We are intentionally dropping the Daemon/RPC architecture. Use SQLite file locking for concurrency safety.
- **WASM Goal**: Keep `beads-core` pure Rust where possible. Abstract IO and Git operations to allow future WASM compilation.

## Helpful Commands
- Build: `cd rust && cargo build`
- Run: `cd rust && cargo run -p beads-cli -- <args>`
- Test: `cd rust && cargo test`
