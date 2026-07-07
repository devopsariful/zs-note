pub mod commit;
pub mod history;
pub mod sync;

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use chrono::{TimeZone, Utc};
use git2::{DiffOptions, Oid, Repository, Signature, StatusOptions};

use crate::types::{CommitEntry, GitStatus};

pub struct GitEngine {
    repo_path: PathBuf,
}

impl GitEngine {
    pub fn new(path: &Path) -> Self {
        Self {
            repo_path: path.to_path_buf(),
        }
    }

    pub fn repo(&self) -> Result<Repository> {
        Repository::open(&self.repo_path)
            .with_context(|| format!("Failed to open git repo at {:?}", self.repo_path))
    }

    pub fn init_vault(&self, path: &Path) -> Result<Repository> {
        let repo = Repository::init(path)?;
        let mut index = repo.index()?;
        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;
        let signature = Signature::now("ZarishNote", "zarishsphere@gmail.com")?;
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "Initial commit: ZarishNote vault initialized",
            &tree,
            &[],
        )?;
        tracing::info!("Initialized git vault at {:?}", path);
        drop(tree);
        Ok(repo)
    }

    pub fn auto_commit(&mut self, path: &Path, _content: &str) -> Result<Oid> {
        let repo = self.repo()?;
        let relative = path.strip_prefix(&self.repo_path).unwrap_or(path);
        let relative_str = relative.to_string_lossy();

        let mut index = repo.index()?;
        index.add_path(relative)?;
        index.write()?;
        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;

        let parent = repo.head().ok().and_then(|h| h.peel_to_commit().ok());
        let parents: Vec<&git2::Commit> = parent.iter().collect();

        let signature = Signature::now("ZarishNote", "zarishsphere@gmail.com")?;
        let message = format!("auto: update {}", relative_str);

        let oid = repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            &message,
            &tree,
            &parents,
        )?;

        tracing::debug!("Auto-committed {}: {}", relative_str, oid);
        Ok(oid)
    }

    pub fn commit_all(&mut self, message: &str) -> Result<Oid> {
        let repo = self.repo()?;
        let mut index = repo.index()?;

        let statuses = repo.statuses(Some(
            StatusOptions::default()
                .include_untracked(true)
                .recurse_untracked_dirs(true),
        ))?;

        for entry in statuses.iter() {
            if let Some(path) = entry.path() {
                let _ = index.add_path(Path::new(path));
            }
        }

        index.write()?;
        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;

        let parent = repo.head().ok().and_then(|h| h.peel_to_commit().ok());
        let parents: Vec<&git2::Commit> = parent.iter().collect();

        let signature = Signature::now("ZarishNote", "zarishsphere@gmail.com")?;

        let oid = repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &parents,
        )?;
        tracing::info!("Committed: {}", message);
        Ok(oid)
    }

    pub fn get_history(&self, path: &str, max_count: usize) -> Result<Vec<CommitEntry>> {
        let repo = self.repo()?;
        let mut revwalk = repo.revwalk()?;
        revwalk.push_head()?;
        revwalk.set_sorting(git2::Sort::TIME)?;

        let mut entries = Vec::new();

        for oid in revwalk.take(max_count) {
            let oid = oid?;
            let commit = repo.find_commit(oid)?;

            if !path.is_empty() {
                let tree = commit.tree()?;
                let mut diff_opts = DiffOptions::new();
                diff_opts.pathspec(path);
                let parent_tree = commit.parent(0).ok().and_then(|p| p.tree().ok());
                let diff = repo.diff_tree_to_tree(
                    parent_tree.as_ref(),
                    Some(&tree),
                    Some(&mut diff_opts),
                )?;

                if diff.deltas().len() == 0 {
                    continue;
                }
            }

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

    pub fn get_diff(&self, path: &str, from: &str, to: &str) -> Result<String> {
        let repo = self.repo()?;
        let from_commit = repo.revparse_single(from)?.peel_to_commit()?;
        let to_commit = repo.revparse_single(to)?.peel_to_commit()?;

        let from_tree = from_commit.tree()?;
        let to_tree = to_commit.tree()?;

        let mut diff_opts = DiffOptions::new();
        diff_opts.pathspec(path);

        let diff =
            repo.diff_tree_to_tree(Some(&from_tree), Some(&to_tree), Some(&mut diff_opts))?;

        let mut output = String::new();
        diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
            let origin = match line.origin() {
                '+' => '+',
                '-' => '-',
                _ => ' ',
            };
            let content = std::str::from_utf8(line.content()).unwrap_or("");
            output.push(origin);
            output.push_str(content);
            true
        })?;

        Ok(output)
    }

    pub fn get_status(&self) -> Result<GitStatus> {
        let repo = self.repo()?;

        let statuses = repo.statuses(Some(
            StatusOptions::default()
                .include_untracked(true)
                .recurse_untracked_dirs(true),
        ))?;

        let mut unstaged = Vec::new();
        for entry in statuses.iter() {
            if let Some(path) = entry.path() {
                unstaged.push(path.to_string());
            }
        }

        let ahead = 0;
        let behind = 0;

        if let Ok(head) = repo.head() {
            let head_oid = head.peel_to_commit().map(|c| c.id()).unwrap_or(Oid::zero());
            if let Some(branch_name) = head.shorthand() {
                if let Ok(branch) = repo.find_branch(branch_name, git2::BranchType::Local) {
                    if let Ok(upstream) = branch.upstream() {
                        let upstream_oid = upstream
                            .get()
                            .peel_to_commit()
                            .map(|c| c.id())
                            .unwrap_or(Oid::zero());

                        let mut revwalk = repo.revwalk()?;
                        revwalk.push(upstream_oid)?;
                        revwalk.hide(head_oid)?;
                        let behind_count = revwalk.count();

                        let mut revwalk = repo.revwalk()?;
                        revwalk.push(head_oid)?;
                        revwalk.hide(upstream_oid)?;
                        let ahead_count = revwalk.count();

                        return Ok(GitStatus {
                            ahead: ahead_count,
                            behind: behind_count,
                            unstaged,
                        });
                    }
                }
            }
        }

        Ok(GitStatus {
            ahead,
            behind,
            unstaged,
        })
    }

    pub fn push(&mut self) -> Result<()> {
        let repo = self.repo()?;
        let mut remote = repo.find_remote("origin")?;

        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            git2::Cred::ssh_key(
                username_from_url.unwrap_or("git"),
                None,
                std::path::Path::new(&format!(
                    "{}/.ssh/id_ed25519",
                    std::env::var("HOME").unwrap_or_else(|_| "/root".to_string())
                )),
                None,
            )
        });
        callbacks.push_update_reference(|refname, status| {
            if let Some(msg) = status {
                Err(git2::Error::new(
                    git2::ErrorCode::GenericError,
                    git2::ErrorClass::Net,
                    format!("Push failed for {}: {}", refname, msg),
                ))
            } else {
                Ok(())
            }
        });

        let mut push_opts = git2::PushOptions::new();
        push_opts.remote_callbacks(callbacks);

        remote.push(&["refs/heads/main:refs/heads/main"], Some(&mut push_opts))?;

        tracing::info!("Pushed to remote");
        Ok(())
    }

    pub fn pull(&mut self) -> Result<()> {
        let repo = self.repo()?;
        let mut remote = repo.find_remote("origin")?;

        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            git2::Cred::ssh_key(
                username_from_url.unwrap_or("git"),
                None,
                std::path::Path::new(&format!(
                    "{}/.ssh/id_ed25519",
                    std::env::var("HOME").unwrap_or_else(|_| "/root".to_string())
                )),
                None,
            )
        });

        let mut fetch_opts = git2::FetchOptions::new();
        fetch_opts.remote_callbacks(callbacks);

        remote.fetch(&["main"], Some(&mut fetch_opts), None)?;

        let fetch_head = repo.find_reference("FETCH_HEAD")?;
        let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)?;
        let analysis = repo.merge_analysis(&[&fetch_commit])?;

        if analysis.0.is_up_to_date() {
            return Ok(());
        }

        if analysis.0.is_fast_forward() {
            let refname = "refs/heads/main";
            let mut reference = repo.find_reference(refname)?;
            reference.set_target(fetch_commit.id(), "Fast-forward merge")?;
            repo.set_head(refname)?;
            repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
        } else {
            bail!("Pull requires merge resolution — not yet supported");
        }

        tracing::info!("Pulled from remote");
        Ok(())
    }
}
