use sha2::{Digest, Sha256};
use chrono::{DateTime, Utc};

pub fn generate_hash_id(title: &str, description: &str, created_at: DateTime<Utc>, workspace_id: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(title.as_bytes());
    hasher.update(description.as_bytes());
    // Note: Go's RFC3339Nano might differ in sub-second precision formatting.
    // Ideally we'd match it exactly, but for a PoC this creates a valid unique ID.
    hasher.update(created_at.to_rfc3339().as_bytes());
    hasher.update(workspace_id.as_bytes());

    let result = hasher.finalize();
    hex::encode(result)
}
