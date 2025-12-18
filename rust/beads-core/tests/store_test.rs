use beads_core::{Store, Issue, Dependency, Comment};
use std::fs;
use chrono::Utc;

#[test]
fn test_export_to_jsonl() -> anyhow::Result<()> {
    // Setup temp DB
    let dir = std::env::temp_dir().join("beads_test_export");
    fs::create_dir_all(&dir)?;
    let db_path = dir.join("beads.db");
    if db_path.exists() {
        fs::remove_file(&db_path)?;
    }

    let store = Store::open(&db_path)?;

    // Init Schema
    store.execute_raw(r#"
        CREATE TABLE issues (
            id TEXT PRIMARY KEY, content_hash TEXT, title TEXT, description TEXT,
            design TEXT, acceptance_criteria TEXT, notes TEXT, status TEXT,
            priority INTEGER, issue_type TEXT, assignee TEXT, estimated_minutes INTEGER,
            created_at DATETIME, updated_at DATETIME, closed_at DATETIME,
            external_ref TEXT, sender TEXT, ephemeral INTEGER, replies_to TEXT,
            relates_to TEXT, duplicate_of TEXT, superseded_by TEXT,
            deleted_at DATETIME, deleted_by TEXT, delete_reason TEXT, original_type TEXT
        );
        CREATE TABLE labels (issue_id TEXT, label TEXT);
        CREATE TABLE dependencies (issue_id TEXT, depends_on_id TEXT, type TEXT, created_at DATETIME, created_by TEXT);
        CREATE TABLE comments (id INTEGER PRIMARY KEY, issue_id TEXT, author TEXT, text TEXT, created_at DATETIME);
        CREATE TABLE dirty_issues (issue_id TEXT);
        CREATE TABLE metadata (key TEXT PRIMARY KEY, value TEXT);
    "#)?;

    // Insert test data
    let now = Utc::now();
    let issue = Issue {
        id: "bd-123456".to_string(),
        content_hash: "".to_string(),
        title: "Test Issue".to_string(),
        description: "Desc".to_string(),
        design: "".to_string(),
        acceptance_criteria: "".to_string(),
        notes: "".to_string(),
        status: "open".to_string(),
        priority: 2,
        issue_type: "bug".to_string(),
        assignee: Some("alice".to_string()),
        estimated_minutes: None,
        created_at: now,
        updated_at: now,
        closed_at: None,
        external_ref: None,
        sender: "".to_string(),
        ephemeral: false,
        replies_to: "".to_string(),
        relates_to: vec!["bd-other".to_string()],
        duplicate_of: "".to_string(),
        superseded_by: "".to_string(),
        deleted_at: None,
        deleted_by: "".to_string(),
        delete_reason: "".to_string(),
        original_type: "".to_string(),
        labels: vec!["bug".to_string(), "urgent".to_string()],
        dependencies: vec![Dependency {
            issue_id: "bd-123456".to_string(),
            depends_on_id: "bd-blocker".to_string(),
            type_: "blocks".to_string(),
            created_at: now,
            created_by: "bob".to_string(),
        }],
        comments: vec![Comment {
            id: 0, // Ignored by create_issue
            issue_id: "bd-123456".to_string(),
            author: "charlie".to_string(),
            text: "Fix this!".to_string(),
            created_at: now,
        }],
    };

    // Create issue (should insert all fields including nested tables)
    store.create_issue(&issue)?;

    // Run Export
    let jsonl_path = dir.join("issues.jsonl");
    store.export_to_jsonl(&jsonl_path)?;

    // Verify JSONL
    let content = fs::read_to_string(&jsonl_path)?;
    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(lines.len(), 1);

    let exported_issue: Issue = serde_json::from_str(lines[0])?;
    assert_eq!(exported_issue.id, "bd-123456");
    assert_eq!(exported_issue.labels.len(), 2);
    assert!(exported_issue.labels.contains(&"bug".to_string()));
    assert_eq!(exported_issue.dependencies.len(), 1);
    assert_eq!(exported_issue.dependencies[0].depends_on_id, "bd-blocker");
    assert_eq!(exported_issue.comments.len(), 1);
    assert_eq!(exported_issue.comments[0].text, "Fix this!");
    assert_eq!(exported_issue.relates_to, vec!["bd-other".to_string()]);

    Ok(())
}
