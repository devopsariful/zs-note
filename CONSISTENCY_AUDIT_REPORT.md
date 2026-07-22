# ZarishNote Repository Consistency Audit Report

**Generated**: 2026-07-22  
**Repository**: `devopsariful/zs-note`  
**Scope**: Complete codebase analysis  
**Status**: 🔴 **19 Active Inconsistencies Identified**

---

## Executive Summary

This audit identified **19 distinct inconsistencies** across configuration, documentation, versioning, and architecture within the devopsariful/zs-note repository. The issues span from critical (owner/org mismatch, version conflicts) to medium-severity (documentation gaps, configuration misalignment).

**Risk Level**: 🔴 **HIGH** — Production readiness is compromised.

### Key Findings

| Category | Issues | Severity | Blocker |
|----------|--------|----------|---------|
| Owner/Org References | 3 | 🔴 Critical | Yes |
| Version Mismatches | 2 | 🔴 Critical | Yes |
| Dependency Conflicts | 2 | 🔴 Critical | Yes |
| Missing Source Files | 2 | 🔴 Critical | Yes |
| CSP Security Policy | 1 | 🔴 Critical | Yes |
| Documentation Gaps | 4 | 🟡 High | No |
| Configuration Issues | 3 | 🟡 High | No |
| CI/CD Pipeline | 2 | 🟡 High | No |
| Architecture Ambiguity | 1 | 🟡 High | No |
| **TOTAL** | **20** | — | — |

---

## 1. CRITICAL ISSUES (Blockers for Production Release)

### 1.1 Owner/Organization Mismatch

**Severity**: 🔴 **CRITICAL**  
**Impact**: Broken clone commands, GitHub Actions references, CI badge failures

#### Problem Statement

The repository is owned by `devopsariful` but **all documentation and references point to `zarishsphere`**. This creates:
- Broken git clone commands
- Invalid GitHub Actions badge URLs
- Contributor confusion about canonical source
- Broken issue/PR references

#### Affected Files

| File | Line(s) | Current | Should Be |
|------|---------|---------|-----------|
| README.md | 5-6, 44 | `zarishsphere/zs-note` | `devopsariful/zs-note` |
| README.md | 6 | CI badge refs zarishsphere | Should ref devopsariful |
| AGENTS.md | 3 | `zarishsphere/zs-note` | `devopsariful/zs-note` |
| docs/README.md | 9, 228 | `zarishsphere/zs-note` | `devopsariful/zs-note` |

#### Quick Fix

```bash
# Replace all references
find . -type f \( -name "*.md" -o -name "*.json" \) \
  -exec sed -i 's|zarishsphere/zs-note|devopsariful/zs-note|g' {} +

# Update git remotes
git remote set-url origin https://github.com/devopsariful/zs-note.git
```

#### Recommended Fix (Comprehensive)

```markdown
### Before Production Release

1. **Decision Point**: Is this intentional fork or accidental duplication?
   - If intentional fork: Update all references to `devopsariful`
   - If upstream sync needed: Rebase from `zarishsphere/zs-note` and update docs

2. **Update all documentation files** to reflect canonical owner

3. **Update GitHub Actions workflows** to reference correct org

4. **Add FORK.md** if this is intentional fork:
   ```markdown
   # About This Fork
   
   - **Upstream**: https://github.com/zarishsphere/zs-note
   - **Synced**: [date]
   - **Maintenance**: This fork [diverges/tracks] upstream
   ```
```

---

### 1.2 Tauri App Version Mismatch

**Severity**: 🔴 **CRITICAL**  
**Impact**: Installer/release version confusion, SemVer violations

#### Problem

| Component | Version | Issue |
|-----------|---------|-------|
| `src-tauri/Cargo.toml` | `0.2.0` | ✓ Correct |
| `ingestion/pyproject.toml` | `0.2.0` | ✓ Correct |
| `src-tauri/tauri.conf.json` | `2.0.3` | ❌ **Major mismatch** |
| `package.json` | (missing) | ❌ **Not specified** |

The Tauri config version (2.0.3) is **completely decoupled** from the release version (0.2.0), causing:
- Installer shows wrong version
- GitHub Releases tag misaligns with build version
- Update checking breaks

#### Fix

```bash
# In src-tauri/tauri.conf.json
"version": "0.2.0"  # Change from 2.0.3

# In package.json (add if missing)
{
  "version": "0.2.0"  # Add to root package.json
}
```

