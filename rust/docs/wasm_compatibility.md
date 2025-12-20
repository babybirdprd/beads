# WASM Compatibility

The Rust port is designed with future WebAssembly (WASM) support in mind. This will allow Beads to run in browser-based environments (like VS Code extensions or web UIs) without requiring a native binary.

## Current Status

*   **FileSystem Abstraction**: The `FileSystem` trait in `beads-core` is the key enabler. It abstracts all file I/O operations (read, write, list, existence check).
*   **GitOps Abstraction**: The `GitOps` trait abstracts git commands.
*   **Core Logic**: The core logic is pure Rust and does not depend on OS-specific features, making it portable.

## Next Steps

To achieve full WASM support, the following steps are needed:

1.  **Target Support**: Ensure all dependencies support `wasm32-unknown-unknown`.
2.  **SQLite Adapter**: Replace or configure `rusqlite` to work with a WASM-compatible SQLite implementation (e.g., `sqlite-wasm` or a virtual file system adapter).
3.  **Git Implementation**: Implement the `GitOps` trait for WASM. Since `std::process::Command` is not available, this will likely involve using a pure-Rust git library (like `git2` or `gix`) or interacting with a JavaScript git provider.
4.  **Virtual FileSystem**: Implement a `FileSystem` that operates on an in-memory or browser-provided file system (e.g., VS Code FS API).
