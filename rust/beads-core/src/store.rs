use rusqlite::{Connection, Result, params};
use crate::models::{Issue, Dependency, Comment};
use chrono::{DateTime, Utc, NaiveDateTime};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufWriter, Write};
use sha2::{Digest, Sha256};

pub struct Store {
    conn: Connection,
    db_path: PathBuf,
}

impl Store {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(&path)?;

        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS issues (
                id TEXT PRIMARY KEY,
                content_hash TEXT DEFAULT '',
                title TEXT,
                description TEXT,
                design TEXT DEFAULT '',
                acceptance_criteria TEXT DEFAULT '',
                notes TEXT DEFAULT '',
                status TEXT,
                priority INTEGER,
                issue_type TEXT,
                assignee TEXT,
                estimated_minutes INTEGER,
                created_at TEXT,
                updated_at TEXT,
                closed_at TEXT,
                external_ref TEXT,
                sender TEXT DEFAULT '',
                ephemeral BOOLEAN DEFAULT 0,
                replies_to TEXT DEFAULT '',
                relates_to TEXT DEFAULT '',
                duplicate_of TEXT DEFAULT '',
                superseded_by TEXT DEFAULT '',
                deleted_at TEXT,
                deleted_by TEXT DEFAULT '',
                delete_reason TEXT DEFAULT '',
                original_type TEXT DEFAULT ''
            );

            CREATE TABLE IF NOT EXISTS labels (
                issue_id TEXT,
                label TEXT,
                PRIMARY KEY (issue_id, label)
            );

            CREATE TABLE IF NOT EXISTS dependencies (
                issue_id TEXT,
                depends_on_id TEXT,
                type TEXT,
                created_at TEXT,
                created_by TEXT,
                PRIMARY KEY (issue_id, depends_on_id, type)
            );

            CREATE TABLE IF NOT EXISTS comments (
                id INTEGER PRIMARY KEY,
                issue_id TEXT,
                author TEXT,
                text TEXT,
                created_at TEXT
            );

            CREATE TABLE IF NOT EXISTS dirty_issues (
                issue_id TEXT PRIMARY KEY
            );

            CREATE TABLE IF NOT EXISTS metadata (
                key TEXT PRIMARY KEY,
                value TEXT
            );