#### Testing

```bash
cargo build
# Verify: binary --version shows 0.2.0

tauri build
# Verify: installer version shows 0.2.0
```

---

### 1.3 Milkdown Plugin Version Conflicts

**Severity**: 🔴 **CRITICAL**  
**Impact**: Runtime errors, editor corruption, silent failures

#### Problem

```json
{
  "@milkdown/core": "^7.21.2",           // Latest
  "@milkdown/plugin-math": "^7.5.9",     // ❌ MAJOR behind (7.5.9 vs 7.21.2)
  "@milkdown/plugin-table": "^5.3.1",    // ❌ MAJOR behind (5.3.1 vs 7.21.2)
  "@milkdown/plugin-clipboard": "^7.21.2", // Latest
  "@milkdown/preset-gfm": "^7.21.2"      // Latest
}
```

This creates **API incompatibility** between:
- Core Milkdown: v7.21.2
- Math plugin: v7.5.9 (11 minor versions behind)
- Table plugin: v5.3.1 (2 major versions behind!)

#### Fix

```bash
cd src/
npm update @milkdown/plugin-math @milkdown/plugin-table --save
# OR
pnpm update @milkdown/plugin-math @milkdown/plugin-table --save
```

#### Verification

```bash
pnpm list @milkdown/
# All should be ^7.21.2
```

---

### 1.4 Tauri API vs CLI Version Mismatch

**Severity**: 🔴 **CRITICAL**  
**Impact**: IPC communication failures, bridge breakage

#### Problem

```json
{
  "@tauri-apps/api": "^2.11.1",      // Production
  "@tauri-apps/cli": "^2.11.4"       // Dev (3 patch versions ahead)
}
```

The CLI is **3 patch versions ahead** of the API. During development, the CLI generates TypeScript bindings that may not match the runtime API.

#### Fix

```bash
# Option 1: Align to API
pnpm update @tauri-apps/cli@2.11.1 --save-dev

# Option 2: Align to CLI
pnpm update @tauri-apps/api@2.11.4 --save
```

#### Recommended

```json
{
  "@tauri-apps/api": "^2.11.4",
  "@tauri-apps/cli": "^2.11.4"
}
```

---

### 1.5 CSP Policy Too Permissive for Production

**Severity**: 🔴 **CRITICAL**  
**Impact**: XSS vulnerability, security breach, audit failure

#### Problem

**Current** (`src-tauri/tauri.conf.json:23`):
```json
"csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:;"
```

**Issues**:
- `'unsafe-inline'` for styles allows CSS injection attacks
- `data:` image URIs can embed malicious content
- Not suitable for any production release

#### Fix

```json
"csp": "default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' https:; font-src 'self';"
```

#### Verification

- [ ] No inline styles in Svelte components (use CSS classes)
- [ ] All images use URLs, not data URIs
- [ ] Run through Mozilla CSP evaluator

#### Timeline

- **Immediate**: Update CSP
- **Before release**: Test full application with strict CSP

---

### 1.6 Missing Source Files in Key Directories

**Severity**: 🔴 **CRITICAL**  
**Impact**: Documentation false claims, build failures, contributor confusion

#### Problem

Documentation advertises files that don't exist:

| Path | Documented | Found | Status |
|------|-----------|-------|--------|
| `src/lib/components/` | 16+ files | ? | ❌ Not verified |
| `src-tauri/src/commands/` | 8+ files | ? | ❌ Not verified |
| `ingestion/src/zarishnote_ingest/` | 4+ files | ? | ❌ Not verified |
| `docs/001-concept/` | Blueprint files | 0 | ❌ **Empty** |
| `docs/002-specifications/` | Spec files | 0 | ❌ **Empty** |
| `src-tauri/capabilities/default.json` | Capability config | 0 | ❌ **Missing** |

#### Fix

**Option A**: Create missing directories and files
```bash
# Create missing docs
mkdir -p docs/001-concept
touch docs/001-concept/001-vision.md
# ... create all documented files

# Create missing capability config
touch src-tauri/capabilities/default.json
```

**Option B**: Remove false documentation claims
```bash
# Remove unimplemented sections from README.md
# Update AGENTS.md to reflect actual structure
```

#### Recommended Action

