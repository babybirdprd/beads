use clap::{Parser, Subcommand};
use beads_core::{Store, Issue, util};
use chrono::Utc;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use std::io::Write;

#[derive(Parser)]
#[command(name = "bd")]
#[command(about = "Beads Issue Tracker (Rust Port)")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    List {
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        assignee: Option<String>,
        #[arg(long)]
        priority: Option<i32>,
        #[arg(long = "type")]
        type_: Option<String>,
        #[arg(long)]
        label: Option<String>,
        #[arg(long)]
        sort: Option<String>,
    },
    Show {
        id: String,
    },
    Update {
        id: String,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        priority: Option<i32>,
        #[arg(long = "type")]
        type_: Option<String>,
        #[arg(long)]
        assignee: Option<String>,
    },
    Edit {
        id: String,
    },
    Close {
        id: String,
    },
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
    Import {
        #[arg(short, long, default_value = ".beads/issues.jsonl")]
        input: String,
    },
    Merge {
        output: String,
        base: String,
        left: String,
        right: String,
        #[arg(long)]
        debug: bool,
    },
    Onboard,
    Ready,
    Sync,
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
    Set { key: String, value: String },
    Get { key: String },
    List,
}

#[derive(Debug, Serialize, Deserialize)]
struct IssueFrontmatter {
    title: String,
    status: String,
    priority: i32,
    #[serde(rename = "type")]
    issue_type: String,
    #[serde(default)]
    assignee: Option<String>,
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    // Find DB
    let db_path = if matches!(cli.command, Commands::Onboard) {
         PathBuf::from(".beads/beads.db")
    } else {
         find_db_path()
    };

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