            CREATE TABLE IF NOT EXISTS config (
                key TEXT PRIMARY KEY,
                value TEXT
            );
            "
        )?;

        Ok(Store {
            conn,
            db_path: path.as_ref().to_path_buf(),
        })
    }

    pub fn get_config(&self, key: &str) -> Result<Option<String>> {
        let mut stmt = self.conn.prepare("SELECT value FROM config WHERE key = ?1")?;
        let mut rows = stmt.query([key])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }

    pub fn set_config(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO config (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn update_issue(&self, issue: &Issue) -> Result<()> {
        // Serialize nested fields
        let relates_to_json = serde_json::to_string(&issue.relates_to).unwrap_or_default();

        // Update main issue record
        self.conn.execute(
            "UPDATE issues SET
                title = ?2, description = ?3, status = ?4, priority = ?5, issue_type = ?6,
                created_at = ?7, updated_at = ?8, closed_at = ?9, external_ref = ?10,
                sender = ?11, ephemeral = ?12, replies_to = ?13, relates_to = ?14,
                duplicate_of = ?15, superseded_by = ?16,
                deleted_at = ?17, deleted_by = ?18, delete_reason = ?19, original_type = ?20,
                assignee = ?21, estimated_minutes = ?22
             WHERE id = ?1",
            params![
                &issue.id,
                &issue.title,
                &issue.description,
                &issue.status,
                &issue.priority,
                &issue.issue_type,
                issue.created_at.to_rfc3339(),
                issue.updated_at.to_rfc3339(),
                issue.closed_at.map(|t| t.to_rfc3339()),
                &issue.external_ref,
                &issue.sender,
                issue.ephemeral,
                &issue.replies_to,
                relates_to_json,
                &issue.duplicate_of,
                &issue.superseded_by,
                issue.deleted_at.map(|t| t.to_rfc3339()),
                &issue.deleted_by,
                &issue.delete_reason,
                &issue.original_type,
                &issue.assignee,
                &issue.estimated_minutes,
            ],
        )?;

        // Replace labels
        self.conn.execute("DELETE FROM labels WHERE issue_id = ?1", params![&issue.id])?;
        for label in &issue.labels {
            self.conn.execute(
                "INSERT INTO labels (issue_id, label) VALUES (?1, ?2)",
                params![&issue.id, label],
            )?;
        }

        // Replace dependencies
        self.conn.execute("DELETE FROM dependencies WHERE issue_id = ?1", params![&issue.id])?;
        for dep in &issue.dependencies {
            self.conn.execute(
                "INSERT INTO dependencies (issue_id, depends_on_id, type, created_at, created_by) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![&dep.issue_id, &dep.depends_on_id, &dep.type_, dep.created_at.to_rfc3339(), &dep.created_by],
            )?;
        }

        // Mark as dirty
        self.conn.execute(
            "INSERT OR IGNORE INTO dirty_issues (issue_id) VALUES (?1)",
            params![&issue.id],
        )?;
        Ok(())
    }

    pub fn list_config(&self) -> Result<Vec<(String, String)>> {
        let mut stmt = self.conn.prepare("SELECT key, value FROM config ORDER BY key")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    pub fn execute_raw(&self, sql: &str) -> Result<()> {
        self.conn.execute_batch(sql)
    }

    pub fn get_issue(&self, id: &str) -> Result<Option<Issue>> {
        // Handle short ID (prefix match)
        let query_id = if id.len() < 36 {
             format!("{}%", id)
        } else {
             id.to_string()
        };

        let mut stmt = self.conn.prepare(
            "SELECT
                id, content_hash, title, description, design, acceptance_criteria, notes,
                status, priority, issue_type, assignee, estimated_minutes,
                created_at, updated_at, closed_at, external_ref,
                sender, ephemeral, replies_to, relates_to, duplicate_of, superseded_by,
                deleted_at, deleted_by, delete_reason, original_type
             FROM issues
             WHERE id LIKE ?1
             LIMIT 1"
        )?;

        let mut rows = stmt.query([&query_id])?;

        let row = if let Some(row) = rows.next()? {
            row
        } else {
            return Ok(None);
        };

        let id: String = row.get(0)?;

        // Fetch children
        let mut labels = Vec::new();
        let mut labels_stmt = self.conn.prepare("SELECT label FROM labels WHERE issue_id = ?1")?;
        let labels_rows = labels_stmt.query_map([&id], |r| r.get(0))?;
        for l in labels_rows {
            labels.push(l?);
        }

        let mut deps = Vec::new();
        let mut deps_stmt = self.conn.prepare("SELECT depends_on_id, type, created_at, created_by FROM dependencies WHERE issue_id = ?1")?;
        let deps_rows = deps_stmt.query_map([&id], |r| {
             let created_at_s: String = r.get(2)?;
             let created_at = parse_timestamp(&created_at_s).unwrap_or_else(|| Utc::now());
             Ok(Dependency {
                 issue_id: id.clone(),
                 depends_on_id: r.get(0)?,
                 type_: r.get(1)?,
                 created_at,
                 created_by: r.get(3)?,
             })
        })?;
        for d in deps_rows {
            deps.push(d?);
        }

        let mut comments = Vec::new();
        let mut comments_stmt = self.conn.prepare("SELECT id, author, text, created_at FROM comments WHERE issue_id = ?1 ORDER BY created_at")?;
        let comments_rows = comments_stmt.query_map([&id], |r| {
             let created_at_s: String = r.get(3)?;
             let created_at = parse_timestamp(&created_at_s).unwrap_or_else(|| Utc::now());
             Ok(Comment {
                 id: r.get(0)?,
                 issue_id: id.clone(),
                 author: r.get(1)?,
                 text: r.get(2)?,
                 created_at,
             })
        })?;
        for c in comments_rows {
            comments.push(c?);
        }

        let created_at_s: String = row.get(12)?;
        let updated_at_s: String = row.get(13)?;
        let closed_at_s: Option<String> = row.get(14)?;
        let deleted_at_s: Option<String> = row.get(22)?;

        let created_at = parse_timestamp(&created_at_s).unwrap_or_else(|| Utc::now());
        let updated_at = parse_timestamp(&updated_at_s).unwrap_or_else(|| Utc::now());
        let closed_at = closed_at_s.and_then(|s| parse_timestamp(&s));
        let deleted_at = deleted_at_s.and_then(|s| parse_timestamp(&s));

        let relates_to_s: String = row.get(19).unwrap_or_default();
        let relates_to = if relates_to_s.is_empty() {
            Vec::new()
        } else {
            serde_json::from_str(&relates_to_s).unwrap_or_default()
        };

        Ok(Some(Issue {
            id,
            content_hash: row.get(1).unwrap_or_default(),
            title: row.get(2).unwrap_or_default(),
            description: row.get(3).unwrap_or_default(),
            design: row.get(4).unwrap_or_default(),
            acceptance_criteria: row.get(5).unwrap_or_default(),
            notes: row.get(6).unwrap_or_default(),
            status: row.get(7).unwrap_or_default(),
            priority: row.get(8).unwrap_or_default(),
            issue_type: row.get(9).unwrap_or_default(),
            assignee: row.get(10)?,
            estimated_minutes: row.get(11)?,
            created_at,
            updated_at,
            closed_at,
            external_ref: row.get(15)?,
            sender: row.get(16).unwrap_or_default(),
            ephemeral: row.get(17).unwrap_or(false),
            replies_to: row.get(18).unwrap_or_default(),
            relates_to,
            duplicate_of: row.get(20).unwrap_or_default(),
            superseded_by: row.get(21).unwrap_or_default(),
            deleted_at,
            deleted_by: row.get(23).unwrap_or_default(),
            delete_reason: row.get(24).unwrap_or_default(),
            original_type: row.get(25).unwrap_or_default(),
            labels,
            dependencies: deps,
            comments,
        }))
    }

    pub fn list_issues(&self, status: Option<&str>, assignee: Option<&str>, priority: Option<i32>, issue_type: Option<&str>) -> Result<Vec<Issue>> {
        // Build query dynamically
        let mut sql = "SELECT id, title, description, status, priority, issue_type, created_at, updated_at, assignee FROM issues WHERE 1=1".to_string();
        let mut params = Vec::new();

        if let Some(s) = status {
            sql.push_str(" AND status = ?");
            params.push(s.to_string());
        }
        if let Some(a) = assignee {
            if a == "unassigned" {
                sql.push_str(" AND (assignee IS NULL OR assignee = '')");
            } else {
                sql.push_str(" AND assignee = ?");
                params.push(a.to_string());
            }
        }
        // Special handling for priority/type to match CLI types if needed, but for now simple string/int binding
        if let Some(p) = priority {
            sql.push_str(" AND priority = ?");
            // params is Vec<String>, but we need to bind int.
            // rusqlite parameters are tricky with dynamic queries and mixed types if using positional params vector.
            // A common workaround is to convert all to explicit dyn ToSql or handle bindings manually.
            // Since we only have a few filters, let's use named parameters or rebuild params vector to be `&[&dyn ToSql]`.
            // But Vec<&dyn ToSql> is hard to manage with lifetimes of values.
            // Let's stick to the simpler approach: keep params separate and bind by index.
        }
        // Let's restart the approach to be safer with rusqlite.

        // We will construct the SQL string and a separate list of parameter values (as enum/trait objects).

        // However, rusqlite's query method takes params! or [] which expects things that implement ToSql.
        // It's easier to just construct the query string with values embedded if they are safe, BUT that's SQL injection risk.
        // So we MUST use parameters.

        // Let's assume we won't have too many combinations and use a simpler logic or a builder.
        // Or simply:

        let mut conditions = Vec::new();
        let mut args: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(s) = status {
            conditions.push("status = ?");
            args.push(Box::new(s.to_string()));
        }

        if let Some(a) = assignee {
            if a == "unassigned" {
                conditions.push("(assignee IS NULL OR assignee = '')");
            } else {
                conditions.push("assignee = ?");
                args.push(Box::new(a.to_string()));
            }
        }

        if let Some(p) = priority {
            conditions.push("priority = ?");
            args.push(Box::new(p));
        }

        if let Some(t) = issue_type {
            conditions.push("issue_type = ?");
            args.push(Box::new(t.to_string()));
        }

        let mut sql = "SELECT id, title, description, status, priority, issue_type, created_at, updated_at, assignee FROM issues".to_string();
        if !conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&conditions.join(" AND "));
        }
        sql.push_str(" ORDER BY created_at DESC");

        let mut stmt = self.conn.prepare(&sql)?;

        // We need to convert Vec<Box<dyn ToSql>> to slice of references.
        // This is a bit annoying in Rust.
        // Workaround: `rusqlite::params_from_iter`.

        let issue_iter = stmt.query_map(rusqlite::params_from_iter(args.iter()), |row| {
            let created_at_s: String = row.get(6)?;
            let updated_at_s: String = row.get(7)?;

            let created_at = parse_timestamp(&created_at_s).unwrap_or_else(|| Utc::now());
            let updated_at = parse_timestamp(&updated_at_s).unwrap_or_else(|| Utc::now());

            Ok(Issue {
                id: row.get(0)?,
                content_hash: String::new(),
                title: row.get(1)?,
                description: row.get(2)?,
                design: String::new(),
                acceptance_criteria: String::new(),
                notes: String::new(),
                status: row.get(3)?,
                priority: row.get(4)?,
                issue_type: row.get(5)?,
                assignee: None,
                estimated_minutes: None,
                created_at,
                updated_at,
                closed_at: None,
                external_ref: None,
                sender: String::new(),
                ephemeral: false,
                replies_to: String::new(),
                relates_to: Vec::new(),
                duplicate_of: String::new(),
                superseded_by: String::new(),

                deleted_at: None,
                deleted_by: String::new(),
                delete_reason: String::new(),
                original_type: String::new(),

                labels: Vec::new(),
                dependencies: Vec::new(),
                comments: Vec::new(),
            })
        })?;

        let mut issues = Vec::new();
        for issue in issue_iter {
            issues.push(issue?);
        }
        Ok(issues)
    }

    pub fn import_from_jsonl<P: AsRef<Path>>(&mut self, jsonl_path: P) -> anyhow::Result<()> {
        let jsonl_path = jsonl_path.as_ref();
        if !jsonl_path.exists() {
            return Ok(());
        }

        let file = File::open(jsonl_path)?;
        let reader = std::io::BufReader::new(file);

        let tx = self.conn.transaction()?;

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let issue: Issue = serde_json::from_str(&line)?;

            // Serialize nested fields
            let relates_to_json = serde_json::to_string(&issue.relates_to).unwrap_or_default();

            tx.execute(
                "INSERT OR REPLACE INTO issues (
                    id, title, description, status, priority, issue_type,
                    created_at, updated_at, closed_at, external_ref,
                    sender, ephemeral, replies_to, relates_to, duplicate_of, superseded_by,
                    deleted_at, deleted_by, delete_reason, original_type
                )
                 VALUES (
                    ?1, ?2, ?3, ?4, ?5, ?6,
                    ?7, ?8, ?9, ?10,
                    ?11, ?12, ?13, ?14, ?15, ?16,
                    ?17, ?18, ?19, ?20
                )",
                params![
                    &issue.id,
                    &issue.title,
                    &issue.description,
                    &issue.status,
                    &issue.priority,
                    &issue.issue_type,
                    issue.created_at.to_rfc3339(),
                    issue.updated_at.to_rfc3339(),
                    issue.closed_at.map(|t| t.to_rfc3339()),
                    &issue.external_ref,
                    &issue.sender,
                    issue.ephemeral,
                    &issue.replies_to,
                    relates_to_json,
                    &issue.duplicate_of,
                    &issue.superseded_by,
                    issue.deleted_at.map(|t| t.to_rfc3339()),
                    &issue.deleted_by,
                    &issue.delete_reason,
                    &issue.original_type,
                ],
            )?;

            // Insert labels
            tx.execute("DELETE FROM labels WHERE issue_id = ?1", params![&issue.id])?;
            for label in &issue.labels {
                tx.execute(
                    "INSERT INTO labels (issue_id, label) VALUES (?1, ?2)",
                    params![&issue.id, label],
                )?;
            }

            // Insert dependencies
            tx.execute("DELETE FROM dependencies WHERE issue_id = ?1", params![&issue.id])?;
            for dep in &issue.dependencies {
                tx.execute(
                    "INSERT INTO dependencies (issue_id, depends_on_id, type, created_at, created_by) VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![&dep.issue_id, &dep.depends_on_id, &dep.type_, dep.created_at.to_rfc3339(), &dep.created_by],
                )?;
            }

            // Insert comments
            // Fetch existing comments for this issue to dedupe in memory (faster than many SELECTs)
            let mut stmt = tx.prepare_cached("SELECT author, text FROM comments WHERE issue_id = ?1")?;
            let existing_comments: std::collections::HashSet<(String, String)> = stmt
                .query_map([&issue.id], |row| Ok((row.get(0)?, row.get(1)?)))?
                .filter_map(Result::ok)
                .collect();
            drop(stmt); // Release borrow

            for comment in &issue.comments {
                if !existing_comments.contains(&(comment.author.clone(), comment.text.clone())) {
                     tx.execute(
                        "INSERT INTO comments (issue_id, author, text, created_at) VALUES (?1, ?2, ?3, ?4)",
                        params![&issue.id, &comment.author, &comment.text, comment.created_at.to_rfc3339()],
                    )?;
                }
            }
        }

        tx.commit()?;
        Ok(())
    }

    pub fn create_issue(&self, issue: &Issue) -> Result<()> {
        // Serialize nested fields
        let relates_to_json = serde_json::to_string(&issue.relates_to).unwrap_or_default();

        self.conn.execute(
            "INSERT INTO issues (
                id, title, description, status, priority, issue_type,
                created_at, updated_at, closed_at, external_ref,
                sender, ephemeral, replies_to, relates_to, duplicate_of, superseded_by,
                deleted_at, deleted_by, delete_reason, original_type
            )
             VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6,
                ?7, ?8, ?9, ?10,
                ?11, ?12, ?13, ?14, ?15, ?16,
                ?17, ?18, ?19, ?20
            )",
            params![
                &issue.id,
                &issue.title,
                &issue.description,
                &issue.status,
                &issue.priority,
                &issue.issue_type,
                issue.created_at.to_rfc3339(),
                issue.updated_at.to_rfc3339(),
                issue.closed_at.map(|t| t.to_rfc3339()),
                &issue.external_ref,
                &issue.sender,
                issue.ephemeral,
                &issue.replies_to,
                relates_to_json,
                &issue.duplicate_of,
                &issue.superseded_by,
                issue.deleted_at.map(|t| t.to_rfc3339()),
                &issue.deleted_by,
                &issue.delete_reason,
                &issue.original_type,
            ],
        )?;

        // Insert labels
        for label in &issue.labels {
            self.conn.execute(
                "INSERT INTO labels (issue_id, label) VALUES (?1, ?2)",
                params![&issue.id, label],
            )?;
        }

        // Insert dependencies
        for dep in &issue.dependencies {
            self.conn.execute(
                "INSERT INTO dependencies (issue_id, depends_on_id, type, created_at, created_by) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![&dep.issue_id, &dep.depends_on_id, &dep.type_, dep.created_at.to_rfc3339(), &dep.created_by],
            )?;
        }

        // Insert comments
        for comment in &issue.comments {
            self.conn.execute(
                "INSERT INTO comments (issue_id, author, text, created_at) VALUES (?1, ?2, ?3, ?4)",
                params![&comment.issue_id, &comment.author, &comment.text, comment.created_at.to_rfc3339()],
            )?;
        }

        // Mark as dirty so Go 'bd export' picks it up
        self.conn.execute(
            "INSERT OR IGNORE INTO dirty_issues (issue_id) VALUES (?1)",
            params![&issue.id],
        )?;
        Ok(())
    }

    pub fn export_to_jsonl<P: AsRef<Path>>(&self, jsonl_path: P) -> anyhow::Result<()> {
        let issues = self.export_all_issues()?;
        let jsonl_path = jsonl_path.as_ref();

        // Write to temp file
        let dir = jsonl_path.parent().unwrap_or_else(|| Path::new("."));
        let file_name = jsonl_path.file_name().unwrap_or_default();
        let temp_path = dir.join(format!(".{}.tmp", file_name.to_string_lossy()));

        {
            let file = File::create(&temp_path)?;
            let mut writer = BufWriter::new(file);

            for issue in &issues {
                let json = serde_json::to_string(issue)?;
                writeln!(writer, "{}", json)?;
            }
            writer.flush()?;
        } // file closed here

        // Rename temp file to target
        std::fs::rename(&temp_path, jsonl_path)?;

        // Compute hash of the new file
        let mut file = File::open(jsonl_path)?;
        let mut hasher = Sha256::new();
        std::io::copy(&mut file, &mut hasher)?;
        let hash = hex::encode(hasher.finalize());

        // Update metadata and clear dirty
        self.conn.execute(
            "INSERT OR REPLACE INTO metadata (key, value) VALUES (?1, ?2)",
            params!["jsonl_content_hash", &hash],
        )?;

        // Clear dirty issues
        self.conn.execute("DELETE FROM dirty_issues", [])?;

        // Update database mtime (touch) to match JSONL (prevent drift)
        // We'll skip explicit touch for now as DB write above updates mtime.

        Ok(())
    }

    fn export_all_issues(&self) -> Result<Vec<Issue>> {
        // Fetch all related data first (bulk)
        let labels_map = self.get_all_labels()?;
        let deps_map = self.get_all_dependencies()?;
        let comments_map = self.get_all_comments()?;

        let mut stmt = self.conn.prepare(
            "SELECT
                id, content_hash, title, description, design, acceptance_criteria, notes,
                status, priority, issue_type, assignee, estimated_minutes,
                created_at, updated_at, closed_at, external_ref,
                sender, ephemeral, replies_to, relates_to, duplicate_of, superseded_by,
                deleted_at, deleted_by, delete_reason, original_type
             FROM issues
             ORDER BY id"
        )?;

        let issue_iter = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let created_at_s: String = row.get(12)?;
            let updated_at_s: String = row.get(13)?;
            let closed_at_s: Option<String> = row.get(14)?;
            let deleted_at_s: Option<String> = row.get(22)?;

            let created_at = parse_timestamp(&created_at_s).unwrap_or_else(|| Utc::now());
            let updated_at = parse_timestamp(&updated_at_s).unwrap_or_else(|| Utc::now());
            let closed_at = closed_at_s.and_then(|s| parse_timestamp(&s));
            let deleted_at = deleted_at_s.and_then(|s| parse_timestamp(&s));

            let relates_to_s: String = row.get(19).unwrap_or_default();
            let relates_to = if relates_to_s.is_empty() {
                Vec::new()
            } else {
                serde_json::from_str(&relates_to_s).unwrap_or_default()
            };

            Ok(Issue {
                id: id.clone(),
                content_hash: row.get(1).unwrap_or_default(),
                title: row.get(2).unwrap_or_default(),
                description: row.get(3).unwrap_or_default(),
                design: row.get(4).unwrap_or_default(),
                acceptance_criteria: row.get(5).unwrap_or_default(),
                notes: row.get(6).unwrap_or_default(),
                status: row.get(7).unwrap_or_default(),
                priority: row.get(8).unwrap_or_default(),
                issue_type: row.get(9).unwrap_or_default(),
                assignee: row.get(10)?,
                estimated_minutes: row.get(11)?,
                created_at,
                updated_at,
                closed_at,
                external_ref: row.get(15)?,
                sender: row.get(16).unwrap_or_default(),
                ephemeral: row.get(17).unwrap_or(false),
                replies_to: row.get(18).unwrap_or_default(),
                relates_to,
                duplicate_of: row.get(20).unwrap_or_default(),
                superseded_by: row.get(21).unwrap_or_default(),

                deleted_at,
                deleted_by: row.get(23).unwrap_or_default(),
                delete_reason: row.get(24).unwrap_or_default(),
                original_type: row.get(25).unwrap_or_default(),

                labels: labels_map.get(&id).cloned().unwrap_or_default(),
                dependencies: deps_map.get(&id).cloned().unwrap_or_default(),
                comments: comments_map.get(&id).cloned().unwrap_or_default(),
            })
        })?;

        let mut issues = Vec::new();
        for issue in issue_iter {
            issues.push(issue?);
        }
        Ok(issues)
    }

    fn get_all_labels(&self) -> Result<HashMap<String, Vec<String>>> {
        let mut stmt = self.conn.prepare("SELECT issue_id, label FROM labels")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        let mut map: HashMap<String, Vec<String>> = HashMap::new();
        for row in rows {
            let (id, label) = row?;
            map.entry(id).or_default().push(label);
        }
        Ok(map)
    }

    fn get_all_dependencies(&self) -> Result<HashMap<String, Vec<Dependency>>> {
        let mut stmt = self.conn.prepare(
            "SELECT issue_id, depends_on_id, type, created_at, created_by FROM dependencies"
        )?;
        let rows = stmt.query_map([], |row| {
            let created_at_s: String = row.get(3)?;
            let created_at = parse_timestamp(&created_at_s).unwrap_or_else(|| Utc::now());

            Ok(Dependency {
                issue_id: row.get(0)?,
                depends_on_id: row.get(1)?,
                type_: row.get(2)?,
                created_at,
                created_by: row.get(4)?,
            })
        })?;

        let mut map: HashMap<String, Vec<Dependency>> = HashMap::new();
        for row in rows {
            let dep = row?;
            map.entry(dep.issue_id.clone()).or_default().push(dep);
        }
        Ok(map)
    }

    fn get_all_comments(&self) -> Result<HashMap<String, Vec<Comment>>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, issue_id, author, text, created_at FROM comments"
        )?;
        let rows = stmt.query_map([], |row| {
            let created_at_s: String = row.get(4)?;
            let created_at = parse_timestamp(&created_at_s).unwrap_or_else(|| Utc::now());

            Ok(Comment {
                id: row.get(0)?,
                issue_id: row.get(1)?,
                author: row.get(2)?,
                text: row.get(3)?,
                created_at,
            })
        })?;

        let mut map: HashMap<String, Vec<Comment>> = HashMap::new();
        for row in rows {
            let comment = row?;
            map.entry(comment.issue_id.clone()).or_default().push(comment);
        }
        Ok(map)
    }
}

fn parse_timestamp(s: &str) -> Option<DateTime<Utc>> {
    // Try RFC3339
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some(dt.with_timezone(&Utc));
    }
    // Try SQLite default format (YYYY-MM-DD HH:MM:SS) - assumes UTC
    if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return Some(DateTime::from_naive_utc_and_offset(dt, Utc));
    }
    None
}
