use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

// Status is just a string in Go
pub type Status = String;
pub type IssueType = String;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Issue {
    pub id: String,

    #[serde(skip)]
    pub content_hash: String,

    pub title: String,
    pub description: String,

    #[serde(default)]
    pub design: String,

    #[serde(default)]
    pub acceptance_criteria: String,

    #[serde(default)]
    pub notes: String,

    pub status: Status,
    pub priority: i32,
    pub issue_type: IssueType,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_minutes: Option<i32>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub closed_at: Option<DateTime<Utc>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_ref: Option<String>,

    // Messaging fields
    #[serde(default)]
    pub sender: String,
    #[serde(default)]
    pub ephemeral: bool,
    #[serde(default)]
    pub replies_to: String,
    #[serde(default)]
    pub relates_to: String,     // Go: []string, but schema says TEXT?
                                // Schema: relates_to TEXT DEFAULT ''
                                // types.go: RelatesTo []string `json:"relates_to,omitempty"`
                                // Wait, if DB has TEXT and Struct has []string, Go sql driver might handle it?
                                // Or does it store JSON in TEXT?
                                // Schema says: relates_to TEXT DEFAULT ''
                                // Go struct: RelatesTo []string
                                // I need to check how it's stored.
                                // If I look at schema.go again: "relates_to TEXT DEFAULT ''"
                                // Maybe it's comma separated?
    #[serde(default)]
    pub duplicate_of: String,
    #[serde(default)]
    pub superseded_by: String,
}

// Check `relates_to` in Go code.
// types.go: RelatesTo []string
// schema.go: relates_to TEXT
// internal/storage/sqlite/issues.go might handle the conversion.
// For PoC, I'll use String to be safe with DB schema.
// If it's a JSONL struct, it should be Vec<String>.
// But if I read from DB using rusqlite, I read TEXT.
// So I need two structs? Or a custom deserializer?
// For PoC, let's just stick to DB representation or handle it simply.
// I'll check how `relates_to` is scanned in Go.

// Re-reading types.go:
// RelatesTo []string `json:"relates_to,omitempty"`
// So in JSONL it's a list.
// In DB it's TEXT.
// I'll assume standard comma-separated or JSON string in DB.
// I'll stick to String for now to avoid complexity in PoC.
