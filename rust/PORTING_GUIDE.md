# Rust Porting Guide

## Current Status (PoC)
We have established the foundational structure for the Rust port of `beads`.
- **Workspace**: `rust/` contains the cargo workspace.
- **beads-core**: Library crate containing domain models (`Issue`) and storage logic (`Store` wrapper around `rusqlite`).
- **beads-cli**: Binary crate using `clap` for CLI parsing. Implements `list` and `create` commands.
- **Interoperability**: The Rust CLI reads and writes to the same SQLite database (`.beads/beads.db`) as the Go implementation. It writes to the `dirty_issues` table to ensure the Go `bd export` command picks up changes.

---

## Progress Assessment
**Overall Completion: ~25-30%**

| Component | Status | Notes |
| :--- | :--- | :--- |
| **Core Models** | 🟢 Complete | `Issue` struct updated with `Dependency`, `Comment` types. `relates_to` is `Vec<String>`. |
| **Storage** | 🟡 Partial | Read/write works. `export_to_jsonl` implemented but needs rigorous testing against Go artifacts. |
| **ID Generation** | 🟢 Complete | Ported Base36 logic and hash generation (prefix, length, nonce) from Go. |
| **CLI** | 🟡 Partial | `create` uses correct ID generation. Binary name is `bd`. |
| **Git Integration** | ⚪ Missing | No wrapper for git operations yet. |
| **Merge Logic** | ⚪ Missing | 3-way merge algorithm not ported. |
| **Sync Logic** | ⚪ Missing | `bd sync` command not implemented. |

---

## Next Steps for the Next Agent

Your goal is to implement Git integration and the core Sync logic.

### 1. Implement Git Integration
* **Task**: Create a `git` module in `beads-core` (or a separate crate if preferred).
* **Strategy**: Wrap `std::process::Command("git")`.
* **Operations**: `init`, `add`, `commit`, `push`, `pull`, `status`, `rev-parse`.
* **Reference**: See `internal/git/git.go`.

### 2. Implement Sync Logic (`bd sync`)
* **Task**: Implement `beads_core::sync::run_sync`.
* **Logic**:
    1.  **Check for changes**: Is the DB dirty? (Check `dirty_issues` table).
    2.  **Export**: If dirty, call `Store::export_to_jsonl`.
    3.  **Git Add/Commit**: Add `.beads/issues.jsonl` and commit with a standard message (e.g., "update issues").
    4.  **Pull/Merge**: Pull remote changes. If conflict, handle via 3-way merge (already partially scaffolded in `merge.rs`, but needs completion).
    5.  **Import**: If new JSONL from remote, call `Store::import_from_jsonl`.

### 3. Implement 3-Way Merge Logic
* **Task**: Port `internal/merge/merge.go` to `rust/beads-core/src/merge.rs`.
* **Logic**:
    *   Compare Base (common ancestor), Left (local), and Right (remote).
    *   Resolve conflicts field-by-field.
    *   Handle tombstones (deletions).
    *   This is critical for `bd sync`.

### 4. Verify Cross-Language Compatibility
* **Task**: Create a test script that:
    1.  Creates an issue with Go `bd`.
    2.  Reads/Updates it with Rust `bd`.
    3.  Exports with Rust `bd`.
    4.  Imports back with Go `bd`.
*   Ensure hashes and content match exactly.

### 4. Implement Git Integration
* **Task**: Create a `git` module in `beads-core`.
* **Strategy**: Wrap `std::process::Command("git")`.
* **Operations**: `init`, `add`, `commit`, `push`, `pull`.

### 5. Implement Merge Logic
* **Task**: Port `internal/merge/merge.go` to Rust.
* **Logic**: Exact port of the 3-way merge algorithm (including Tombstone handling).

---

## Architecture Notes
* **No Daemon**: We are intentionally dropping the Daemon/RPC architecture. Use SQLite file locking for concurrency safety.
* **WASM Goal**: Keep `beads-core` pure Rust where possible. Abstract IO and Git operations to allow future WASM compilation.

## Helpful Commands
* **Build**: `cd rust && cargo build`
* **Run**: `cd rust && cargo run -p beads-cli -- <args>`
* **Test**: `cd rust && cargo test`
