# Rust Porting Guide

## Current Status (PoC)
We have established the foundational structure for the Rust port of `beads`.
- **Workspace**: `rust/` contains the cargo workspace.
- **beads-core**: Library crate containing domain models (`Issue`) and storage logic (`Store` wrapper around `rusqlite`).
- **beads-cli**: Binary crate using `clap` for CLI parsing. Implements `list` and `create` commands.
- **Interoperability**: The Rust CLI reads and writes to the same SQLite database (`.beads/beads.db`) as the Go implementation. It writes to the `dirty_issues` table to ensure the Go `bd export` command picks up changes.

---

## Progress Assessment
**Overall Completion: ~15-20%**

| Component | Status | Notes |
| :--- | :--- | :--- |
| **Core Models** | 🟡 Partial | `Issue` struct exists but missing `Dependency`, `Label`, `Comment` types. `relates_to` is `String` instead of `Vec<String>`. |
| **Storage** | 🟡 Partial | Basic read/write for `issues` table. Missing table joins and JSONL export. |
| **ID Generation** | 🔴 Incorrect | Uses Hex encoding. Must port Base36 logic from `internal/storage/sqlite/ids.go`. |
| **CLI** | 🟡 Partial | `list` and `create` work. Binary name is `beads-cli` (needs rename to `bd`). |
| **Git Integration** | ⚪ Missing | No wrapper for git operations yet. |
| **Merge Logic** | ⚪ Missing | 3-way merge algorithm not ported. |
| **Sync Logic** | ⚪ Missing | `bd sync` command not implemented. |

---

## Next Steps for the Next Agent

Your goal is to continue the port towards full feature parity, focusing on data correctness and the **Export** capability.

### 1. Fix ID Generation & Binary Name
* **Binary**: Rename `beads-cli` to `bd` in `rust/beads-cli/Cargo.toml`:
    ```toml
    [[bin]]
    name = "bd"
    path = "src/main.rs"
    ```
* **ID Generation**: The Go app uses **Base36** encoding (`0-9`, `a-z`), **NOT** Hex.
    * Port `encodeBase36` from `internal/storage/sqlite/ids.go` to `rust/beads-core/src/util.rs`.
    * Ensure the hash content matches the Go format: `fmt.Sprintf("%s|%s|%s|%d|%d", ...)` (Check `ids.go` for exact format).

### 2. Complete Domain Model
* **Task**: Update `rust/beads-core/src/models.rs`.
* **Changes**:
    * Add structs: `Dependency`, `Label`, `Comment`.
    * **Update Issue**:
        * `relates_to`: Change to `Vec<String>` (handle parsing from DB string).
        * Add fields: `dependencies: Vec<Dependency>`, `labels: Vec<String>`, `comments: Vec<Comment>`.
* **Note**: This requires updating `Store::list_issues` to perform JOINs or separate queries to populate these fields.

### 3. Implement JSONL Export
* **Task**: Implement `Store::export_to_jsonl()` in `beads-core`.
* **Logic**:
    * Query issues (all or dirty).
    * Serialize to JSONL (one JSON object per line).
    * Write to `.beads/issues.jsonl`.
    * **Reference**: See `internal/export/executor.go` and `cmd/bd/export.go`.

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
