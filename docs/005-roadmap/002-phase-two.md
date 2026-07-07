# 002-phase-two.md
## ZarishNote Phase 2 Roadmap
### Weeks 9–16: Full feature set and mobile

**Document type:** Roadmap — V1
**Date:** June 08, 2026
**Author:** Mohammad Ariful Islam / ZarishSphere Foundation
**License:** CC BY 4.0
**Status:** V1 — Authoritative

---

## Table of Contents

1. [Scope](#1-scope)
2. [Week 9–10: Mobile + Multi-Window](#2-week-9-10-mobile--multi-window)
3. [Week 11–12: MCP Marketplace + Plugin System](#3-week-11-12-mcp-marketplace--plugin-system)
4. [Week 13–14: Advanced AI + Image Generation](#4-week-13-14-advanced-ai--image-generation)
5. [Week 15–16: Polish, Performance, Ecosystem](#5-week-15-16-polish-performance-ecosystem)

---

## 1. Scope

Phase 2 extends ZarishNote to **iOS and Android**, adds the MCP marketplace and plugin ecosystem, advanced AI features (image generation, speaker labels), and cross-device sync.

### 1.1 What Phase 2 Adds

| Feature | Status |
|---|---|
| Mobile (iOS + Android) via Tauri v2 | ✅ |
| Touch toolbar, tab bar, mobile-optimized UI | ✅ |
| Multi-window on desktop | ✅ |
| Multi-tab within window | ✅ |
| MCP marketplace (one-click install) | ✅ |
| Plugin API + WASM plugin system | ✅ |
| Image generation (DALL-E, Stability AI) | ✅ |
| Voice speaker labels (diarization) | ✅ |
| Cross-device Git sync | ✅ |
| i18n (Bangla, English, Arabic) | ✅ |
| Performance optimization (10K+ file vaults) | ✅ |

---

## 2. Mobile + Multi-Window

- [ ] Tauri v2 mobile targets: iOS (arm64), Android (arm64)
- [ ] Touch toolbar for Markdown formatting
- [ ] Tab bar for multi-file editing
- [ ] iPad Magic Keyboard shortcut support
- [ ] Floating toolbar on touch surfaces
- [ ] Mobile-optimized AI panel (bottom sheet instead of right panel)
- [ ] Multi-window support on desktop
- [ ] Window state persistence (position, size, active file)
- [ ] App Store preparation (Apple App Store submission, Google Play Store)
- [ ] Mobile automated testing (emulator/device farm integration)

---

## 3. MCP Marketplace + Plugin System

- [ ] MCP marketplace registry API
- [ ] One-click MCP server installation
- [ ] Server version management and updates
- [ ] WASM plugin API (WIT interface, host functions)
- [ ] Plugin manifest format and validation
- [ ] Plugin installer (download `.wasm` to `.znrc-plugins/`)
- [ ] Plugin sandboxing (capability model same as tools)
- [ ] Plugin marketplace browser UI
- [ ] Plugin signing and verification
- [ ] Plugin development guide and sample plugins

---

## 4. Advanced AI + Image Generation

- [ ] Image generation dialog (DALL-E, Stability AI)
- [ ] Image saving to `assets/ai-images/` + Markdown insertion
- [ ] Speaker diarization via pyannote.audio
- [ ] Speaker label UI (rename speakers, color coding)
- [ ] SRT export with speaker labels
- [ ] Temperature/parameter controls per model
- [ ] System prompt configuration per workspace
- [ ] Multi-provider routing rules GUI
- [ ] AI template library expansion
- [ ] Context inspector panel (what is being sent to AI)

---

## 5. Polish, Performance, Ecosystem

- [ ] i18n framework via svelte-i18n or FormatJS (Bangla, English, Arabic as first targets)
- [ ] Translation file format specification and external translation management guide
- [ ] Right-to-left (RTL) layout testing for Arabic
- [ ] Right-to-left text support
- [ ] Large vault performance (10K+ files)
- [ ] Lazy-loading file tree for large vaults
- [ ] Memory profiling and optimization
- [ ] Accessibility audit (WCAG 2.1 AA)
- [ ] Documentation site launch
- [ ] Community contribution guidelines
- [ ] Performance benchmarks and regression suite
- [ ] Beta testing program for mobile

---

*ZarishSphere Foundation · V1 · June 2026*
*License: CC BY 4.0*
