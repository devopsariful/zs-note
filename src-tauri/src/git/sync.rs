use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{bail, Result};
use git2::{Cred, FetchOptions, PushOptions, RemoteCallbacks};
use tokio::sync::Mutex;
use tracing::info;

use crate::git::GitEngine;

pub struct RemoteSyncManager {
    engine: Arc<Mutex<GitEngine>>,
    remote_url: String,
    ssh_key_path: Option<String>,
    auto_sync: bool,
    sync_interval: u64,
}

impl RemoteSyncManager {
    pub fn new(
        engine: Arc<Mutex<GitEngine>>,
        remote_url: &str,
        ssh_key_path: Option<String>,
        auto_sync: bool,
        sync_interval: u64,
    ) -> Self {
        Self {
            engine,
            remote_url: remote_url.to_string(),
            ssh_key_path,
            auto_sync,
            sync_interval,
        }
    }

    fn build_callbacks(&self) -> RemoteCallbacks<'static> {
        let ssh_key = self.ssh_key_path.clone();
        let mut callbacks = RemoteCallbacks::new();

        callbacks.credentials(move |_url, username_from_url, _allowed_types| {
            if let Some(ref key_path) = ssh_key {
                Cred::ssh_key(
                    username_from_url.unwrap_or("git"),
                    None,
                    Path::new(key_path),
                    None,
                )
            } else {
                Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
            }
        });

        callbacks.push_update_reference(|refname, status| {
            if let Some(msg) = status {
                tracing::warn!("Push rejected for {}: {}", refname, msg);
            }
            Ok(())
        });

        callbacks.transfer_progress(|progress| {
            if progress.received_objects() % 100 == 0 {
                tracing::debug!(
                    "Transfer progress: {}/{} objects, {}/{} bytes",
                    progress.received_objects(),
                    progress.total_objects(),
                    progress.received_bytes(),
                    progress.total_objects()
                );
            }
            true
        });

        callbacks
    }

    pub async fn push(&self) -> Result<()> {
        let engine = self.engine.lock().await;
        let repo = engine.repo()?;
        let mut remote = repo.find_remote("origin")?;

        let callbacks = self.build_callbacks();
        let mut push_opts = PushOptions::new();
        push_opts.remote_callbacks(callbacks);

        remote.push(&["refs/heads/main:refs/heads/main"], Some(&mut push_opts))?;

        info!("Successfully pushed to {}", self.remote_url);
        Ok(())
    }

    pub async fn pull(&self) -> Result<()> {
        let engine = self.engine.lock().await;
        let repo = engine.repo()?;

        if repo.find_remote("origin").is_err() {
            repo.remote("origin", &self.remote_url)?;
        }

        let mut remote = repo.find_remote("origin")?;
        let callbacks = self.build_callbacks();

        let mut fetch_opts = FetchOptions::new();
        fetch_opts.remote_callbacks(callbacks);

        remote.fetch(&["main"], Some(&mut fetch_opts), None)?;

        let fetch_head = repo.find_reference("FETCH_HEAD")?;
        let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)?;
        let analysis = repo.merge_analysis(&[&fetch_commit])?;

        if analysis.0.is_up_to_date() {
            info!("Already up to date");
            return Ok(());
        }

        if analysis.0.is_fast_forward() {
            let refname = "refs/heads/main";
            let mut reference = repo.find_reference(refname)?;
            reference.set_target(fetch_commit.id(), "Fast-forward merge")?;
            repo.set_head(refname)?;
            repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
            info!("Fast-forwarded to remote");
        } else if analysis.0.is_normal() {
            let refname = "refs/heads/main";
            let mut reference = repo.find_reference(refname)?;
            reference.set_target(fetch_commit.id(), "Merge")?;
            repo.set_head(refname)?;
            repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
            info!("Merged with remote");
        } else {
            bail!("Merge conflict detected — manual resolution required");
        }

        Ok(())
    }

    pub async fn start_background_sync(&self) {
        if !self.auto_sync {
            return;
        }

        let interval = self.sync_interval;
        let engine = self.engine.clone();
        let remote_url = self.remote_url.clone();
        let ssh_key = self.ssh_key_path.clone();

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(interval)).await;

                let sync_manager = RemoteSyncManager::new(
                    engine.clone(),
                    &remote_url,
                    ssh_key.clone(),
                    false,
                    interval,
                );

                match sync_manager.push().await {
                    Ok(()) => info!("Background sync: push successful"),
                    Err(e) => tracing::warn!("Background sync: push failed: {}", e),
                }

                match sync_manager.pull().await {
                    Ok(()) => info!("Background sync: pull successful"),
                    Err(e) => tracing::warn!("Background sync: pull failed: {}", e),
                }
            }
        });
    }

    pub async fn set_remote(&self, url: &str) -> Result<()> {
        let engine = self.engine.lock().await;
        let repo = engine.repo()?;

        if repo.find_remote("origin").is_ok() {
            repo.remote_set_url("origin", url)?;
        } else {
            repo.remote("origin", url)?;
        }

        info!("Remote set to {}", url);
        Ok(())
    }
}

pub async fn push_with_retry(sync: &RemoteSyncManager, max_retries: u32) -> Result<()> {
    let mut last_error = None;

    for attempt in 1..=max_retries {
        match sync.push().await {
            Ok(()) => return Ok(()),
            Err(e) => {
                tracing::warn!("Push attempt {}/{} failed: {}", attempt, max_retries, e);
                last_error = Some(e);
                tokio::time::sleep(Duration::from_secs(2u64.pow(attempt))).await;
            }
        }
    }

    Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Push failed after {} retries", max_retries)))
}
