use std::path::Path;

use anyhow::{Context, Result};
use chrono::{TimeZone, Utc};
use git2::{DiffOptions, Sort};

use crate::git::GitEngine;
use crate::types::CommitEntry;

impl GitEngine {
    pub fn get_file_history(&self, path: &Path, max_count: usize) -> Result<Vec<CommitEntry>> {
        let repo = self.repo()?;
        let mut revwalk = repo.revwalk()?;
        revwalk.push_head()?;
        revwalk.set_sorting(Sort::TIME)?;

        let relative = path.strip_prefix(&self.repo_path).unwrap_or(path);

        let mut entries = Vec::new();

        for oid in revwalk.take(max_count) {
            let oid = oid?;
            let commit = repo.find_commit(oid)?;
            let tree = commit.tree()?;

            let parent_tree = commit.parent(0).ok().and_then(|p| p.tree().ok());

            let mut diff_opts = DiffOptions::new();
            diff_opts.pathspec(relative);

            let diff =
                repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), Some(&mut diff_opts))?;

            if diff.deltas().len() > 0 {
                let time = commit.time();
                let timestamp = Utc
                    .timestamp_opt(time.seconds(), 0)
                    .single()
                    .unwrap_or_default();

                entries.push(CommitEntry {
                    hash: oid.to_string(),
                    message: commit.message().unwrap_or("").to_string(),
                    author: commit.author().name().unwrap_or("unknown").to_string(),
                    timestamp,
                });
            }
        }

        Ok(entries)
    }

    pub fn get_vault_history(&self, max_count: usize) -> Result<Vec<CommitEntry>> {
        let repo = self.repo()?;
        let mut revwalk = repo.revwalk()?;
        revwalk.push_head()?;
        revwalk.set_sorting(Sort::TIME)?;

        let mut entries = Vec::new();

        for oid in revwalk.take(max_count) {
            let oid = oid?;
            let commit = repo.find_commit(oid)?;
            let time = commit.time();
            let timestamp = Utc
                .timestamp_opt(time.seconds(), 0)
                .single()
                .unwrap_or_default();

            entries.push(CommitEntry {
                hash: oid.to_string(),
                message: commit.message().unwrap_or("").to_string(),
                author: commit.author().name().unwrap_or("unknown").to_string(),
                timestamp,
            });
        }

        Ok(entries)
    }

    pub fn restore_file(&self, path: &Path, revision: &str) -> Result<()> {
        let repo = self.repo()?;
        let commit_obj = repo.revparse_single(revision)?;
        let commit = commit_obj.peel_to_commit()?;
        let tree = commit.tree()?;

        let relative = path.strip_prefix(&self.repo_path).unwrap_or(path);

        let entry = tree
            .get_path(relative)
            .with_context(|| format!("Path {:?} not found in revision {}", relative, revision))?;

        let blob = repo.find_blob(entry.id())?;
        let content = blob.content();

        std::fs::write(path, content)?;

        tracing::info!("Restored {:?} to revision {}", path, revision);
        Ok(())
    }

    pub fn get_revision_content(&self, path: &Path, revision: &str) -> Result<String> {
        let repo = self.repo()?;
        let commit_obj = repo.revparse_single(revision)?;
        let commit = commit_obj.peel_to_commit()?;
        let tree = commit.tree()?;

        let relative = path.strip_prefix(&self.repo_path).unwrap_or(path);

        let entry = tree
            .get_path(relative)
            .with_context(|| format!("Path {:?} not found in revision {}", relative, revision))?;

        let blob = repo.find_blob(entry.id())?;
        let content = String::from_utf8_lossy(blob.content()).to_string();

        Ok(content)
    }
}
