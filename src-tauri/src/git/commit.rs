use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use git2::Oid;
use tokio::sync::Mutex;
use tracing::info;

use crate::git::GitEngine;

pub struct AutoCommitManager {
    engine: Arc<Mutex<GitEngine>>,
    pending: Arc<Mutex<HashMap<PathBuf, (String, Instant)>>>,
    debounce_ms: u64,
    running: Arc<AtomicBool>,
}

impl AutoCommitManager {
    pub fn new(engine: Arc<Mutex<GitEngine>>, debounce_ms: u64) -> Self {
        Self {
            engine,
            pending: Arc::new(Mutex::new(HashMap::new())),
            debounce_ms,
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub async fn schedule(&self, path: PathBuf, content: String) {
        let mut pending = self.pending.lock().await;
        pending.insert(path, (content, Instant::now()));
    }

    pub async fn flush(&self) -> Result<Vec<Oid>> {
        let mut pending = self.pending.lock().await;
        let mut oids = Vec::new();

        if pending.is_empty() {
            return Ok(oids);
        }

        let files: Vec<(PathBuf, String)> = pending
            .drain()
            .map(|(path, (content, _))| (path, content))
            .collect();

        let mut engine = self.engine.lock().await;

        for (path, _content) in &files {
            let oid = engine.auto_commit(path, "")?;
            oids.push(oid);
        }

        if files.len() > 1 {
            info!("Auto-committed {} files in batch", files.len());
        }

        Ok(oids)
    }

    pub async fn debounced_schedule(&self, path: PathBuf, content: String) {
        self.schedule(path, content).await;

        if self.running.load(Ordering::SeqCst) {
            return;
        }

        self.running.store(true, Ordering::SeqCst);
        tokio::spawn({
            let pending = self.pending.clone();
            let engine = self.engine.clone();
            let debounce_ms = self.debounce_ms;
            let running = self.running.clone();

            async move {
                loop {
                    tokio::time::sleep(Duration::from_millis(debounce_ms)).await;

                    let should_flush = {
                        let p = pending.lock().await;
                        p.is_empty()
                            || p.values()
                                .all(|(_, ts)| ts.elapsed() >= Duration::from_millis(debounce_ms))
                    };

                    if should_flush {
                        let mut p = pending.lock().await;
                        let files: Vec<(PathBuf, String)> =
                            p.drain().map(|(k, (c, _))| (k, c)).collect();
                        drop(p);

                        if !files.is_empty() {
                            let mut engine = engine.lock().await;
                            for (path, _content) in &files {
                                if let Err(e) = engine.auto_commit(path, "") {
                                    tracing::warn!(
                                        "Debounced auto-commit failed for {:?}: {}",
                                        path,
                                        e
                                    );
                                }
                            }
                            info!("Debounced auto-commit: {} files", files.len());
                        }

                        running.store(false, Ordering::SeqCst);
                        break;
                    }
                }
            }
        });
    }
}

pub fn generate_commit_message(paths: &[&Path], style: &str) -> String {
    match style {
        "conventional" => {
            let types = detect_change_types(paths);
            let scope = paths
                .first()
                .and_then(|p| p.parent())
                .and_then(|p| p.file_name())
                .and_then(|s| s.to_str())
                .unwrap_or("vault");

            let descriptions: Vec<String> = paths
                .iter()
                .filter_map(|p| p.file_stem())
                .filter_map(|s| s.to_str())
                .map(|s| s.to_string())
                .collect();

            format!(
                "{}({}): update {}",
                types.first().map(|s| s.as_str()).unwrap_or("chore"),
                scope,
                descriptions.join(", ")
            )
        }
        "simple" => {
            let names: Vec<String> = paths
                .iter()
                .filter_map(|p| p.file_name())
                .filter_map(|s| s.to_str())
                .map(|s| s.to_string())
                .collect();
            format!("Update {}", names.join(", "))
        }
        "descriptive" => {
            let now = chrono::Local::now();
            format!(
                "Auto-save from ZarishNote on {}",
                now.format("%Y-%m-%d %H:%M")
            )
        }
        _ => format!("Update {} files", paths.len()),
    }
}

fn detect_change_types(paths: &[&Path]) -> Vec<String> {
    let mut types = Vec::new();

    for path in paths {
        let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");

        if name == ".znrc" || name == "znrc.yml" || name == "znrc.yaml" {
            types.push("config".to_string());
        } else if ext == "md" || ext == "markdown" {
            types.push("docs".to_string());
        } else if ext == "rs" || ext == "ts" || ext == "js" || ext == "py" {
            types.push("feat".to_string());
        } else if ext == "json" || ext == "toml" || ext == "yaml" || ext == "yml" {
            types.push("chore".to_string());
        } else {
            types.push("chore".to_string());
        }
    }

    types.sort();
    types.dedup();
    types
}
