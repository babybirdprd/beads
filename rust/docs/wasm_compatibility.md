# WASM Compatibility

The Rust port is designed with future WebAssembly (WASM) support in mind. This will allow Beads to run in browser-based environments (like VS Code extensions or web UIs) without requiring a native binary.

## Current Status

*   **FileSystem Abstraction**: The `FileSystem` trait in `beads-core` abstracts all file I/O operations.
    *   **Native**: `StdFileSystem` uses `std::fs`.
    *   **WASM**: `WasmFileSystem` delegates to JavaScript functions via `wasm-bindgen`.
*   **GitOps Abstraction**: The `GitOps` trait abstracts git commands.
    *   **Native**: `StdGit` uses `std::process::Command` to call the `git` binary.
    *   **WASM**: `WasmGit` delegates to JavaScript functions via `wasm-bindgen`.
*   **Store Abstraction**: The `Store` trait in `beads-core` abstracts the persistence layer.
    *   **Native**: `SqliteStore` uses `rusqlite` (SQLite).
    *   **WASM/All**: `MemoryStore` provides an in-memory implementation suitable for ephemeral sessions or testing.
*   **Core Logic**: The core logic is pure Rust and does not depend on OS-specific features.
*   **Compilation**: `beads-core` compiles for `wasm32-unknown-unknown`. Dependencies like `rusqlite` are gated behind `#[cfg(not(target_arch = "wasm32"))]`.

## JavaScript Bindings

The `beads-core` library exposes the following JS bindings when compiled for WASM:

*   **Modules**:
    *   `/js/beads_fs.js`: Expected to export filesystem functions (`fs_read_to_string`, `fs_write`, etc.).
    *   `/js/beads_git.js`: Expected to export git functions (`git_init`, `git_commit`, etc.).
*   **Classes**:
    *   `WasmFileSystem`: Rust wrapper around the JS filesystem module.
    *   `WasmGit`: Rust wrapper around the JS git module.

## Usage in WASM

To use Beads in a WASM environment:
1.  Compile `beads-core` with `wasm-pack`.
2.  Provide implementations for the functions in `beads_fs.js` and `beads_git.js` in the host environment.
3.  Instantiate `MemoryStore` or implement a custom `Store` (e.g., on top of IndexedDB).

## Next Steps

1.  **JS Implementation**: Create a reference implementation of the JS bindings (e.g., using `isomorphic-git` and a browser-fs adapter).
2.  **End-to-End Test**: Create a browser-based test harness to verify the full flow.
