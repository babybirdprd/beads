use clap::{Parser, Subcommand};
use beads_core::{Store, Issue, util};
use chrono::Utc;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "bd")]
#[command(about = "Beads Issue Tracker (Rust Port)")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    List,
    Create {
        title: String,
        #[arg(short, long, default_value = "")]
        description: String,
        #[arg(short = 't', long = "type", default_value = "bug")]
        type_: String,
        #[arg(short, long, default_value_t = 2)]
        priority: i32,
    },
    Export {
        #[arg(short, long, default_value = ".beads/issues.jsonl")]
        output: String,
    },
    Merge {
        output: String,
        base: String,
        left: String,
        right: String,
        #[arg(long)]
        debug: bool,
    },
    Sync,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Find DB
    let db_path = find_db_path();
    // Ensure parent dir exists if we are creating
    if let Commands::Create { .. } = cli.command {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
    }
    // Ensure output dir exists if we are exporting
    if let Commands::Export { output } = &cli.command {
         if let Some(parent) = std::path::Path::new(output).parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
    }

    let store = Store::open(&db_path).map_err(|e| anyhow::anyhow!("Failed to open DB at {:?}: {}", db_path, e))?;

    match cli.command {
        Commands::List => {
            let issues = store.list_issues()?;
            println!("{:<10} {:<10} {:<10} {}", "ID", "STATUS", "PRIORITY", "TITLE");
            println!("{:-<60}", "");
            for issue in issues {
                println!("{:<10} {:<10} {:<10} {}", issue.id, issue.status, issue.priority, issue.title);
            }
        }
        Commands::Export { output } => {
            store.export_to_jsonl(&output)?;
            // Output nothing on success unless verbose?
            // Go tool is silent on success?
            // Go: "Exporting pending changes to JSONL..." then silent if no error.
            // But if called via "bd export", it might be silent.
            // beads-cli is primarily for sync, so keep it minimal.
            // But helpful feedback is good.
            println!("Exported issues to {}", output);
        }
        Commands::Merge { output, base, left, right, debug } => {
            beads_core::merge::merge3way(&output, &base, &left, &right, debug)?;
        }
        Commands::Sync => {
            let beads_dir = db_path.parent().unwrap();
            let git_root = beads_dir.parent().unwrap_or(std::path::Path::new("."));
            let jsonl_path = beads_dir.join("issues.jsonl");
            beads_core::sync::run_sync(&store, git_root, &jsonl_path)?;
            println!("Sync complete.");
        }
        Commands::Create { title, description, type_, priority } => {
            let now = Utc::now();
            // TODO: Real workspace ID from config
            let id_hash = util::generate_hash_id(&title, &description, now, "default-workspace");
            let short_id = format!("bd-{}", id_hash);

            let issue = Issue {
                id: short_id.clone(),
                content_hash: String::new(),
                title,
                description,
                design: String::new(),
                acceptance_criteria: String::new(),
                notes: String::new(),
                status: "open".to_string(),
                priority,
                issue_type: type_,
                assignee: None,
                estimated_minutes: None,
                created_at: now,
                updated_at: now,
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
            };

            store.create_issue(&issue)?;
            println!("Created issue {}", short_id);
        }
    }

    Ok(())
}

fn find_db_path() -> PathBuf {
    // Check current dir .beads/beads.db
    let p = PathBuf::from(".beads/beads.db");
    if p.exists() {
        return p;
    }
    // Check parent dir (for development)
    let p_parent = PathBuf::from("../.beads/beads.db");
    if p_parent.exists() {
        return p_parent;
    }
    // Fallback to relative
    p
}
