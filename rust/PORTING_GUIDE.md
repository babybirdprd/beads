# Rust Porting Guide

## Current Status (Fully Implemented CLI Commands)
We have achieved feature parity for the core CLI workflow and enhanced usability:

- **beads-core**:
    - `Store::import_from_jsonl` implements strict sync.
    - `Store::get_issue`, `Store::update_issue` support reading and modifying issues.
    - `Store::list_issues` supports filtering by status, assignee, priority, type, labels, and sorting.
    - `merge` module has unit tests covering conflicts and tombstones.

- **beads-cli** (binary `bd`):
    - **Interactive Editing**: `bd edit <id>` opens issues in `$EDITOR` with YAML frontmatter for metadata. `bd create` defaults to interactive mode if description is missing.
    - **Path Handling**: Robust recursive search for `.beads/beads.db` allows running `bd` from any subdirectory.
    - **Testing**: Integration tests in `tests/cli_tests.rs` cover the full lifecycle (`onboard`, `create`, `list`, `edit`, `close`) using `assert_cmd`.
    - **Management**: `create`, `show`, `update`, `close`.
    - **Workflow**: `list` (with advanced filters & tables), `ready` (personal backlog), `onboard` (wizard).
    - **Sync**: `sync`, `export`, `import`, `merge`.
    - **Config**: `config set/get/list`.
    - **UX**: `bd list` uses `comfy-table` and `colored` for pretty output.

## Continuation Prompt / Next Steps

You are continuing the port of the Beads issue tracker to Rust. The core logic, CLI structure, interactive editing, and advanced listing/filtering are complete. Your goal is to finish the Git integration and polish error handling.

### 1. Robust Git Integration & Sync
Currently, `bd sync` manually calls `Store::import_from_jsonl` / `export_to_jsonl`.
- **Task**: Automate Git operations within `bd sync`.
    - Auto-commit changes to `.beads/issues.jsonl` after export?
    - Handle git merge conflicts if `issues.jsonl` is conflicted? (The `bd merge` command exists for 3-way merge, but needs to be hooked up to git merge driver or handled manually).
    - Consider implementing a `git-merge-beads` driver using the `merge` module logic.

### 2. Error Handling & Logging
- **Task**: Ensure all user-facing errors are helpful (avoid raw `anyhow` stack traces for common errors like "Issue not found").
- **Task**: verify `RUST_LOG` usage for debugging.

## Architecture Notes
- **No Daemon**: We are intentionally dropping the Daemon/RPC architecture. Use SQLite file locking for concurrency safety.
- **WASM Goal**: Keep `beads-core` pure Rust where possible. Abstract IO and Git operations to allow future WASM compilation.

## Helpful Commands
- Build: `cd rust && cargo build`
- Run: `cd rust && cargo run -p beads-cli -- <args>`
- Test: `cd rust && cargo test`
