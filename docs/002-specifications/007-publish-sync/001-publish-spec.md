# 001-publish-spec.md
## ZarishNote Publishing & Sync Specification
### GitHub publishing, custom APIs, RSS, image hosting, and Git sync

**Document type:** Specification — V1
**Date:** June 08, 2026
**Author:** Mohammad Ariful Islam / ZarishSphere Foundation
**License:** CC BY 4.0
**Status:** V1 — Authoritative

---

## Table of Contents

1. [Publishing Overview](#1-publishing-overview)
2. [GitHub Publishing](#2-github-publishing)
3. [Custom API Publishing](#3-custom-api-publishing)
4. [RSS Generation](#4-rss-generation)
5. [Image Hosting](#5-image-hosting)
6. [Git Auto-Commit (Doc as Code)](#6-git-auto-commit-doc-as-code)
7. [Cross-Device Sync (Phase 2)](#7-cross-device-sync-phase-2)
8. [GUI: Publish Panel](#8-gui-publish-panel)

---

## 1. Publishing Overview

ZarishNote supports publishing Markdown documents without leaving the editor. All publishing is done from the **Publish panel** or via `Cmd/Ctrl + Shift + P`.

Publishing targets are configured in `.znrc` under `publish.targets[]`. Multiple targets per vault are supported.

---

## 2. GitHub Publishing

### 2.1 GitHub Pages / Repo Direct Push

Publishes one or more Markdown files directly to a GitHub repository.

```yaml
publish:
  targets:
    - name: "my-blog"
      type: "github"
      repo: "YOUR_USERNAME/my-blog"       # user/repo
      branch: "main"
      path: "/"                         # path within repo
      key_id: "github-token"            # OS Keychain reference
      build_command: null               # optional: runs before push
      transform:
        add_frontmatter: true           # ensure title, date present
        convert_wikilinks: true         # [[note]] → [note](./note.md)
        strip_private_blocks: true      # remove blocks tagged <!-- private -->
```

### 2.2 Publishing Flow

1. User clicks "Publish" in Publish panel
2. Select target from dropdown
3. (Optional) select which files to publish (default: current file)
4. ZarishNote runs `transform` pipeline on selected files
5. Creates/updates files in GitHub repo via GitHub API
6. Commits with message: `feat(publish): update [filename] via ZarishNote`
7. If `build_command` set: triggers GitHub Actions workflow
8. Success notification with link to GitHub

### 2.3 GitHub Token Setup

- Settings → Publish → Add Target → GitHub
- "Generate Token" button opens GitHub token creation page with required scopes pre-filled
- Token stored in OS Keychain after user pastes it

Required token scopes: `repo` (for private repos) or `public_repo` (for public).

---

## 3. Custom API Publishing

Publish to any REST API (Headless CMS, internal API, Notion-like backend):

```yaml
publish:
  targets:
    - name: "my-cms"
      type: "api"
      endpoint: "https://cms.example.com/api/posts"
      method: "POST"                    # POST | PUT | PATCH
      key_id: "cms-api-key"
      headers:
        Content-Type: "application/json"
        X-Source: "zarishnote"
      format: "json"                    # json | markdown | multipart
      body_template: |
        {
          "title": "{{title}}",
          "content": "{{markdown}}",
          "status": "draft",
          "tags": {{tags}}
        }
```

Template variables:
| Variable | Value |
|---|---|
| `{{title}}` | Front matter `title` or first H1 |
| `{{markdown}}` | Full document Markdown |
| `{{html}}` | Document as HTML |
| `{{tags}}` | Front matter `tags` as JSON array |
| `{{date}}` | Front matter `date` or today |
| `{{author}}` | Front matter `author` |
| `{{slug}}` | URL-safe filename |

---

## 4. RSS Generation

When a GitHub target has `rss.enabled: true`, ZarishNote maintains an RSS feed:

```yaml
publish:
  targets:
    - name: "my-blog"
      type: "github"
      rss:
        enabled: true
        feed_file: "feed.xml"
        title: "Ariful's Notes"
        description: "Public health, humanitarian response, technology"
        author: "Mohammad Ariful Islam"
        base_url: "https://YOUR_USERNAME.github.io/my-blog"
        max_items: 20
        include_full_content: true
```

RSS feed is generated at publish time:
- All published documents sorted by `date` front matter
- Includes: title, date, description (first paragraph or `description` front matter), link, full content (optional)
- Saved as `feed.xml` in repo root
- Valid RSS 2.0 format

---

## 5. Image Hosting

When publishing, local images need a hosting URL. ZarishNote handles this automatically:

```yaml
publish:
  image_hosting:
    provider: "github"
    repo: "YOUR_USERNAME/assets"
    branch: "main"
    path: "images/"
    key_id: "github-token"
    cdn_base_url: "https://raw.githubusercontent.com/YOUR_USERNAME/assets/main/images"
```

### 5.1 Image Publishing Flow

1. Scan document for local image references
2. For each local image:
   - Check if already pushed to image hosting repo
   - If not: push to `images/` in assets repo
3. Rewrite Markdown image references from local paths to CDN URLs
4. Publish document with updated image URLs

Local: `![chart](assets/chart.png)`
Published: `![chart](https://raw.githubusercontent.com/YOUR_USERNAME/assets/main/images/chart.png)`

### 5.2 Alternative Image Providers

| Provider | Notes |
|---|---|
| `github` | Free, CDN via jsDelivr or raw.githubusercontent.com |
| `cloudflare` | R2 bucket (free tier: 10GB storage, 1M requests/month) |
| `local` | Keep local paths (for local-only publishing) |
| `none` | Skip image upload, keep as-is |

---

## 6. Git Auto-Commit (Doc as Code)

ZarishNote treats every save as a Git commit. This is the "Doc as Code" principle.

### 6.1 Auto-Commit Behavior

- Every file save → Git `add` + `commit` (debounced: 2s after last keystroke)
- Commit message auto-generated: `docs: update [filename]` or custom strategy
- Message strategies:

| Strategy | Example commit message |
|---|---|
| `conventional` | `docs: update report.md` |
| `timestamp` | `Auto-commit 2026-06-08 14:32:05` |
| `diff-summary` | `Modified section 2 in report.md (+120 chars)` |

### 6.2 Git Configuration

```yaml
sync:
  auto_commit: true
  commit_message_style: "conventional"
  commit_author: "ZarishNote <zarishnote@local>"
  ignore_patterns:
    - ".znrc-vectors/**"
    - ".znrc-plugins/**"
    - ".znrc-state/**"
    - ".znrc-history/**"
```

### 6.3 History Browser

- Sidebar: click file → History tab shows commit list
- Each commit: timestamp, message, diff summary
- "Restore" button: reverts file to selected commit
- "Diff view": side-by-side comparison of any two commits

---

## 7. Cross-Device Sync (Phase 2)

Phase 2 adds push to a remote Git remote for cross-device sync:

```yaml
sync:
  remote:
    enabled: true
    url: "git@github.com:YOUR_USERNAME/my-vault.git"
    branch: "main"
    push_on_commit: true      # push after every auto-commit
    pull_on_open: true        # pull before opening vault
    conflict_strategy: "merge"  # merge | ours | theirs | manual
```

Sync options:
- **GitHub private repo** (free, unlimited storage for private) — recommended
- **Self-hosted Gitea** — for full local control
- **Any Git remote** — SSH or HTTPS

On mobile (iOS/Android), same sync works — Tauri v2 mobile can use the same Git engine.

---

## 8. GUI: Publish Panel

```
┌────────────────────────────────────────────────────────┐
│  Publish                                               │
│                                                        │
│  Current file: report.md                              │
│                                                        │
│  Targets:                                             │
│  ○ my-blog (GitHub: YOUR_USERNAME/my-blog)               │
│  ○ my-cms (API: cms.example.com)                      │
│                                                        │
│  Options:                                             │
│  ☑ Upload images                                      │
│  ☑ Convert wikilinks                                  │
│  ☑ Strip private blocks                               │
│  ☑ Generate RSS feed                                  │
│                                                        │
│  Preview (last publish: 2026-06-07 09:15)             │
│  [View diff since last publish]                       │
│                                                        │
│  [Publish now]          [Schedule...]                 │
└────────────────────────────────────────────────────────┘
```

---

*ZarishSphere Foundation · V1 · June 2026*
*License: CC BY 4.0*