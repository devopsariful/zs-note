# ZarishNote — Complete Build & Use Guide
## For Mohammad Ariful Islam · Solo Non-Technical Builder
### Step-by-Step from Zero to Working Desktop App

**Document type:** Master Build Guide
**Date:** June 23, 2026
**Author:** Claude (Anthropic) for ZarishSphere Foundation
**License:** CC BY 4.0
**Status:** V1 — Authoritative. Follow every step in order.

---

## Table of Contents

1. [What This Guide Covers](#1-what-this-guide-covers)
2. [Understanding the Project](#2-understanding-the-project)
3. [One-Time Machine Setup](#3-one-time-machine-setup)
4. [Getting the Code onto GitHub](#4-getting-the-code-onto-github)
5. [Triggering a Release Build](#5-triggering-a-release-build)
6. [Installing the App on Your Laptop](#6-installing-the-app-on-your-laptop)
7. [Using ZarishNote Daily](#7-using-zarishnote-daily)
8. [Understanding What Was Fixed](#8-understanding-what-was-fixed)
9. [Future Maintenance](#9-future-maintenance)
10. [Troubleshooting](#10-troubleshooting)

---

## 1. What This Guide Covers

This guide tells you **exactly what to do, in what order, command by command**, to:

- Set up your Lenovo Ubuntu laptop as a development environment (one time only)
- Push the corrected ZarishNote code to GitHub
- Let GitHub's free build servers compile the app for you (so your 8 GB RAM laptop does NOT have to)
- Download and install the finished `.deb` app on your laptop
- Use ZarishNote as a daily note-taking tool

**You do not need to understand any code.** Every command is copy-paste ready. Every screen is described.

---

## 2. Understanding the Project

Before touching a terminal, here is what ZarishNote is and how its pieces fit together.

### 2.1 The three layers

| Layer | Folder | What it is | Language |
|---|---|---|---|
| Frontend (screen) | `src/` | What you see — the editor, sidebar, AI panel | Svelte / TypeScript |
| Backend (engine) | `src-tauri/` | Runs on your computer — file system, AI, Git, sandbox | Rust |
| Ingestion (converter) | `ingestion/` | Converts PDF, Word, YouTube → Markdown | Python |

The Rust backend and Svelte frontend are **compiled together by Tauri** into a single `.deb` installer for Ubuntu.

### 2.2 Why your laptop cannot compile

Rust compilation of a Tauri project with this many libraries requires 4–8 GB RAM at peak. Your laptop has 8 GB total, and Ubuntu itself needs ~2 GB. That leaves ~6 GB — not enough for a reliable build.

**Solution:** GitHub Actions (GitHub's free build servers) compile the app for you in the cloud. You only push code; GitHub does the heavy lifting.

### 2.3 The two GitHub accounts

| Account | Used for |
|---|---|
| `codeandbrain` | All normal work — code, commits |
| `arwazarish` | Only for pushing files inside `.github/workflows/` |

This split exists because GitHub free organizations have a quirk with workflow file permissions. You will follow this rule exactly as described in Section 4.

---

## 3. One-Time Machine Setup

Open a Terminal on your Ubuntu laptop. To open Terminal: press `Ctrl + Alt + T`.

Every block below that starts with `$` is a command. Copy it exactly (without the `$`) and press Enter.

### 3.1 Update your system

```bash
sudo apt update && sudo apt upgrade -y
```

When asked for your password, type it (nothing will appear on screen — that is normal) and press Enter.

### 3.2 Install system tools

```bash
sudo apt install -y curl wget git build-essential libssl-dev \
  libwebkit2gtk-4.1-dev libayatana-appindicator3-dev \
  librsvg2-dev libxdo-dev pkg-config
```

This installs everything Tauri needs. It takes 2–5 minutes.

### 3.3 Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

When it asks, press `1` then Enter to choose the default installation. When finished, run:

```bash
source $HOME/.cargo/env
```

Verify it worked:

```bash
rustc --version
```

You should see something like `rustc 1.96.0 (...)`.

### 3.4 Install Node.js via NVM

NVM lets you install any Node.js version without administrator problems.

```bash
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.7/install.sh | bash
```

Close and reopen your Terminal, then:

```bash
nvm install 24
nvm use 24
node --version
```

You should see `v24.17.0`.

### 3.5 Install pnpm

```bash
npm install -g pnpm@11.8.0
pnpm --version
```

You should see `11.8.0`.

### 3.6 Install the GitHub CLI (gh)

The `gh` tool lets you log into GitHub from the terminal.

```bash
sudo apt install -y gh
```

Or if that fails:

```bash
curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg | sudo dd of=/usr/share/keyrings/githubcli-archive-keyring.gpg
echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | sudo tee /etc/apt/sources.list.d/github-cli.list > /dev/null
sudo apt update && sudo apt install gh -y
```

### 3.7 Log into GitHub CLI with codeandbrain

```bash
gh auth login
```

Follow these answers:
- `What account do you want to log into?` → press Enter on **GitHub.com**
- `What is your preferred protocol?` → press Enter on **HTTPS**
- `How would you like to authenticate?` → press Enter on **Login with a web browser**
- A code appears. Copy it. Press Enter. Your browser opens. Paste the code. Authorize.

When finished, verify:

```bash
gh auth status
```

You should see `Logged in to github.com as codeandbrain`.

### 3.8 Set your Git identity

```bash
git config --global user.name "Mohammad Ariful Islam"
git config --global user.email "zarishsphere@gmail.com"
```

### 3.9 Install Python 3.12 and pip

Ubuntu 26.04 should have Python 3.12 already. Check:

```bash
python3 --version
```

If you see `Python 3.12.x`, you are set. Then install pip:

```bash
sudo apt install -y python3-pip python3-venv
```

---

## 4. Getting the Code onto GitHub

### 4.1 Unzip the corrected code

The file `zs-note-fixed.zip` that came with this guide is the corrected codebase. Open your File Manager, find that zip file, right-click it → **Extract Here**. You will get a folder called `zs-note`.

Move it to your home directory:

```bash
mv ~/Downloads/zs-note ~/zs-note
cd ~/zs-note
```

### 4.2 Install frontend packages (local check only)

```bash
pnpm install --no-frozen-lockfile
```

Wait for it to finish. This downloads JavaScript packages.

### 4.3 Check the code compiles (TypeScript only — fast, no Rust needed)

```bash
pnpm typecheck
```

If it finishes with no error lines starting with `error TS`, you are ready. The `[WARN] Unsupported engine` warning is harmless — ignore it.

### 4.4 Initialise the local Git repository

```bash
cd ~/zs-note
git init
git add .
git commit -m "fix: all IPC wirings, TypeScript errors, sandbox/git/mcp commands"
```

### 4.5 Connect to your GitHub repository

```bash
git remote add origin https://github.com/zarishsphere/zs-note.git
```

If the repo already has content on GitHub, pull first:

```bash
git pull origin main --allow-unrelated-histories
```

If you get merge conflicts, do not worry — run:

```bash
git checkout --ours .
git add .
git commit -m "merge: keep fixed local version"
```

### 4.6 Push code as codeandbrain (normal code — NOT workflow files)

Push everything except the `.github/workflows/` folder:

```bash
git push -u origin main
```

If it asks for credentials, use your GitHub Personal Access Token (PAT). To create one:

1. Go to `https://github.com/settings/tokens`
2. Click **Generate new token (classic)**
3. Tick: `repo`, `workflow`
4. Copy the token
5. Paste it when the terminal asks for a password

### 4.7 Push workflow files as arwazarish

The `.github/workflows/` files must be pushed by the `arwazarish` account due to GitHub org workflow permissions.

First, log in as arwazarish:

```bash
gh auth login --hostname github.com
```

Follow the same browser flow, but log in as `arwazarish` this time.

Now switch the active account:

```bash
gh auth switch -u arwazarish
gh auth status
```

Confirm you see `Logged in to github.com as arwazarish`.

Create a separate branch just for the workflow push:

```bash
git checkout -b workflow-push
git push origin workflow-push
```

Go to `https://github.com/zarishsphere/zs-note` in your browser, find the branch `workflow-push`, and open a Pull Request into `main`. If `arwazarish` has write access to the org, it can merge it directly.

After merging, switch back to `codeandbrain`:

```bash
gh auth switch -u codeandbrain
git checkout main
git pull origin main
```

---

## 5. Triggering a Release Build

GitHub Actions will build the app automatically when you push a version tag.

### 5.1 Create a version tag

```bash
cd ~/zs-note
git tag v0.1.0
git push origin v0.1.0
```

### 5.2 Watch the build

1. Go to `https://github.com/zarishsphere/zs-note/actions` in your browser
2. You will see a workflow called **Build** running
3. Click on it to see the progress
4. It builds on Ubuntu, macOS, and Windows simultaneously
5. Wait approximately 20–35 minutes for all three to finish

### 5.3 Download the Ubuntu installer

When the build finishes:

1. Click the **Build** workflow run
2. Scroll down to **Artifacts**
3. Click `ZarishNote-x86_64-unknown-linux-gnu` to download a `.zip`
4. Inside that zip is a `bundle/deb/` folder with a `.deb` file

Or, if a GitHub Release was created automatically, go to:

`https://github.com/zarishsphere/zs-note/releases`

Download the `.deb` file directly.

---

## 6. Installing the App on Your Laptop

### 6.1 Install the .deb package

Open your File Manager, find the downloaded `.deb` file, right-click → **Open With Software Install**. Click **Install**.

Or from the terminal (replace `filename.deb` with the actual name):

```bash
sudo dpkg -i ~/Downloads/zs-note_0.1.0_amd64.deb
sudo apt-get install -f
```

### 6.2 Launch ZarishNote

Search for **ZarishNote** in your applications menu, or from the terminal:

```bash
zs-note
```

The app window opens. You are ready.

---

## 7. Using ZarishNote Daily

### 7.1 Opening your first vault

When ZarishNote opens for the first time:

1. It asks you to choose a **vault folder** — this is where all your notes live
2. Click the folder icon or **Open Vault**
3. Choose (or create) a folder, for example: `~/Documents/my-notes`
4. Click **Open**

ZarishNote creates a `.znrc` config file inside that folder automatically.

### 7.2 Creating and editing notes

| Action | How |
|---|---|
| New note | Click the `+` button in the sidebar, or press `Ctrl + N` |
| Save | Press `Ctrl + S` (saves automatically too) |
| Switch view modes | Click **WYSIWYG**, **Source**, or **Split** at the top |
| Bold text | Select text, press `Ctrl + B` |
| Italic text | Select text, press `Ctrl + I` |
| Insert heading | Type `#` then Space at the start of a line |
| Insert code block | Type three backticks ``` then Enter |
| Insert math | Type `$` then your formula then `$` |
| Insert diagram | Type ` ```mermaid ` then a diagram on the next lines |

### 7.3 Organising files

- **Create folder:** Right-click in the file tree → **New Folder**
- **Rename:** Right-click on any file → **Rename**
- **Move:** Drag a file to a different folder in the tree
- **Delete:** Right-click → **Delete** (creates a Git backup automatically)
- **Search:** Press `Ctrl + P` to open the file search

### 7.4 Using the AI assistant

> **Before using AI:** You need either Ollama (free, local) or an API key from a provider.

**Option A — Ollama (recommended, free, works offline):**

Install Ollama:

```bash
curl -fsSL https://ollama.com/install.sh | sh
ollama pull llama3.2
```

In ZarishNote → **Settings** → **AI** → set Provider to `ollama`, Model to `llama3.2`.

**Option B — Cloud API (requires internet):**

In ZarishNote → **Settings** → **API Keys** → enter your key for OpenAI, Anthropic (Claude), or Google Gemini.

**Using AI:**

1. Open the AI panel (right side, or press `Ctrl + Shift + A`)
2. Type your question or instruction
3. The AI responds in the chat panel
4. Click **Insert** to add the AI's text to your current note

**AI Templates:** Click the template icon (wand ✨) to use built-in prompts:
- Summarize, Explain, Rewrite, Translate, Continue Writing
- Extract Action Items, Pros & Cons, Blog Outline

### 7.5 Git version history

Every time you save, ZarishNote silently creates a Git commit — a snapshot of your note.

To browse history:

1. Click the **History** button (clock icon) in the toolbar
2. See a list of past versions with dates
3. Click any version to see what changed (green = added, red = removed)
4. Click **Restore** to bring back an older version (it creates a new commit, so nothing is lost)

### 7.6 Importing documents

Drag any of these file types into the ZarishNote window to convert them to Markdown automatically:

- `.pdf` (PDF documents)
- `.docx` (Word documents)
- `.pptx` (PowerPoint presentations)
- `.xlsx` (Excel spreadsheets)
- `.epub` (e-books)
- `.html` (web pages)
- `.csv` (data tables)

Or use **File → Import** from the menu.

For YouTube videos: paste the YouTube URL into the Import dialog to get a transcript in Markdown.

### 7.7 MCP tool servers (advanced)

MCP (Model Context Protocol) lets AI assistants use external tools — for example, reading from GitHub, running database queries, or calling APIs.

To add an MCP server:

1. Go to **Settings → MCP Servers**
2. Click **Add Server**
3. Fill in:
   - **Name:** e.g., `github`
   - **Transport:** `stdio` or `sse`
   - **Command:** the executable path, e.g., `/usr/local/bin/mcp-github`
4. Click **Save**

The server appears in the list. Toggle it on/off with the switch.

### 7.8 Publishing notes

Go to **Settings → Publish** to configure:

- **GitHub Pages:** push notes as a static website to a GitHub repo
- **Custom API:** POST notes to any endpoint
- **RSS feed:** auto-generate a feed of published notes

Once configured, open any note and click **Publish → Publish Now**.

---

## 8. Understanding What Was Fixed

This section explains what was wrong in the original codebase and what was corrected, so you can brief any future AI assistant or developer accurately.

### 8.1 Two TypeScript compile errors (now fixed)

| Error | File | Problem | Fix |
|---|---|---|---|
| `Cannot find name 'toVaultRelativePath'` | `src/lib/commands/editor.ts` | Function was used but never imported | Added `import { toVaultRelativePath } from '../utils/vaultPath'` |
| `Type 'string' is not assignable to type 'ProviderType'` | `src/lib/stores/config.svelte.ts` | Backend returns `provider` as plain string; frontend type is a union | Added `as ProviderType` cast |

### 8.2 Eight missing backend commands (now added)

The frontend was calling Tauri IPC commands that did not exist in the Rust backend. The app would silently fail whenever these features were used.

| Missing command | Where called from | What it does | Fix |
|---|---|---|---|
| `get_templates` | `src/lib/stores/ai.svelte.ts` | Returns the 8 built-in AI prompt templates | Added to `commands/ai.rs` + registered in `lib.rs` |
| `git_log` | `src/lib/components/HistoryBrowser.svelte` | Alias for git history with optional file filter | Added to `commands/git.rs` + registered |
| `git_restore` | `src/lib/components/HistoryBrowser.svelte` | Restores a file to a past commit version | Added to `commands/git.rs` + registered |
| `mcp_toggle_server` | `src/lib/stores/mcp.svelte.ts` | Enable/disable an MCP server by id | Added to `commands/mcp.rs` + registered |
| `sandbox_exec` | `src/lib/commands/sandbox.ts` | Run a WASM file directly by path | Added to `commands/sandbox.rs` + registered |
| `sandbox_list_snapshots` | `src/lib/commands/sandbox.ts` | List saved sandbox snapshots | Added (stub) + registered |
| `sandbox_create_snapshot` | `src/lib/commands/sandbox.ts` | Save a sandbox state | Added (stub) + registered |
| `sandbox_restore_snapshot` | `src/lib/commands/sandbox.ts` | Restore a snapshot | Added (stub) + registered |
| `sandbox_delete_snapshot` | `src/lib/commands/sandbox.ts` | Delete a snapshot | Added (stub) + registered |

### 8.3 Parameter mismatches (now fixed)

Several commands existed in both frontend and backend but used different parameter names — causing silent failures at runtime.

| Command | Frontend sent | Backend expected | Fix |
|---|---|---|---|
| `mcp_add_server` | `{ server: McpServerInfo }` | `config_data: Value` | Backend now accepts `server` key |
| `mcp_remove_server` | `{ id }` | `name: String` | Backend now matches by `id` OR `name` |
| `mcp_test_connection` | `{ id }` | `name: String` | Backend parameter renamed to `id` |
| `mcp_toggle_server` | `{ id, enabled }` | (did not exist) | Added new command |
| `git_diff` | `{ filePath, from, to }` | `{ path, rev1, rev2 }` | HistoryBrowser now sends `path/rev1/rev2` |

### 8.4 McpServerInfo type alignment (now fixed)

The frontend's `McpServerInfo` interface had `id`, `enabled`, `command`, `args`, `url`, `errorMessage` fields. The Rust `McpServerInfo` struct was missing all of these. Added them to `types.rs`.

### 8.5 Sandbox commands not registered (now fixed)

Three sandbox commands existed in the Rust code (`sandbox_execute`, `sandbox_get_tools`, `sandbox_test_tool`) but were never added to the `invoke_handler![]` list in `lib.rs`. The sandbox panel would silently fail. All are now registered.

### 8.6 MCP transport crash bug (now fixed)

`StdioTransport::new()` in `mcp/transport.rs` used `.expect()` which causes the entire app to crash if an MCP server command fails to start. Added `.kill_on_drop(true)` and improved error logging so a bad MCP server fails gracefully instead of crashing ZarishNote.

### 8.7 Stale files removed

- `dist/index.html` — old build artifact committed accidentally, caused confusion
- `zs_note_interface/` — design mockup folder, not part of the actual app

---

## 9. Future Maintenance

### 9.1 Making changes to notes (normal daily use)

No terminal needed. Just open ZarishNote and edit. Saves are automatic.

### 9.2 Making code changes

If you need to change the app itself (with help from an AI assistant like Claude):

```bash
cd ~/zs-note
# Make changes to files using a text editor or AI
pnpm typecheck              # Verify no TypeScript errors
git add .
git commit -m "describe what changed"
git push origin main
```

To release a new version:

```bash
git tag v0.1.1
git push origin v0.1.1
```

GitHub builds the new installer automatically. Download it from the Releases page.

### 9.3 Updating Dependabot security alerts

When GitHub sends Dependabot alerts (security updates for packages):

1. Go to `https://github.com/zarishsphere/zs-note/pulls`
2. Find pull requests from `dependabot`
3. Click each one
4. Click **Merge pull request** → **Confirm merge**
5. Then run locally: `git pull origin main`

There are currently 9 open Dependabot PRs — merge them in order from oldest to newest.

### 9.4 Backing up your notes

Your notes are stored as plain Markdown files in your vault folder (e.g., `~/Documents/my-notes`). Back them up like any folder:

- Copy to an external USB drive
- Or push the vault folder to a private GitHub repo:

```bash
cd ~/Documents/my-notes
git init
git remote add origin https://github.com/YOUR_USERNAME/my-notes-private.git
git add .
git commit -m "backup"
git push origin main
```

---

## 10. Troubleshooting

### 10.1 "command not found: pnpm"

Close the terminal, open a new one, then:

```bash
source ~/.bashrc
pnpm --version
```

### 10.2 "error: linker `cc` not found"

```bash
sudo apt install -y build-essential
```

### 10.3 GitHub push asks for username/password repeatedly

Set up credential storage:

```bash
git config --global credential.helper store
```

Then push once more and enter your PAT. It will be saved.

### 10.4 The app opens but AI does nothing

The most common cause: no API key set and Ollama not running.

```bash
ollama serve
```

Leave this terminal open, then try the AI panel in ZarishNote again.

### 10.5 GitHub Actions build fails

Go to `https://github.com/zarishsphere/zs-note/actions`, click the failed run, expand the failed step, and read the error. Copy the error text and paste it into Claude for help.

### 10.6 TypeScript errors after making code changes

```bash
cd ~/zs-note
pnpm typecheck 2>&1 | grep "error TS"
```

Copy the error lines and paste into Claude for help.

### 10.7 The app crashes when clicking MCP features

Ensure that any MCP server binary you configured actually exists at the path you provided in Settings → MCP Servers. Remove or disable servers whose commands do not exist on your machine.

---

*ZarishSphere Foundation · V1 · June 2026*
*License: CC BY 4.0*
*GitHub: https://github.com/zarishsphere/zs-note*
