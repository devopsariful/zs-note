# 002-sync-spec.md
## ZarishNote Sync Specification
### Git auto-commit, history browsing, and cross-device sync

**Document type:** Specification — V1
**Date:** June 08, 2026
**Author:** Mohammad Ariful Islam / ZarishSphere Foundation
**License:** CC BY 4.0
**Status:** V1 — Authoritative

---

## Table of Contents

1. [Auto-Commit Engine](#1-auto-commit-engine)
2. [Commit Strategies](#2-commit-strategies)
3. [History Browser](#3-history-browser)
4. [Remote Sync](#4-remote-sync)
5. [Conflict Resolution](#5-conflict-resolution)
6. [Git Integration Details](#6-git-integration-details)
7. [Mobile Sync (Phase 2)](#7-mobile-sync-phase-2)

---

## 1. Auto-Commit Engine

Every file save triggers a Git commit. This is the "Doc as Code" principle — a full version history of every edit, stored as commits.

### 1.1 Trigger Behavior

- Save event (every keystroke with auto-save) → Git `add` + `commit`
- Debounced: 2 seconds after last keystroke
- Batch: if multiple files changed within debounce window, single commit for all
- Commit happens on a background thread — no UI blocking

### 1.2 Initialization

On vault creation:
```bash
git init
git config user.name "ZarishNote"
git config user.email "zarishnote@local"
git add .znrc
git commit -m "chore: initialize ZarishNote vault"
```

On vault open (existing folder):
- Check for `.git` directory
- If no `.git`: offer to initialize Git (default: yes)
- If Git exists but no commits: create initial commit

### 1.3 Ignored Patterns

Files matching these patterns are never tracked:

```yaml
sync:
  ignore_patterns:
    - ".znrc-vectors/**"
    - ".znrc-plugins/**"
    - ".znrc-state/**"
    - ".znrc-history/**"
    - ".znrc-search/**"
    - ".znrc-audit.log"
    - "*.lock"
    - ".git/**"
```

---

## 2. Commit Strategies

### 2.1 Conventional (Default)

```
docs: update report.md
docs: add meeting-notes.md
docs: update report.md (+ clinical section)
```

### 2.2 Timestamp

```
Auto-commit 2026-06-08 14:32:05
Auto-commit 2026-06-08 14:34:12
```

### 2.3 Diff-Summary

```
Modified section 2 in report.md (+120 characters)
Created meeting-notes.md (450 characters)
```

### 2.4 Custom Prefix

```yaml
sync:
  prefix: "ZNOTE"
  # Generates: "ZNOTE: update report.md"
```

---

## 3. History Browser

### 3.1 Access

Sidebar → click file → "History" tab at bottom of file tree.

### 3.2 UI

```
┌──────────────────────────────────────────────────────┐
│  📜 report.md — Commit History                       │
│                                                      │
│  2026-06-08 14:32                                    │
│  docs: update report.md (+120 chars)                 │
│  [Restore] [View Diff]                               │
│                                                      │
│  2026-06-08 12:15                                    │
│  docs: update report.md (+450 chars)                 │
│  [Restore] [View Diff]                               │
│                                                      │
│  2026-06-08 09:00                                    │
│  docs: add report.md                                 │
│  [Restore] [View Diff]                               │
│                                                      │
│  [Show all vault changes]                            │
└──────────────────────────────────────────────────────┘
```

### 3.3 Diff View

- Side-by-side: old file (left) vs new file (right)
- Line-level highlighting: green (added), red (removed)
- Word-level diff within lines
- "Revert" button for individual changes
- "Restore to this version" for full file revert

### 3.4 All-Vault History

From History tab root: shows commits across all files in reverse chronological order. Click commit → shows files changed in that commit.

---

## 4. Remote Sync

### 4.1 Configuration

```yaml
sync:
  remote:
    enabled: true
    url: "git@github.com:user/my-vault.git"
    branch: "main"
    push_on_commit: true
    pull_on_open: true
    conflict_strategy: "merge"
    ssh_key_id: "github-ssh-key"        # OS Keychain reference
```

### 4.2 Push Flow

- After each auto-commit: `git push origin main`
- Push runs in background
- Failure (network down): queue for retry
- Retry on next save, or manual "Sync Now" button
- Push status indicator in status bar

### 4.3 Pull Flow

- On vault open: `git pull origin main` (if remote configured)
- Merge conflicts handled according to strategy (see §5)
- Pull status notification with change summary

---

## 5. Conflict Resolution

| Strategy | Behavior |
|---|---|
| `merge` (default) | Standard Git merge. Creates merge commit. Mark conflicts for manual resolution. |
| `ours` | Discard remote changes. Local version wins. |
| `theirs` | Discard local changes. Remote version wins. |
| `manual` | Never auto-merge. Show diff, user picks. |

### 5.1 Conflict UI (V1)

When merge produces conflicts:
1. Markdown files: conflict markers inserted (`<<<<<<<`, `=======`, `>>>>>>>`)
2. Conflict banner appears at top of editor
3. "Resolve" button opens side-by-side compare
4. User edits to resolve, saves → `git add` + `git commit`
5. Non-Markdown files: use `theirs` strategy silently

---

## 6. Git Integration Details

### 6.1 Library

Using `git2-rs` (libgit2 bindings) for all Git operations. No shell-out to `git` CLI except for SSH operations.

### 6.2 SSH Key Management

- SSH keys stored in OS Keychain
- On first push: key is written to a temporary SSH agent session
- Session key is scoped to ZarishNote's process only

### 6.3 Large Files

Files over 10MB are excluded from auto-commit. User must manually add via Git CLI or use Git LFS (Phase 2).

### 6.4 Git Hooks

ZarishNote does not install Git hooks. Users can add their own.

---

## 7. Mobile Sync (Phase 2)

On mobile (Tauri v2 iOS/Android):

- Same Git engine via `git2-rs` (compiled for arm64)
- Push/pull triggers on device wake or app foreground
- SSH key stored in mobile keychain
- Offline queue: commits queued locally, pushed when online

---

*ZarishSphere Foundation · V1 · June 2026*
*License: CC BY 4.0*
