use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

// Status is just a string in Go
pub type Status = String;
pub type IssueType = String;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Dependency {
    pub issue_id: String,
    pub depends_on_id: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Comment {
    pub id: i64,
    pub issue_id: String,
    pub author: String,
    pub text: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
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

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relates_to: Vec<String>,

    #[serde(default)]
    pub duplicate_of: String,
    #[serde(default)]
    pub superseded_by: String,

    // Tombstone fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub deleted_by: String,
    #[serde(default)]
    pub delete_reason: String,
    #[serde(default)]
    pub original_type: String,

    // Extra fields for export
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labels: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<Dependency>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub comments: Vec<Comment>,
}
