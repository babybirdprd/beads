use crate::{Store, Git};
use crate::merge::merge3way;
use anyhow::{Result, Context};
use std::path::Path;
use std::fs;

pub fn run_sync(store: &Store, git_root: &Path, jsonl_path: &Path) -> Result<()> {
    let git = Git::new(git_root);

    // 1. Export
    store.export_to_jsonl(jsonl_path).context("Export failed")?;

    // 2. Git Add
    git.add(jsonl_path).context("Git add failed")?;

    // 3. Git Commit
    // Check if changes to commit? git commit will fail/no-op if clean.
    // Our Git wrapper returns Ok if clean.
    git.commit("sync: update issues").context("Git commit failed")?;

    // 4. Pull Rebase
    if let Err(e) = git.pull_rebase() {
        // Check for conflict
        let status = git.status()?;
        // Check if issues.jsonl is in conflict (UU = both modified)
        if status.contains("UU") && status.contains("issues.jsonl") {
            println!("Conflict detected on issues.jsonl. Attempting merge...");

            // Extract versions
            // Assuming jsonl_path is relative to git root or we know the relative path
            // For now, hardcode .beads/issues.jsonl which is the standard
            let rel_path = ".beads/issues.jsonl";

            let base_content = git.show(&format!(":1:{}", rel_path))?;
            let left_content = git.show(&format!(":2:{}", rel_path))?;
            let right_content = git.show(&format!(":3:{}", rel_path))?;

            // Write to temp files
            let temp_dir = std::env::temp_dir();
            let base_path = temp_dir.join("base.jsonl");
            let left_path = temp_dir.join("left.jsonl");
            let right_path = temp_dir.join("right.jsonl");

            fs::write(&base_path, base_content)?;
            fs::write(&left_path, left_content)?;
            fs::write(&right_path, right_content)?;

            // Run merge
            // Output directly to jsonl_path (overwriting conflict markers)
            merge3way(
                jsonl_path.to_str().unwrap(),
                base_path.to_str().unwrap(),
                left_path.to_str().unwrap(),
                right_path.to_str().unwrap(),
                false
            )?;

            // Add and continue
            git.add(jsonl_path)?;
            git.rebase_continue()?;

            println!("Merge resolved.");
        } else {
            // Re-throw original error if not our specific conflict
            return Err(e);
        }
    }

    // 5. Push
    git.push().context("Git push failed")?;

    Ok(())
}