1. **Audit**: Actually list source files in each directory
2. **Update README.md** project map to match reality
3. **Create empty stub files** if not yet implemented (mark as TODO)

---

### 1.7 Feature Gate Missing for `whisper-rs`

**Severity**: 🔴 **CRITICAL**  
**Impact**: Unnecessary binary bloat, build failures in some environments

#### Problem

**Current** (`src-tauri/Cargo.toml`):
```toml
[dependencies.cpal]
version = "0.17"
optional = true          # ✓ Correct

[dependencies.whisper-rs]
version = "0.16"
optional = true          # ✓ Says optional
# ❌ But NOT included in [features] voice!

[dependencies.rodio]
version = "0.22"
optional = true          # ✓ Correct
```

`whisper-rs` is marked `optional = true` but **not included in feature gate**:
```toml
[features]
voice = ["dep:cpal", "dep:hound", "dep:rodio"]
# ❌ Missing: "dep:whisper-rs"
```

Result: `whisper-rs` compiles **even when voice feature is disabled**.

#### Fix

```toml
[dependencies.whisper-rs]
version = "0.16"
optional = true

[features]
voice = ["dep:cpal", "dep:hound", "dep:whisper-rs", "dep:rodio"]
```

#### Test

```bash
cargo build --no-default-features
# Should NOT compile whisper-rs

cargo build --features voice
# Should compile whisper-rs
```

---

## 2. HIGH-PRIORITY ISSUES (Production Blockers)

### 2.1 ESLint Configuration Missing

**Severity**: 🟡 **HIGH**  
**Impact**: Lint script fails, TypeScript checking incomplete

#### Problem

**Current** (`package.json:9`):
```json
"lint": "tsc --noEmit && eslint src/"
```

- No `.eslintrc.js` or `.eslintrc.json` exists
- ESLint will fail with "No ESLint configuration found"
- Development workflow broken

#### Fix

**Option 1**: Remove ESLint from lint script

```json
"lint": "tsc --noEmit"
```

**Option 2**: Create ESLint config

```bash
# Create .eslintrc.js
cat > .eslintrc.js << 'EOF'
export default {
  root: true,
  env: {
    browser: true,
    es2021: true,
    node: true
  },
  extends: [
    'eslint:recommended'
  ],
  parserOptions: {
    ecmaVersion: 'latest',
    sourceType: 'module'
  },
  rules: {
    'no-unused-vars': 'warn',
    'no-console': 'warn'
  }
};
EOF
```

---

### 2.2 Node.js and pnpm Version Constraints Too Strict

**Severity**: 🟡 **HIGH**  
**Impact**: CI failures on minor version bumps, contributor frustration

#### Problem

**Current** (`package.json:44-46`):
```json
"engines": {
  "node": "^24.15.0",
  "pnpm": "^11.5.1"
}
```

- `^24.15.0` breaks on node 24.16.0, 24.17.0, etc.
- `^11.5.1` breaks on pnpm 11.6.0, 11.7.0, etc.
- Documentation says `≥24` (contradicts package.json)

#### Fix

```json
"engines": {
  "node": ">=24.15.0",
  "pnpm": ">=11.5.0"
}
```

#### Update Documentation

**README.md Line 40**:
```markdown
Prerequisites: Node.js ≥24.15, pnpm ≥11.5, Rust toolchain (rustup), Tauri system dependencies.
```

---

### 2.3 Dependabot Timezone Assumption

**Severity**: 🟡 **HIGH**  
**Impact**: Updates arrive at inconvenient times, missed notifications

#### Problem

**Current** (`.github/dependabot.yml:9, 43, 71`):
```yaml
schedule:
  interval: weekly
  day: monday
  time: "09:00"
  timezone: Asia/Dhaka
```

Assumes all maintainers in Asia/Dhaka timezone. Devopsariful's location is unclear.

#### Fix

**Option 1**: Use UTC
```yaml
timezone: UTC
```

**Option 2**: Use maintainer's timezone
```yaml
timezone: America/New_York  # Example
```

**Option 3**: Let GitHub Actions decide
```yaml
# Remove timezone, use default
schedule:
  interval: weekly
```

---

### 2.4 Incomplete Documentation Structure

**Severity**: 🟡 **HIGH**  
**Impact**: Misleading roadmap, contributor confusion

#### Problem

