use rusqlite::{Connection, Result};
use crate::models::{Issue};
use chrono::{DateTime, Utc, NaiveDateTime};
use std::path::Path;

pub struct Store {
    conn: Connection,
}

impl Store {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        Ok(Store { conn })
    }

    pub fn list_issues(&self) -> Result<Vec<Issue>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, description, status, priority, issue_type, created_at, updated_at
             FROM issues"
        )?;

        let issue_iter = stmt.query_map([], |row| {
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
                relates_to: String::new(),
                duplicate_of: String::new(),
                superseded_by: String::new(),
            })
        })?;

        let mut issues = Vec::new();
        for issue in issue_iter {
            issues.push(issue?);
        }
        Ok(issues)
    }

    pub fn create_issue(&self, issue: &Issue) -> Result<()> {
        self.conn.execute(
            "INSERT INTO issues (id, title, description, status, priority, issue_type, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            (
                &issue.id,
                &issue.title,
                &issue.description,
                &issue.status,
                &issue.priority,
                &issue.issue_type,
                issue.created_at.to_rfc3339(),
                issue.updated_at.to_rfc3339(),
            ),
        )?;
        // Mark as dirty so Go 'bd export' picks it up
        self.conn.execute(
            "INSERT OR IGNORE INTO dirty_issues (issue_id) VALUES (?1)",
            (&issue.id,),
        )?;
        Ok(())
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
