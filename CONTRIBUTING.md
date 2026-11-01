# Contributing to Transparent SACCO on Logos

This guide is for **everyone who works in this repository — human contributors and AI coding agents alike**. It exists so that whoever picks up work (or whatever tool they point at the repo) follows the same rules and does not make a mess.

## How work is organized

- **Roadmap:** [`docs/roadmap-sacco-pivot.md`](./docs/roadmap-sacco-pivot.md) — the pivot from university fee transparency to a SACCO ledger, expressed as **12 steps across 8 phases**. Phase order is the build order.
- **Issues:** one issue per phase, with the phase's task checklist, step range, and exit criterion ("Done when"). Work is claimed on issues, not in chat.
- **Labels:** `lez` (Logos Blockchain integration) · `backend` (core, storage, messaging) · `frontend` (QML UI, packaging) · `community` (pilot, training, metrics).

## Ground rules for all contributors

1. **Claim before you build.** Assign yourself to the phase issue (or comment on it). One contributor per phase at a time; don't silently start work claimed by someone else.
2. **Stay inside your phase.** A phase issue lists exactly what's in scope. Anything outside it — rename refactors, dependency bumps, doc rewrites — needs its own discussion first.
3. **Never work on `main`.** Branch from `main`, named `phase<N>/<short-topic>` (e.g. `phase1/sacco-ledger-rename`). Changes reach `main` only through pull request review.
4. **Small PRs, one concern.** Reference the phase issue in the PR (`Closes #N`). Docs-only changes stay separate from code changes.
5. **Conventional commits** (`feat:`, `fix:`, `docs:`, `chore:`, `test:`), imperative subject line, reference the issue number when relevant.
6. **Check off tasks as you complete them** in the phase issue, and leave the "Done when" criterion for a maintainer to verify at review.
7. **Writing conventions:** documentation in this repo deliberately uses **step ordering, never durations or dates** ("Steps 2–4", not week ranges or calendar dates). Keep new docs consistent.
8. **Ask in the issue or PR** when scope is unclear. Don't guess around a blocker.

## Rules for AI coding agents

Agents are welcome here, under the same rules as humans plus the following, which exist because agents are the ones most likely to break them:

1. **Read before acting.** Read this file, the roadmap, and your assigned phase issue **before** touching anything. The roadmap is the source of truth for direction; the issue is the source of truth for scope.
2. **No repo-wide changes.** Do not reformat, restructure, or "clean up" files outside your phase's checklist. Renames and restructures of programs or modules are assigned work, not drive-bys (the Phase 1 `fee_transparency` → `sacco_ledger` rename is done; don't reintroduce the old name).
3. **Never force-push, never rewrite history, never push to `main`.** If your branch diverges, rebase onto `main` normally.
4. **Do not create, close, or mass-edit issues or labels.** Issue lifecycle belongs to the phase leads. If work needs a new issue, propose it in a comment.
5. **Run the checks you can:** for LEZ program changes, `RISC0_DEV_MODE=1 cargo test` in `programs/`; see [`docs/testing.md`](./docs/testing.md) for the rest. A change that skips its checks is not done.
6. **Keep commits reviewable.** Many small commits, each passing its checks. No mega-commits, no commit-and-fix-later.
7. **Don't commit secrets, tokens, or environment files.** Ever.
8. **If blocked or ambiguous, stop and comment on the issue** rather than improvising a workaround that changes scope.

## Repository map

| Path | What lives there |
|---|---|
| `docs/` | Roadmap, architecture, historical custody proposal, IDL, testing, deploy runbook, publishing docs, demo screenshots |
| `programs/sacco_ledger/` | LEZ program (Rust guest, RISC Zero) — the SACCO attestation ledger, renamed from `fee_transparency` in Phase 1 |
| `modules/sacco_core/`, `modules/sacco_ui/` | Logos Core module (submits IDL instructions to LEZ) and QML UI module (Member, Committee and Auditor views) |
| `artifacts/` | Exported program IDL |
| `prototype/` | Earlier React + Hardhat demo — reference only, not the production path |
| `docs/superpowers/` | Internal design and plan records for the pivot |

## Questions?

Open a discussion or comment on the relevant phase issue. Logos Circle Kampala leads: `YodHeVauHe` and the workstream owners named in the roadmap.