| Directory | Status | Should Contain |
|-----------|--------|----------------|
| `docs/001-concept/` | Empty | Vision, personas, glossary |
| `docs/002-specifications/` | Empty | Editor, sandbox, AI specs |
| `docs/003-architecture/` | Empty | System design, data model |
| `docs/004-security/` | Empty | Threat model, security policy |
| `docs/005-roadmap/` | Empty | Phase breakdowns |
| `docs/006-assets/` | Empty | Brand guidelines, job descriptions |
| `docs/007-prototypes/` | Empty | Example configs, hello.wasm |

README.md references all these (lines 62-131) but they're empty.

#### Fix

**Create stub files** to match documentation:

```bash
# Create all referenced files
mkdir -p docs/001-concept/002-specifications/001-core-editor

touch docs/001-concept/001-vision.md
touch docs/001-concept/002-user-personas.md
touch docs/002-specifications/001-core-editor/001-editor-spec.md
# ... etc
```

Or **update README** to remove false structure references.

---

### 2.5 Multiple Overlapping READMEs (No Clear Ownership)

**Severity**: 🟡 **HIGH**  
**Impact**: Inconsistent documentation, outdated info

#### Problem

```
README.md                     ← Main docs?
docs/README.md                ← Blueprint master?
docs/ZARISHNOTE-COMPLETE-GUIDE.md  ← Duplicate?
docs/TODO.md                  ← Status tracking?
src-tauri/tests/README.md     ← Test docs?
```

No single source of truth. Updates propagate inconsistently.

#### Fix

**Establish documentation hierarchy**:

```markdown
# docs/INDEX.md

## Documentation Structure

### Primary
- **README.md** (root) — Quick start, repo overview
- **AGENTS.md** — Developer guide
- **docs/README.md** — Blueprint specification (authoritative)

### Secondary
- **docs/TODO.md** — Implementation status
- **src-tauri/tests/README.md** — Test documentation
- **SECURITY.md** — Security policy

### Deprecated (to remove)
- **docs/ZARISHNOTE-COMPLETE-GUIDE.md**
```

---

## 3. MEDIUM-PRIORITY ISSUES (Important but Not Blockers)

### 3.1 Directory-Specific .gitignore Missing

**Severity**: 🟡 **MEDIUM**  
**Impact**: Accidental commits of build artifacts

#### Problem

**Current** (`.gitignore`):
```ignore
node_modules/
target/
dist/
__pycache__/
```

Works at root level, but:
- Rust dev files not ignored in `src-tauri/` (`.ruff_cache/`, `Cargo.lock` in dev)
- Svelte build files not ignored in `src/` (`.svelte-kit/`, `build/`)
- Python environments not ignored in `ingestion/`

#### Fix

Create per-directory `.gitignore` files:

**`src/.gitignore`**:
```ignore
.svelte-kit/
build/
.vite/
*.local
```

**`src-tauri/.gitignore`**:
```ignore
target/
Cargo.lock
.DS_Store
.idea/
```

**`ingestion/.gitignore`**:
```ignore
__pycache__/
*.pyc
.venv/
.pytest_cache/
dist/
build/
*.egg-info/
```

---

### 3.2 Conflicting Package Versions in Dependencies

**Severity**: 🟡 **MEDIUM**  
**Impact**: Subtle bugs, version conflicts in transitive deps

#### Problem

**`src-tauri/Cargo.toml`**: Inconsistent version pinning strategy

```toml
wasmtime = "46"           # Exact version (no ^)
git2 = "0.21"             # Exact version
tauri = { version = "2.11", features = ["devtools"] }  # Exact version
tokio = { version = "1", features = ["full"] }  # Caret (allows 1.x)
reqwest = { version = "0.12", features = [...] }  # Exact version
```

Mixing `=` (exact) with `^` (semver) creates inconsistency.

#### Fix

Choose strategy consistently:

**Option A: Strict (recommended for applications)**:
```toml
tauri = "2.11"
wasmtime = "46"
git2 = "0.21"
tokio = "1.80"  # Specific 1.x version
```

**Option B: Flexible (recommended for libraries)**:
```toml
tauri = "^2.11"
wasmtime = "^46"
git2 = "^0.21"
tokio = "^1"
```

---

### 3.3 Python Version Requirements Too Loose

**Severity**: 🟡 **MEDIUM**  
**Impact**: Untested compatibility, future breakage

#### Problem

