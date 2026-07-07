use crate::types::*;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub fn git_commit(state: State<'_, AppState>, path: String) -> Result<(), String> {
    let mut git = state.git.blocking_write();
    git.commit_all(&format!("Update {}", path))
        .map(|_| ())
        .map_err(|e| format!("Git commit failed: {}", e))
}

#[tauri::command]
pub fn git_history(state: State<'_, AppState>, path: String) -> Result<Vec<CommitEntry>, String> {
    let git = state.git.blocking_read();
    git.get_history(&path, 100)
        .map_err(|e| format!("Git history failed: {}", e))
}

#[tauri::command]
pub fn git_diff(
    state: State<'_, AppState>,
    path: String,
    rev1: String,
    rev2: String,
) -> Result<DiffResult, String> {
    let git = state.git.blocking_read();
    let diff_text = git
        .get_diff(&path, &rev1, &rev2)
        .map_err(|e| format!("Git diff failed: {}", e))?;

    let lines: Vec<&str> = diff_text.lines().collect();
    let mut hunks = Vec::new();
    let mut current_hunk_lines = Vec::new();

    for line in &lines {
        if let Some(_hdr) = line.strip_prefix("@@ ") {
            if !current_hunk_lines.is_empty() {
                hunks.push(DiffHunk {
                    old_start: 0,
                    old_lines: 0,
                    new_start: 0,
                    new_lines: 0,
                    lines: std::mem::take(&mut current_hunk_lines),
                });
            }
        }

        let origin = if line.starts_with("+") {
            '+'
        } else if line.starts_with("-") {
            '-'
        } else {
            ' '
        };

        current_hunk_lines.push(DiffLine {
            origin,
            content: line.to_string(),
        });
    }

    if !current_hunk_lines.is_empty() {
        hunks.push(DiffHunk {
            old_start: 0,
            old_lines: 0,
            new_start: 0,
            new_lines: 0,
            lines: current_hunk_lines,
        });
    }

    Ok(DiffResult {
        old_path: path.clone(),
        new_path: path,
        hunks,
    })
}

#[tauri::command]
pub fn git_status(state: State<'_, AppState>) -> Result<GitStatus, String> {
    let git = state.git.blocking_read();
    git.get_status()
        .map_err(|e| format!("Git status failed: {}", e))
}

#[tauri::command]
pub fn git_push(state: State<'_, AppState>) -> Result<(), String> {
    let mut git = state.git.blocking_write();
    git.push().map_err(|e| format!("Git push failed: {}", e))
}

#[tauri::command]
pub fn git_pull(state: State<'_, AppState>) -> Result<(), String> {
    let mut git = state.git.blocking_write();
    git.pull().map_err(|e| format!("Git pull failed: {}", e))
}

/// Alias for git_history — frontend HistoryBrowser calls git_log
#[tauri::command]
pub fn git_log(
    state: State<'_, AppState>,
    file_path: Option<String>,
) -> Result<Vec<CommitEntry>, String> {
    let git = state.git.blocking_read();
    let path = file_path.unwrap_or_default();
    git.get_history(&path, 100)
        .map_err(|e| format!("Git log failed: {}", e))
}

/// Restore a file to a specific commit version (creates a new commit)
#[tauri::command]
pub fn git_restore(
    state: State<'_, AppState>,
    file_path: String,
    commit: String,
) -> Result<(), String> {
    let git = state.git.blocking_read();
    let repo = git.repo().map_err(|e| format!("Git repo error: {}", e))?;

    let obj = repo
        .revparse_single(&commit)
        .map_err(|e| format!("Commit not found '{}': {}", commit, e))?;
    let tree_commit = obj
        .peel_to_commit()
        .map_err(|e| format!("Not a commit: {}", e))?;
    let tree = tree_commit
        .tree()
        .map_err(|e| format!("Failed to get tree: {}", e))?;

    // Try to find the file in the commit's tree
    let entry = tree
        .get_path(std::path::Path::new(&file_path))
        .map_err(|_| format!("File '{}' not found in commit '{}'", file_path, commit))?;

    let blob = repo
        .find_blob(entry.id())
        .map_err(|e| format!("Failed to read blob: {}", e))?;

    let content = std::str::from_utf8(blob.content())
        .map_err(|_| "File content is not valid UTF-8".to_string())?;

    // Write the restored content to the vault
    let vault_path = &state.vault_path;
    let full_path = vault_path.join(&file_path);
    if let Some(parent) = full_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directories: {}", e))?;
    }
    std::fs::write(&full_path, content).map_err(|e| format!("Failed to write file: {}", e))?;

    drop(git);

    // Auto-commit the restore
    let mut git_mut = state.git.blocking_write();
    git_mut
        .auto_commit(&full_path, content)
        .map(|_| ())
        .map_err(|e| format!("Failed to commit restore: {}", e))
}
