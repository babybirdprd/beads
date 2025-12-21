pub mod models;
pub mod store;
pub mod util;
pub mod git;
pub mod merge;
pub mod sync;
pub mod fs;

pub use models::*;
pub use store::Store;

#[cfg(not(target_arch = "wasm32"))]
pub use store::SqliteStore;

pub use git::GitOps;
#[cfg(not(target_arch = "wasm32"))]
pub use git::StdGit;

pub use fs::FileSystem;
#[cfg(not(target_arch = "wasm32"))]
pub use fs::StdFileSystem;