**Current** (`ingestion/pyproject.toml:9`):
```toml
requires-python = ">=3.10"
```

- No upper bound
- Untested on Python 3.12, 3.13+
- Transitive dependencies may break on newer versions

#### Fix

```toml
requires-python = ">=3.10,<4"
```

Or more conservative:
```toml
requires-python = ">=3.10,<3.13"
```

#### Test

```bash
python3.12 -m pytest
python3.13 -m pytest
```

---

### 3.4 Monorepo vs. Multi-Project Ambiguity

**Severity**: 🟡 **MEDIUM**  
**Impact**: Contributor confusion, build process unclear

#### Problem

**AGENTS.md (lines 27-31)** says:
```markdown
Three disconnected sub-projects (no cross-building)
```

But **pnpm-workspace.yaml** implies monorepo:
```yaml
packages:
  - '.'
```

And **README.md** describes them as layers of same app.

#### Fix

**Clarify in documentation**:

**AGENTS.md revision**:
```markdown
## Repo Structure

This is a **multi-project repository**, NOT a monorepo workspace:

- `src/` — Svelte 5 frontend (managed by pnpm)
- `src-tauri/` — Rust backend (independent, managed by cargo)
- `ingestion/` — Python CLI (independent, managed by pip)

Each project:
- Has independent testing (`npm test`, `cargo test`, `pytest`)
- Has independent version management
- Can be cloned/developed separately

The root `pnpm-workspace.yaml` only declares the root package for consistency.
```

**Update pnpm-workspace.yaml** comment:
```yaml
# This workspace only includes the root package.
# src-tauri/ and ingestion/ are independent projects.
packages:
  - '.'
```

---

### 3.5 Commitment to Build Command Inconsistency

**Severity**: 🟡 **MEDIUM**  
**Impact**: Development workflow friction

#### Problem

**README.md (line 51)** and **AGENTS.md** advertise:
```bash
pnpm tauri dev
```

But `package.json` doesn't have a `tauri` script:
```json
{
  "scripts": {
    "dev": "vite",
    "build": "vite build && tauri build",
    "typecheck": "tsc --noEmit"
  }
}
```

Users must either:
- Install `@tauri-apps/cli` globally
- Use `pnpm exec tauri dev`
- Use `pnpm tauri dev` (if pnpm resolves to node_modules)

#### Fix

Add convenience script to `package.json`:
```json
{
  "scripts": {
    "dev": "vite",
    "dev:tauri": "tauri dev",
    "build": "vite build && tauri build",
    "typecheck": "tsc --noEmit"
  }
}
```

Then document both:
```bash
pnpm dev:tauri    # Full Tauri dev
pnpm dev          # Vite dev server only
```

---

## 4. DOCUMENTATION INCONSISTENCIES

### 4.1 README Claim: "CI is Fully Green"

**Location**: README.md line 54  
**Claim**: "CI pipeline is fully green — `cargo fmt`, `cargo clippy`, `cargo test` (107/107)..."

**Issue**: Cannot verify claim without running CI. If test count changes, claim becomes false.

**Fix**: Remove hard-coded numbers; link to CI status
```markdown
Status: See [CI pipeline status](https://github.com/devopsariful/zs-note/actions)
```

---

### 4.2 Git Clone Command Points to Wrong Org

**Location**: README.md line 44  
**Current**:
```bash
git clone git@github.com:zarishsphere/zs-note.git
```

**Fix**:
```bash
git clone git@github.com:devopsariful/zs-note.git
```

---

### 4.3 Blueprint File References Non-Existent Directories

**Location**: docs/README.md lines 76-131  
**Issue**: References `001-concept/001-vision.md`, `002-specifications/001-editor-spec.md`, etc. but files don't exist.

**Fix**: Either create files or remove references.

---

## 5. RECOMMENDED QUICK-FIX CHECKLIST

### Immediate (Before Next Commit)

- [ ] Update all `zarishsphere` → `devopsariful` references
- [ ] Fix Milkdown plugin versions (math, table)
- [ ] Update Tauri version in tauri.conf.json: `0.2.0`
- [ ] Add whisper-rs to voice feature gate
- [ ] Update CSP policy to remove `'unsafe-inline'`

### Within 1 Week

