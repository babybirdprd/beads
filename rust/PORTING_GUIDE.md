# Rust Porting Guide

## Current Status (PoC)
We have established the foundational structure for the Rust port of `beads`.
- **Workspace**: `rust/` contains the cargo workspace.
- **beads-core**: Library crate containing domain models (`Issue`), storage logic (`Store` wrapper around `rusqlite`), git integration, and sync logic.
- **beads-cli**: Binary crate using `clap` for CLI parsing. Implements `list`, `create`, and `sync` commands.
- **Interoperability**: The Rust CLI reads and writes to the same SQLite database (`.beads/beads.db`) as the Go implementation. It writes to the `dirty_issues` table to ensure the Go `bd export` command picks up changes.

---

## Progress Assessment
**Overall Completion: ~75%**

| Component | Status | Notes |
| :--- | :--- | :--- |
| **Core Models** | 🟢 Complete | `Issue` struct updated with `Dependency`, `Comment` types. `relates_to` is `Vec<String>`. |
| **Storage** | 🟢 Complete | Read/write works. `export_to_jsonl` implemented. |
| **ID Generation** | 🟢 Complete | Ported Base36 logic and hash generation (prefix, length, nonce) from Go. |
| **CLI** | 🟢 Complete | `create`, `list`, `sync` commands implemented. Binary name is `bd`. |
| **Git Integration** | 🟢 Complete | `git` module implemented in `beads-core` (init, add, commit, pull --rebase, push, status, show). |
| **Merge Logic** | 🟢 Complete | 3-way merge algorithm ported including tombstone handling. |
| **Sync Logic** | 🟢 Complete | `bd sync` command implemented with conflict resolution. |
| **Compatibility** | 🟢 Verified | Cross-language test suite `scripts/verify_compat.sh` passes. |
| **UX/Error Handling** | 🟢 Improved | Added `anyhow::Context` and cleaned up CLI output. |

---

## Next Steps for the Next Agent

Your goal is to prepare the codebase for WASM compilation and expand feature parity.

### 1. WASM Preparation
* **Task**: Audit `beads-core` for non-WASM compatible IO (mostly `std::fs` and `std::process::Command` in `git.rs`).
* **Strategy**: Consider defining a `GitProvider` trait to abstract git operations, allowing a JS/WASM implementation later.
* **Refactor**: Introduce traits for file system access to abstract away `std::fs`.

### 2. Feature Parity
* **Task**: Implement remaining commands or flags (e.g., `bd config`, `bd stats`).
* **Task**: Enhance `bd sync` to support more flags present in Go version (e.g., `--squash`, `--dry-run`).

### 3. CI Integration
* **Task**: Add `scripts/verify_compat.sh` to the repository's CI pipeline (e.g., GitHub Actions) to ensure ongoing compatibility.

---

## Architecture Notes
* **No Daemon**: We are intentionally dropping the Daemon/RPC architecture. Use SQLite file locking for concurrency safety.
* **WASM Goal**: Keep `beads-core` pure Rust where possible. Abstract IO and Git operations to allow future WASM compilation.

## Helpful Commands
* **Build**: `cd rust && cargo build`
* **Run**: `cd rust && cargo run -p beads-cli -- <args>`
* **Test**: `cd rust && cargo test`
