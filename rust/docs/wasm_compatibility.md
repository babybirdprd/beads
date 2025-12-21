# WASM Compatibility

The Rust port is designed with future WebAssembly (WASM) support in mind. This will allow Beads to run in browser-based environments (like VS Code extensions or web UIs) without requiring a native binary.

## Current Status

*   **FileSystem Abstraction**: The `FileSystem` trait in `beads-core` abstracts all file I/O operations (read, write, list, existence check).
*   **GitOps Abstraction**: The `GitOps` trait abstracts git commands.
*   **Store Abstraction**: The `Store` trait in `beads-core` abstracts the persistence layer, allowing different implementations (e.g. SQLite, In-Memory, IndexedDB).
*   **Core Logic**: The core logic is pure Rust and does not depend on OS-specific features, making it portable.
*   **Conditional Compilation**: The native implementations `StdFileSystem`, `StdGit`, and `SqliteStore` are guarded by `#[cfg(not(target_arch = "wasm32"))]`, allowing `beads-core` to compile on WASM targets without linking errors.

## Next Steps

To achieve full WASM support (runtime), the following steps are needed:

1.  **WASM Implementations**:
    *   **FileSystem**: Implement a `FileSystem` that operates on an in-memory or browser-provided file system (e.g., VS Code FS API).
    *   **GitOps**: Implement the `GitOps` trait for WASM. This will likely involve interacting with a JavaScript git provider (e.g. isomorphic-git) or a pure-Rust library compatible with WASM.
    *   **Store**: Implement a WASM-compatible `Store`. This could be an in-memory store for ephemeral sessions, or an adapter for `sqlite-wasm` / IndexedDB for persistence.

2.  **JS/WASM Bindings**:
    *   Expose the `beads-core` API to JavaScript via `wasm-bindgen`.
    *   Create a demo or test harness running in a browser environment to verify end-to-end functionality.
