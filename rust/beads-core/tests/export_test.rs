
use beads_core::{Store, Issue, Comment, StdFileSystem};
use chrono::Utc;
use tempfile::tempdir;

#[test]
fn test_export_to_jsonl() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let db_path = dir.path().join("beads.db");
    let store = Store::open(&db_path)?;

    // Create an issue with labels, deps, comments
    let issue = Issue {
        id: "bd-test1".to_string(),
        title: "Test Issue".to_string(),
        description: "Desc".to_string(),
        status: "open".to_string(),
        priority: 1,
        issue_type: "bug".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        labels: vec!["label1".to_string(), "label2".to_string()],
        comments: vec![
            Comment {
                id: 1,
                issue_id: "bd-test1".to_string(),
                author: "me".to_string(),
                text: "comment1".to_string(),
                created_at: Utc::now(),
            }
        ],
        ..Default::default()
    };

    store.create_issue(&issue)?;

    let jsonl_path = dir.path().join("issues.jsonl");
    let fs = StdFileSystem;
    store.export_to_jsonl(&jsonl_path, &fs)?;

    let content = std::fs::read_to_string(&jsonl_path)?;
    assert!(content.contains("\"id\":\"bd-test1\""));
    assert!(content.contains("\"labels\":[\"label1\",\"label2\"]"));
    // Check comment presence
    assert!(content.contains("\"text\":\"comment1\""));

    Ok(())
}
