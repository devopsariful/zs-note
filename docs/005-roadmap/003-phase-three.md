# 003-phase-three.md
## ZarishNote Phase 3 Roadmap
### Weeks 17+: Ecosystem and advanced features

**Document type:** Roadmap — V1
**Date:** June 08, 2026
**Author:** Mohammad Ariful Islam / ZarishSphere Foundation
**License:** CC BY 4.0
**Status:** V1 — Authoritative

---

## Table of Contents

1. [Scope](#1-scope)
2. [Collaboration Features](#2-collaboration-features)
3. [Plugin Ecosystem](#3-plugin-ecosystem)
4. [Enterprise / Advanced](#4-enterprise--advanced)
5. [Long-Term Vision](#5-long-term-vision)

---

## 1. Scope

Phase 3 is the ongoing ecosystem phase. There is no fixed timeline — items are prioritized based on community demand and funding. Items may be added, removed, or reordered.

---

## 2. Collaboration Features

| Feature | Description |
|---|---|
| Real-time collaboration | CRDT-based concurrent editing via yjs (integrates with ProseMirror/Milkdown) |
| Comment and annotation | Inline comments on document sections |
| Shared vaults | Multi-user vault with permission levels |
| Change proposals | Suggest changes without direct edit |
| Presence indicators | See who else is viewing a document |

**Technical note:** Real-time collaboration requires a relay server. ZarishNote would provide an optional self-hosted relay (not a SaaS).

---

## 3. Plugin Ecosystem

| Feature | Description |
|---|---|
| Plugin marketplace launch | Public registry for community plugins |
| Plugin monetization | Optional paid plugins (ZarishSphere takes 0% cut) |
| Plugin categories | Renderers, panels, importers, exporters |
| WASM SDK | Stable API + documentation for plugin developers |
| Plugin testing framework | Run plugins in headless Wasmtime for CI |

---

## 4. Enterprise / Advanced

| Feature | Description |
|---|---|
| FHIR integration | Import/export healthcare data via FHIR API |
| Encryption at rest | Vault-level encryption via Age or GPG |
| LDAP/OIDC auth | Enterprise authentication for shared vaults |
| Audit compliance | Exportable audit trails for regulated environments |
| Custom branding | White-label for NGOs and institutions |
| Bulk document workflows | Template-based document generation pipeline |
| Advanced automation | Webhook triggers on file events |

---

## 5. Long-Term Vision

### 5.1 Decentralized Knowledge Platform

ZarishNote, in its mature form, aims to be more than a note editor — a platform for **decentralized knowledge management** in humanitarian and public health contexts:

- **Offline-first field data collection** with structured forms
- **Semantic document linking** across vaults
- **Peer-to-peer sync** (via IPFS or similar)
- **AI-assisted translation** across 100+ languages
- **Standard operating procedure (SOP) automation**

### 5.2 Sustainability

| Model | Description |
|---|---|
| Open source | Apache 2.0, CC BY 4.0 — always free |
| Community | GitHub issues, discussions, contributions |
| Funding | Grants (humanitarian tech), donations, optional enterprise support |
| Governance | ZarishSphere Foundation |

---

*ZarishSphere Foundation · V1 · June 2026*
*License: CC BY 4.0*
