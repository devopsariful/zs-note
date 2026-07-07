# 003-brand-guidelines.md
## ZarishNote Brand Guidelines
### Name, colors, tone, and visual identity

**Document type:** Asset — V1
**Date:** June 08, 2026
**Author:** Mohammad Ariful Islam / ZarishSphere Foundation
**License:** CC BY 4.0

---

## Table of Contents

1. [Name and Logo](#1-name-and-logo)
2. [Color Palette](#2-color-palette)
3. [Typography](#3-typography)
4. [Tone of Voice](#4-tone-of-voice)
5. [Logo Usage](#5-logo-usage)

---

## 1. Name and Logo

### 1.1 Product Name

**ZarishNote** — always capitalized as shown. Not "Zarish Note", "zarishnote", or "ZarishNote®".

- Project code: `zs-note`
- Bundle ID: `com.zarishsphere.note`
- Repository: `zarishsphere/zs-note`

### 1.2 Logo

The ZarishNote logo is a stylized golden raindrop (representing the meaning of "Zarish" — golden rain) combined with a pen nib.

**Logo variants:**
- Full: Icon + "ZarishNote" wordmark (horizontal)
- Icon-only: Golden raindrop-nib (favicon, toolbar, app icon)
- Dark mode: White/gold on dark background
- Light mode: Dark blue on white background

---

## 2. Color Palette

| Role | Hex | Usage |
|---|---|---|
| Primary | `#1A1A2E` | Deep navy — app chrome, sidebar background |
| Accent | `#D4AF37` | Gold — highlighting, active states, logo |
| Surface | `#FFFFFF` | Editor background (light mode) |
| Surface dark | `#1E1E2E` | Editor background (dark mode) |
| Text | `#2D2D3A` | Body text (light mode) |
| Text dark | `#E4E4EB` | Body text (dark mode) |
| Muted | `#7C7C8A` | Secondary text, file tree |
| Success | `#4CAF50` | Sync status, save indicator |
| Warning | `#FF9800` | Unsaved changes, warnings |
| Error | `#F44336` | Errors, sandbox violations |

Primary/Acident gradient (for splash screen): `#1A1A2E → #D4AF37`

---

## 3. Typography

| Context | Font | Fallback |
|---|---|---|
| Editor (source mode) | JetBrains Mono | `monospace` |
| Editor (WYSIWYG body) | System UI | `-apple-system, Segoe UI, Roboto, Noto Sans` |
| UI headings | System UI (bold) | Same |
| Code blocks | JetBrains Mono | `monospace` |
| UI (mobile) | San Francisco (iOS), Roboto (Android) | System default |

### 3.1 Sizes

| Element | Size |
|---|---|
| App title (sidebar header) | 14px bold |
| File tree | 13px regular |
| Editor body | 16px regular (configurable) |
| Editor H1 | 28px bold |
| AI panel text | 14px regular |
| Status bar | 12px regular |

---

## 4. Tone of Voice

### 4.1 Principles

- **Calm and confident:** No hype language, no exclamation marks, no superlatives
- **Direct:** "Open a vault" not "Let's get started by opening a vault"
- **Honest about limitations:** "Available in Phase 2" not "Coming soon"
- **Developer-aware:** Settings and errors provide technical details when useful

### 4.2 Examples

| Context | Good | Avoid |
|---|---|---|
| Empty state | No documents yet. Open a vault or create a new file. | You have no documents! Click here to get started! |
| Error | Failed to load file: permission denied. Check file permissions. | Oops! Something went wrong. |
| Feature not available | Speaker labels are planned for Phase 2. | Speaker labels coming soon! |
| Config validation | `sandbox.engine` must be `"wasmtime"`. Got: `"wasm"`. | Invalid configuration value. |

---

## 5. Logo Usage

- Minimum clear space: 1x icon width on all sides
- Minimum size: 32px (icon), 120px (full logo)
- Do not: recolor, stretch, rotate, add shadows, or place on busy backgrounds
- Always provide dark mode variant when used on dark backgrounds

---

*ZarishSphere Foundation · V1 · June 2026*
*License: CC BY 4.0*