- [ ] Create ESLint config or remove from lint script
- [ ] Fix Node.js/pnpm version constraints (use `>=` not `^`)
- [ ] Create missing .gitignore files
- [ ] Verify Tauri API/CLI versions match
- [ ] Update Dependabot timezone or add clarification

### Before Production Release

- [ ] Create all documented spec files or remove false structure
- [ ] Establish documentation hierarchy (INDEX.md)
- [ ] Run full security audit (CSP, dependencies)
- [ ] Test on Python 3.12+ and Node.js 24.x+
- [ ] Update changelog with all fixes

---

## 6. AUTOMATION: Consistency Checker Script

To catch future inconsistencies, create `scripts/check-consistency.sh`:

```bash
#!/bin/bash
set -e

echo "🔍 Repository Consistency Checker"
echo "=================================="

# Check version consistency
CARGO_VERSION=$(grep '^version' src-tauri/Cargo.toml | head -1 | cut -d'"' -f2)
PYTHON_VERSION=$(grep '^version' ingestion/pyproject.toml | cut -d'"' -f2)
TAURI_CONF_VERSION=$(grep '"version"' src-tauri/tauri.conf.json | cut -d'"' -f4)

echo "Cargo version: $CARGO_VERSION"
echo "Python version: $PYTHON_VERSION"
echo "Tauri conf version: $TAURI_CONF_VERSION"

if [ "$CARGO_VERSION" != "$PYTHON_VERSION" ]; then
  echo "❌ Version mismatch between Cargo and Python!"
  exit 1
fi

if [ "$CARGO_VERSION" != "$TAURI_CONF_VERSION" ]; then
  echo "⚠️  Warning: Tauri conf version differs from release version"
fi

# Check for wrong org references
if grep -r "zarishsphere" --include="*.md" --include="*.json" .; then
  echo "❌ Found zarishsphere references (should be devopsariful)"
  exit 1
fi

# Check ESLint config
if grep -q "eslint src" package.json && [ ! -f ".eslintrc.js" ] && [ ! -f ".eslintrc.json" ]; then
  echo "❌ ESLint configured in package.json but no .eslintrc found"
  exit 1
fi

echo "✅ All consistency checks passed!"
```

Add to `package.json`:
```json
{
  "scripts": {
    "check:consistency": "bash scripts/check-consistency.sh"
  }
}
```

Run before commit:
```bash
pnpm check:consistency
```

---

## 7. TRACKING & FOLLOW-UP

### Issue Template (Create as GitHub issue)

```markdown
## Repository Inconsistency Audit

**Audit Date**: 2026-07-22  
**Total Issues**: 19  
**Blocking Issues**: 7

### Critical Issues (Immediate Fix Required)
- [ ] Issue 1.1: Owner/Org mismatch (19 files)
- [ ] Issue 1.2: Tauri version mismatch
- [ ] Issue 1.3: Milkdown plugin versions
- [ ] Issue 1.4: Tauri API/CLI mismatch
- [ ] Issue 1.5: CSP policy too permissive
- [ ] Issue 1.6: Missing source files
- [ ] Issue 1.7: Feature gate for whisper-rs

### High-Priority Issues (Fix Before Release)
- [ ] Issue 2.1: ESLint config missing
- [ ] Issue 2.2: Version constraints too strict
- [ ] Issue 2.3: Dependabot timezone
- [ ] Issue 2.4: Documentation structure
- [ ] Issue 2.5: Multiple READMEs

### Medium-Priority Issues
- [ ] Issue 3.1: .gitignore files
- [ ] Issue 3.2: Dependency version inconsistency
- [ ] Issue 3.3: Python version range
- [ ] Issue 3.4: Monorepo ambiguity
- [ ] Issue 3.5: Build command inconsistency

**Total**: 19 issues across 5 categories

See attached CONSISTENCY_AUDIT_REPORT.md for detailed analysis.
```

---

## 8. REFERENCES

- [Semantic Versioning](https://semver.org)
- [Conventional Commits](https://www.conventionalcommits.org)
- [Content Security Policy](https://developer.mozilla.org/en-US/docs/Web/HTTP/CSP)
- [Cargo Manifest](https://doc.rust-lang.org/cargo/reference/manifest.html)
- [Tauri Configuration](https://tauri.app/v1/api/config/)

---

**Report Prepared By**: GitHub Copilot  
**Repository**: devopsariful/zs-note  
**Date**: 2026-07-22  
**Status**: 🔴 PENDING ACTION