    // Ensure parent dir exists if we are onboarding
    if let Commands::Onboard = &cli.command {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let mut store = Store::open(&db_path).map_err(|e| anyhow::anyhow!("Failed to open DB at {:?}: {}", db_path, e))?;

    match cli.command {
        Commands::List { status, assignee, priority, type_, label, sort } => {
            let issues = store.list_issues(status.as_deref(), assignee.as_deref(), priority, type_.as_deref(), label.as_deref(), sort.as_deref())?;

            use comfy_table::{Table, Row, Cell};
            use comfy_table::modifiers::UTF8_ROUND_CORNERS;
            use comfy_table::presets::UTF8_FULL;
            use colored::Colorize;

            let mut table = Table::new();
            table.load_preset(UTF8_FULL)
                 .apply_modifier(UTF8_ROUND_CORNERS)
                 .set_content_arrangement(comfy_table::ContentArrangement::Dynamic);

            table.set_header(vec!["ID", "Status", "Priority", "Title"]);

            for issue in issues {
                let status_str = issue.status.clone();
                let status_cell = if status_str == "bug" {
                    Cell::new(&status_str).fg(comfy_table::Color::Red)
                } else if status_str == "closed" {
                    Cell::new(&status_str).fg(comfy_table::Color::Green)
                } else if status_str == "open" {
                    Cell::new(&status_str).fg(comfy_table::Color::Yellow)
                } else {
                     Cell::new(&status_str)
                };

                let title_truncated = if issue.title.len() > 60 {
                    format!("{}...", &issue.title[..57])
                } else {
                    issue.title.clone()
                };

                table.add_row(vec![
                    Cell::new(&issue.id),
                    status_cell,
                    Cell::new(issue.priority),
                    Cell::new(title_truncated),
                ]);
            }
            println!("{}", table);
        }
        Commands::Show { id } => {
            if let Some(issue) = store.get_issue(&id)? {
                println!("ID:          {}", issue.id);
                println!("Title:       {}", issue.title);
                println!("Status:      {}", issue.status);
                println!("Priority:    {}", issue.priority);
                println!("Type:        {}", issue.issue_type);
                if let Some(assignee) = &issue.assignee {
                    println!("Assignee:    {}", assignee);
                }
                println!("Created:     {}", issue.created_at);
                println!("Updated:     {}", issue.updated_at);
                println!("------------------------------------------------------------");
                println!("{}", issue.description);

                if !issue.labels.is_empty() {
                    println!("\nLabels: {}", issue.labels.join(", "));
                }

                if !issue.dependencies.is_empty() {
                    println!("\nDependencies:");
                    for dep in issue.dependencies {
                        println!("  {} ({})", dep.depends_on_id, dep.type_);
                    }
                }

                if !issue.comments.is_empty() {
                    println!("\nComments:");
                    for comment in issue.comments {
                        println!("  {} at {}:", comment.author, comment.created_at);
                        println!("    {}", comment.text);
                    }
                }
            } else {
                eprintln!("Issue not found: {}", id);
            }
        }
        Commands::Update { id, title, description, status, priority, type_, assignee } => {
            if let Some(mut issue) = store.get_issue(&id)? {
                let mut updated = false;
                if let Some(t) = title { issue.title = t; updated = true; }
                if let Some(d) = description { issue.description = d; updated = true; }
                if let Some(s) = status { issue.status = s; updated = true; }
                if let Some(p) = priority { issue.priority = p; updated = true; }
                if let Some(t) = type_ { issue.issue_type = t; updated = true; }
                if let Some(a) = assignee {
                    issue.assignee = if a.is_empty() { None } else { Some(a) };
                    updated = true;
                }

                if updated {
                    issue.updated_at = Utc::now();
                    store.update_issue(&issue)?;
                    println!("Updated issue {}", issue.id);
                } else {
                    println!("No changes provided.");
                }
            } else {
                eprintln!("Issue not found: {}", id);
            }
        }
        Commands::Edit { id } => {
            if let Some(mut issue) = store.get_issue(&id)? {
                let frontmatter = IssueFrontmatter {
                    title: issue.title.clone(),
                    status: issue.status.clone(),
                    priority: issue.priority,
                    issue_type: issue.issue_type.clone(),
                    assignee: issue.assignee.clone(),
                };

                let yaml = serde_yaml::to_string(&frontmatter)?;
                let content = format!("---\n{}---\n\n{}", yaml, issue.description);

                let mut file = tempfile::Builder::new()
                    .suffix(".md")
                    .tempfile()?;
                write!(file, "{}", content)?;

                let path = file.path().to_owned();
                file.keep()?; // Keep the file so editor can open it, we'll delete later or let OS handle tmp

                edit::edit_file(&path)?;

                let new_content = std::fs::read_to_string(&path)?;
                std::fs::remove_file(path)?;

                // Parse
                if new_content.starts_with("---") {
                    let parts: Vec<&str> = new_content.splitn(3, "---").collect();
                    if parts.len() >= 3 {
                        let yaml_part = parts[1];
                        let body_part = parts[2].trim().to_string();

                        let new_fm: IssueFrontmatter = serde_yaml::from_str(yaml_part)
                            .map_err(|e| anyhow::anyhow!("Invalid frontmatter: {}", e))?;

                        issue.title = new_fm.title;
                        issue.status = new_fm.status;
                        issue.priority = new_fm.priority;
                        issue.issue_type = new_fm.issue_type;
                        issue.assignee = new_fm.assignee;
                        issue.description = body_part;
                        issue.updated_at = Utc::now();

                        store.update_issue(&issue)?;
                        println!("Updated issue {}", issue.id);
                    } else {
                        eprintln!("Invalid format: missing frontmatter delimiters");
                    }
                } else {
                     // Assume just description if no frontmatter?
                     // Or error out? Better to be safe.
                     eprintln!("Invalid format: file must start with ---");
                }
            } else {
                eprintln!("Issue not found: {}", id);
            }
        }
        Commands::Close { id } => {
            if let Some(mut issue) = store.get_issue(&id)? {
                if issue.status != "closed" {
                    issue.status = "closed".to_string();
                    issue.closed_at = Some(Utc::now());
                    issue.updated_at = Utc::now();
                    store.update_issue(&issue)?;
                    println!("Closed issue {}", issue.id);
                } else {
                    println!("Issue {} is already closed.", issue.id);
                }
            } else {
                eprintln!("Issue not found: {}", id);
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
        Commands::Import { input } => {
            store.import_from_jsonl(&input)?;
            println!("Imported issues from {}", input);
        }
        Commands::Merge { output, base, left, right, debug } => {
            beads_core::merge::merge3way(&output, &base, &left, &right, debug)?;
        }
        Commands::Onboard => {
            // Check git init
            if !std::path::Path::new(".git").exists() {
                println!("Not a git repository. Initializing...");
                std::process::Command::new("git")
                    .arg("init")
                    .status()?;
            }

            // Create .beads
            let beads_dir = std::path::Path::new(".beads");
            if !beads_dir.exists() {
                std::fs::create_dir(beads_dir)?;
                println!("Created .beads directory.");
            }

            // Create .gitignore
            let gitignore_path = std::path::Path::new(".gitignore");
            let mut gitignore_content = String::new();
            if gitignore_path.exists() {
                 gitignore_content = std::fs::read_to_string(gitignore_path)?;
            }
            if !gitignore_content.contains("beads.db") {
                println!("Adding beads.db to .gitignore...");
                use std::io::Write;
                let mut file = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(gitignore_path)?;
                writeln!(file, "\n.beads/beads.db")?;
            }

            // User config
            // Try to read git config
            let output = std::process::Command::new("git")
                .args(["config", "user.name"])
                .output();

            let default_user = if let Ok(out) = output {
                String::from_utf8_lossy(&out.stdout).trim().to_string()
            } else {
                String::new()
            };

            print!("Enter your username [{}]: ", default_user);
            use std::io::Write;
            std::io::stdout().flush()?;

            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            let input = input.trim();

            let user = if input.is_empty() {
                default_user
            } else {
                input.to_string()
            };

            if !user.is_empty() {
                store.set_config("user.name", &user)?;
                println!("Configured user.name = {}", user);
            } else {
                println!("No user configured.");
            }

            println!("Onboarding complete!");
        }
        Commands::Ready => {
            // Get current user
            let user = store.get_config("user.name")?;
            let assignee = user.as_deref().unwrap_or("unassigned");

            // List issues not closed, assigned to user or unassigned (if user not set?)
            // Requirement: "alias for listing open issues assigned to user or unassigned"
            // If we have a user, we filter by that user.
            // If we don't have a user, maybe list unassigned?

            // Let's implement: Status != closed AND Assignee = <user>
            // But list_issues currently filters via exact match or unassigned.
            // Store::list_issues doesn't support "NOT closed". It supports "status = ?"
            // So we might need to filter in memory or fetch "open", "in_progress" separately?
            // "open" is default status. "closed" is closed.
            // We usually want everything NOT closed.
            // Since `list_issues` takes specific status, we can't easily say "not closed".
            // Let's fetch all and filter in memory for now, or fetch by common open statuses.
            // Given the limited "list_issues" SQL generation I wrote (AND logic), fetching all then filtering is safest without changing Store again.
            // Wait, I can pass None for status (all) and filter in loop.

            let all_issues = store.list_issues(None, None, None, None, None, None)?;

            println!("Ready issues for {}:", assignee);
            println!("{:<10} {:<10} {:<10} {}", "ID", "STATUS", "PRIORITY", "TITLE");
            println!("{:-<60}", "");

            for issue in all_issues {
                if issue.status == "closed" {
                    continue;
                }

                let matches_assignee = if let Some(a) = &issue.assignee {
                    if let Some(u) = &user {
                        a == u
                    } else {
                         // No user configured.
                         // Should we show unassigned? Or everything?
                         // "bd ready" implies "ready for ME".
                         // If I don't know who ME is, I can't filter by assignee efficiently.
                         // Maybe just show unassigned?
                         // Let's assume matches_assignee = true if user is None (show all open?) or false?
                         // Go with: if user is known, match it. If not, match nothing?
                         // Or maybe prompt user to configure user.name?
                         false
                    }
                } else {
                     // Issue is unassigned.
                     // Often "ready" queue includes unassigned issues one could pick up.
                     // Let's include unassigned.
                     true
                };

                if matches_assignee {
                    println!("{:<10} {:<10} {:<10} {}", issue.id, issue.status, issue.priority, issue.title);
                }
            }
        }
        Commands::Sync => {
            let beads_dir = db_path.parent().unwrap();
            let git_root = beads_dir.parent().unwrap_or(std::path::Path::new("."));
            let jsonl_path = beads_dir.join("issues.jsonl");
            beads_core::sync::run_sync(&mut store, git_root, &jsonl_path)?;
            println!("Sync complete.");
        }
        Commands::Config { command } => match command {
            ConfigCommands::Set { key, value } => {
                store.set_config(&key, &value)?;
                println!("{} = {}", key, value);
            }
            ConfigCommands::Get { key } => {
                if let Some(val) = store.get_config(&key)? {
                    println!("{}", val);
                } else {
                    eprintln!("Key not found: {}", key);
                }
            }
            ConfigCommands::List => {
                let items = store.list_config()?;
                for (k, v) in items {
                    println!("{} = {}", k, v);
                }
            }
        },
        Commands::Create { title, mut description, type_, priority } => {
            // Interactive editing if description is empty
            if description.is_empty() {
                 let frontmatter = IssueFrontmatter {
                    title: title.clone(),
                    status: "open".to_string(),
                    priority,
                    issue_type: type_.clone(),
                    assignee: None,
                };

                let yaml = serde_yaml::to_string(&frontmatter)?;
                let content = format!("---\n{}---\n\n{}", yaml, "");

                let mut file = tempfile::Builder::new()
                    .suffix(".md")
                    .tempfile()?;
                write!(file, "{}", content)?;

                let path = file.path().to_owned();
                file.keep()?;

                edit::edit_file(&path)?;

                let new_content = std::fs::read_to_string(&path)?;
                std::fs::remove_file(path)?;

                 if new_content.starts_with("---") {
                    let parts: Vec<&str> = new_content.splitn(3, "---").collect();
                    if parts.len() >= 3 {
                         let body_part = parts[2].trim().to_string();
                         // We could also parse the YAML to allow user to change title/priority/type during creation!
                         // This is better UX.
                         let yaml_part = parts[1];
                         match serde_yaml::from_str::<IssueFrontmatter>(yaml_part) {
                             Ok(new_fm) => {
                                 // Let's assume we use the parsed values if valid.
                                 description = body_part;

                                 let now = Utc::now();
                                 // Use new_fm values
                                 let id_hash = util::generate_hash_id(&new_fm.title, &description, now, "default-workspace");
                                 let short_id = format!("bd-{}", id_hash);

                                 let issue = Issue {
                                    id: short_id.clone(),
                                    content_hash: String::new(),
                                    title: new_fm.title,
                                    description,
                                    design: String::new(),
                                    acceptance_criteria: String::new(),
                                    notes: String::new(),
                                    status: new_fm.status,
                                    priority: new_fm.priority,
                                    issue_type: new_fm.issue_type,
                                    assignee: new_fm.assignee,
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
                                return Ok(());
                             },
                             Err(e) => {
                                 anyhow::bail!("Invalid frontmatter: {}", e);
                             }
                         }
                    } else {
                         anyhow::bail!("Invalid format: missing frontmatter delimiters");
                    }
                } else {
                     anyhow::bail!("Invalid format: file must start with ---");
                }
            }

            // Fallback only happens if description was NOT empty initially, which is handled above.
            // If description IS empty, we returned or bailed above.
            // So this path is only for non-interactive creation.
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
    let mut current = match std::env::current_dir() {
        Ok(c) => c,
        Err(_) => return PathBuf::from(".beads/beads.db"),
    };

    loop {
        let p = current.join(".beads/beads.db");
        if p.exists() {
            return p;
        }
        if !current.pop() {
            break;
        }
    }
    // Default to .beads/beads.db in original CWD if not found (relative)
    // We can't easily get original CWD here since we popped `current`.
    // But `PathBuf::from` is relative to process CWD.
    PathBuf::from(".beads/beads.db")
}
