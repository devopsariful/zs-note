# TODO: Actionable tasks for zs-note (CI triage)

Priority 1 — retrieve failing logs
- [ ] Download run 30660274312 logs from Actions UI and upload to `analysis/ci-logs/30660274312/` in this branch, or paste failing-step output here.
- [ ] Re-run the failing workflow with full logs if the failure seems transient.

Priority 2 — triage & reproduce locally
- [ ] If TypeScript errors: run locally:
  - pnpm install
  - pnpm run typecheck
  - Fix or pin offending types or packages.
- [ ] If pnpm/node install errors: ensure CI sets `actions/setup-node` to node-version 24 and `pnpm` install step is present.
- [ ] If Rust failures: run `rustup show` and `cargo test`/`cargo clippy` locally; reproduce failing crate tests and pin or upgrade toolchain as needed.

Priority 3 — CI hardening
- [ ] Add a CI step to run `scripts/check-consistency.sh --verbose` to catch version drift earlier.
- [ ] Cache pnpm and cargo artifacts effectively (workflow already includes caches; verify cache keys are correct).

Priority 4 — fixes & PRs
- [ ] After logs are parsed, create small targeted PR(s) that either:
  - fix TypeScript errors, or
  - pin Node/pnpm versions in workflows, or
  - adjust Cargo.toml to accommodate breaking changes, or
  - modify CI to run `rustup default` to the required toolchain.

How I will proceed once logs are uploaded
- Parse failing-step output, add exact error excerpts to `ANALYSIS_REPORT.md`, and propose a change set.
- Update `TODO.md` with the precise commands required to reproduce and fix the issue.
- Open PR(s) with the fixes.

