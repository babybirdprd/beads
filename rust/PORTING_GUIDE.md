# Rust Porting Guide

## Current Status (Fully Implemented CLI Commands)
We have achieved feature parity for the core CLI workflow:
- **beads-core**:
    - `Store::import_from_jsonl` implements strict sync.
    - `Store::get_issue`, `Store::update_issue` support reading and modifying issues.
    - `Store::list_issues` supports filtering by status, assignee, priority, and type.
    - `merge` module has unit tests covering conflicts and tombstones.
- **beads-cli** (binary `bd`):
    - **Management**: `create`, `show`, `update`, `close`.
    - **Workflow**: `list` (with filters), `ready` (personal backlog), `onboard` (wizard).
    - **Sync**: `sync`, `export`, `import`, `merge`.
    - **Config**: `config set/get/list`.

## Continuation Prompt / Next Steps

Your goal is to refine the user experience and ensure robustness through comprehensive testing and interactive features.

### 1. Interactive Editing & Usability
The current `bd create` and `bd update` rely on command-line flags.
- **Task**: Implement `bd edit <id>` to open the issue description (and potentially other fields via frontmatter) in the user's `$EDITOR`.
- **Task**: Enhance `bd create` to launch the editor if the description flag is omitted.
- **Logic**:
    - Detect `$EDITOR` or fallback to `vi/nano`.
    - Create a temp file with the current description.
    - Wait for editor process to exit.
    - Read content and update the issue.

### 2. Comprehensive CLI Testing
We have unit tests for `beads-core`, but `beads-cli` lacks integration tests.
- **Task**: Add integration tests using `assert_cmd` or a shell script harness to verify CLI behavior.
    - Test `bd onboard` in a temp directory.
    - Test the full lifecycle: `create` -> `show` -> `update` -> `close`.
    - Verify `sync` creates valid JSONL.

### 3. Cross-Platform & Path Handling
- **Task**: Verify `bd onboard` and path handling on non-Unix systems (if relevant) or ensure robust handling of `.beads` directory paths in nested subdirectories.
- **Check**: Ensure `find_db_path` correctly locates the DB when running from a subdirectory.

## Architecture Notes
- **No Daemon**: We are intentionally dropping the Daemon/RPC architecture. Use SQLite file locking for concurrency safety.
- **WASM Goal**: Keep `beads-core` pure Rust where possible. Abstract IO and Git operations to allow future WASM compilation.

## Helpful Commands
- Build: `cd rust && cargo build`
- Run: `cd rust && cargo run -p beads-cli -- <args>`
- Test: `cd rust && cargo test`
