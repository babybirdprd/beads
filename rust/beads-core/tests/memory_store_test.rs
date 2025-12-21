use beads_core::models::{Comment, Dependency};
use beads_core::{FileSystem, Issue, MemoryStore, StdFileSystem, Store};
use chrono::Utc;
use std::fs;

#[test]
fn test_memory_store_crud() -> anyhow::Result<()> {
    // Setup
    let store = MemoryStore::new();

    // 1. Create Issue
    let now = Utc::now();
    let issue = Issue {
        id: "bd-mem-123".to_string(),
        title: "Memory Test".to_string(),
        description: "In memory".to_string(),
        status: "open".to_string(),
        priority: 1,
        issue_type: "task".to_string(),
        created_at: now,
        updated_at: now,
        labels: vec!["mem".to_string()],
        dependencies: vec![Dependency {
            issue_id: "bd-mem-123".to_string(),
            depends_on_id: "bd-other".to_string(),
            type_: "blocks".to_string(),
            created_at: now,
            created_by: "me".to_string(),
        }],
        comments: vec![Comment {
            id: 1,
            issue_id: "bd-mem-123".to_string(),
            author: "me".to_string(),
            text: "comment".to_string(),
            created_at: now,
        }],
        ..Default::default()
    };

    store.create_issue(&issue)?;

    // 2. Get Issue
    let fetched = store.get_issue("bd-mem-123")?.expect("Issue should exist");
    assert_eq!(fetched.title, "Memory Test");
    assert_eq!(fetched.labels, vec!["mem".to_string()]);
    assert_eq!(fetched.dependencies.len(), 1);
    assert_eq!(fetched.comments.len(), 1);

    // 3. Update Issue
    let mut updated = fetched.clone();
    updated.status = "closed".to_string();
    updated.labels.push("closed".to_string());
    store.update_issue(&updated)?;

    let fetched2 = store.get_issue("bd-mem-123")?.expect("Issue should exist");
    assert_eq!(fetched2.status, "closed");
    assert_eq!(fetched2.labels.len(), 2);

    // 4. List Issues
    let list = store.list_issues(None, None, None, None, None, None)?;
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, "bd-mem-123");

    // 5. Config
    store.set_config("user.name", "Memory User")?;
    let config_val = store.get_config("user.name")?;
    assert_eq!(config_val, Some("Memory User".to_string()));

    Ok(())
}

#[test]
fn test_memory_store_export_import() -> anyhow::Result<()> {
    let dir = std::env::temp_dir().join("beads_mem_test");
    fs::create_dir_all(&dir)?;
    let jsonl_path = dir.join("issues.jsonl");

    let store = MemoryStore::new();
    let now = Utc::now();
    let issue = Issue {
        id: "bd-mem-export".to_string(),
        title: "Export Me".to_string(),
        created_at: now,
        updated_at: now,
        ..Default::default()
    };
    store.create_issue(&issue)?;

    let fs_impl = StdFileSystem;
    store.export_to_jsonl(&jsonl_path, &fs_impl)?;

    assert!(jsonl_path.exists());

    // Import into new store
    let mut store2 = MemoryStore::new();
    store2.import_from_jsonl(&jsonl_path, &fs_impl)?;

    let fetched = store2.get_issue("bd-mem-export")?;
    assert!(fetched.is_some());
    assert_eq!(fetched.unwrap().title, "Export Me");

    Ok(())
}
