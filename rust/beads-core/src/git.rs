use std::process::Command;
use std::path::{Path, PathBuf};
use anyhow::{Result, Context, bail};

pub struct Git {
    root: PathBuf,
}

impl Git {
    pub fn new<P: AsRef<Path>>(root: P) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
        }
    }

    fn command(&self, args: &[&str]) -> Command {
        let mut cmd = Command::new("git");
        cmd.current_dir(&self.root);
        cmd.args(args);
        cmd
    }

    pub fn init(&self) -> Result<()> {
        let output = self.command(&["init"])
            .output()
            .context("Failed to run git init")?;

        if !output.status.success() {
             bail!("git init failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        Ok(())
    }

    pub fn add<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path_s = path.as_ref().to_string_lossy();
        let output = self.command(&["add", &path_s])
            .output()
            .context("Failed to run git add")?;

        if !output.status.success() {
             bail!("git add failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        Ok(())
    }

    pub fn commit(&self, message: &str) -> Result<()> {
        let output = self.command(&["commit", "-m", message])
            .output()
            .context("Failed to run git commit")?;

        if !output.status.success() {
            // Check if "nothing to commit" or "clean"
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("nothing to commit") || stdout.contains("working tree clean") {
                return Ok(());
            }
            bail!("git commit failed: {}\n{}", stdout, String::from_utf8_lossy(&output.stderr));
        }
        Ok(())
    }

    pub fn pull_rebase(&self) -> Result<()> {
        // pull --rebase
        let output = self.command(&["pull", "--rebase"])
            .output()
            .context("Failed to run git pull --rebase")?;

        if !output.status.success() {
             // Return error, caller checks for conflict
             bail!("git pull --rebase failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        Ok(())
    }

    pub fn push(&self) -> Result<()> {
        let output = self.command(&["push"])
            .output()
            .context("Failed to run git push")?;

        if !output.status.success() {
             bail!("git push failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        Ok(())
    }

    pub fn status(&self) -> Result<String> {
        let output = self.command(&["status", "--porcelain"])
            .output()
            .context("Failed to run git status")?;

        if !output.status.success() {
             bail!("git status failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        Ok(String::from_utf8(output.stdout)?)
    }

    pub fn show(&self, revision: &str) -> Result<String> {
        let output = self.command(&["show", revision])
            .output()
            .context("Failed to run git show")?;

        if !output.status.success() {
             bail!("git show failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        Ok(String::from_utf8(output.stdout)?)
    }

    pub fn rebase_continue(&self) -> Result<()> {
        let output = self.command(&["rebase", "--continue"])
            .env("GIT_EDITOR", "true") // avoid editor opening
            .output()
            .context("Failed to run git rebase --continue")?;

        if !output.status.success() {
             bail!("git rebase --continue failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        Ok(())
    }

    pub fn has_remote(&self) -> Result<bool> {
        let output = self.command(&["remote"])
            .output()
            .context("Failed to run git remote")?;

        if !output.status.success() {
             return Ok(false);
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(!stdout.trim().is_empty())
    }
}
