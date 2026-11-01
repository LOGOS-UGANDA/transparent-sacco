# Transparent SACCO

<p align="left">
  <a href="./LICENSE"><img alt="License: MIT" src="https://img.shields.io/badge/license-MIT-blue.svg"></a>
  <img alt="Status: in development" src="https://img.shields.io/badge/status-in%20development-orange">
  <img alt="Platform: Logos LEZ" src="https://img.shields.io/badge/platform-Logos%20LEZ-6C47FF">
</p>

**Every shilling accounted for. Every record verifiable.**

Transparent SACCO is a Logos-native **SACCO ledger**: every deposit, loan and expense a savings and credit cooperative handles is recorded and approved on the [Logos Execution Zone (LEZ)](https://docs.logos.co/), with multi-signature approval before any expense passes and members' personal balances kept private. It ships both as a **standalone desktop app** and as a **Basecamp module** users install from a package repository.

> **Status:** the `sacco_ledger` LEZ program, its local deployment, and the core and UI wiring are done (Phases 1–3): one full flow — record a deposit, the member sees and confirms it, an expense passes multi-sig — runs end to end inside Basecamp on a local sequencer. The Phase 0 pilot sign-off and baseline survey are still outstanding. See [Status](#status) for the per-phase breakdown.

## Contents

- [The problem](#the-problem)
- [The solution: a ledger, not a custodian](#the-solution-a-ledger-not-a-custodian)
- [Architecture](#architecture-logos-native)
- [Status](#status)
- [Roadmap](#roadmap)
- [Getting started](#getting-started)
- [Repository layout](#repository-layout)
- [Contributing](#contributing)
- [Research anchors](#research-anchors)
- [License](#license)

---

## The problem

SACCO members cannot see how their savings, loans and expenses are handled. Records live in treasurers' books and spreadsheets; trust is low, and when something goes wrong — lost savings, disputed expenses, unresolved estates — there is no shared record anyone can check.

## The solution: a ledger, not a custodian

The program is **the record, not the custodian of value**. Savings deposits, loan disbursements and expense payments happen through the SACCO's existing off-chain channels, such as mobile money, bank accounts and cash. LEZ records and approves the related activity; it does not receive, hold or transfer that money. Amounts and references stored on LEZ are accounting records, not funds held by the program. Development and product work can stay on the local sequencer; public testnet deployment is deferred and is not required to build the product.

| Capability | What it does |
|------------|--------------|
| Typed deposits | Record the off-chain amount, type (savings vs share capital), reference and member confirmation |
| Loan lifecycle | Applications, multi-sig approvals, and off-chain disbursement and repayment events are recorded on chain with evidence |
| Multi-sig expenses | Expenses receive multiple on-chain approvals; payment happens off chain and its evidence is recorded |
| Supervisory oversight | A Supervisory Committee is distinct from approvers — approve ≠ audit |
| Per-SACCO config | Approver thresholds and role composition are configured per SACCO, not chain-wide |
| Public totals, private members | Recorded SACCO totals are public and auditable; individual accounting records become private in the privacy phase |

## Architecture (Logos-native)

This is **not** an EVM / Solidity dApp. Logos LEZ runs RISC-V programs (RISC Zero zkVM). The product ships as Logos modules packaged as `.lgx` files.

```text
┌─────────────────────────────────────────────────────────┐
│  Transparent SACCO                                       │
│  • programs/sacco_ledger      — LEZ program (local)      │
│  • modules/sacco_core         — core module (IDL calls)  │
│  • modules/sacco_ui           — QML UI module (3 views)  │
└───────────────────────────┬─────────────────────────────┘
                            │
            ┌───────────────┴───────────────┐
            ▼                               ▼
   logos-standalone-app              Logos Basecamp
   (focused desktop app)             (shell + Package Manager)
```

| Layer | Technology |
|-------|------------|
| Execution | Logos Execution Zone (LEZ) / LEE |
| On-chain logic | LEZ program (Rust guest, RISC Zero) — attestation model, no custody |
| App runtime | Logos Core modules (`.lgx`) |
| UI | QML UI module (`logos-standalone-app` or Basecamp) |
| Distribution | `logos-repo.json` catalogue + release index |

Docs: [docs.logos.co](https://docs.logos.co/) · [Basecamp](https://logos.co/basecamp)

---

## Status

Progress against the [pivot roadmap](./docs/roadmap-sacco-pivot.md). A phase is marked done only when its issue is closed and its work is merged on `main`; the roadmap's phase checklists carry the task-level detail.

| Phase | Steps | Status | Issue |
|---|---|---|---|
| 0 · Scope and validate | 1 | Partly done. Field research, workstreams and the three-flow scope (deposits, loans, expenses) are settled. The pilot SACCO agreement and the baseline trust survey are still outstanding; the pilot kit (agreement template, metrics and survey, workstream owners) is under review in [PR #19](https://github.com/LOGOS-UGANDA/transparent-sacco/pull/19) and is not on `main`. | [#2](https://github.com/LOGOS-UGANDA/transparent-sacco/issues/2) (closed) |
| 1 · Domain model and LEZ program | 2–4 | Done. `sacco_ledger` runs all 19 IDL instructions on `lee_core` v0.3.1 with per-instruction tests, a generated IDL and CI. | [#3](https://github.com/LOGOS-UGANDA/transparent-sacco/issues/3) (closed) |
| 2 · Upgrade and deploy | 4 | Done. Deployed to a local sequencer with `logos-scaffold` and a successful `Initialize` call recorded in [docs/deploy.md](./docs/deploy.md). Public testnet deployment is deferred by decision. | [#4](https://github.com/LOGOS-UGANDA/transparent-sacco/issues/4) (closed) |
| 3 · Core and UI wiring | 5–7 | Done. `sacco_core` submits IDL instructions through the LEZ wallet; Member, Committee and Auditor views run in Basecamp; the demo flow is captured in [docs/screenshots/issue-5/](./docs/screenshots/issue-5/). | [#5](https://github.com/LOGOS-UGANDA/transparent-sacco/issues/5) (closed) |
| 4 · Privacy | 7–8 | Tracked by the phase owners in the issue. | [#6](https://github.com/LOGOS-UGANDA/transparent-sacco/issues/6) (open) |
| 5 · Storage and Messaging | 8–9 | Tracked by the phase owners in the issue. | [#7](https://github.com/LOGOS-UGANDA/transparent-sacco/issues/7) (open) |
| 6 · Package and publish | 9 | Tracked by the phase owners in the issue. | [#8](https://github.com/LOGOS-UGANDA/transparent-sacco/issues/8) (open) |
| 7 · Pilot and report | 10–12 | Tracked by the phase owners in the issue. | [#9](https://github.com/LOGOS-UGANDA/transparent-sacco/issues/9) (open) |

## Roadmap

The [pivot roadmap](./docs/roadmap-sacco-pivot.md) carries the full plan: 8 phases across 12 steps — domain model, deploy, core/UI wiring, privacy (private member accounts), storage and messaging (receipts by CID, member alerts), packaging, and a real-SACCO pilot with published metrics. Work is tracked as phase issues; see [CONTRIBUTING.md](./CONTRIBUTING.md) before picking one up.

## Getting started

### 1. Install Basecamp (recommended)

Download from [logos.co/basecamp](https://logos.co/basecamp) and use it to explore the stack (wallet, LEZ, Package Manager). The SACCO modules build and install into Basecamp locally; see [docs/testing.md](./docs/testing.md).

### 2. Work on the program

```bash
cd programs
RISC0_DEV_MODE=1 cargo test --workspace
```

See [docs/testing.md](./docs/testing.md) for the module and Basecamp checks, [docs/deploy.md](./docs/deploy.md) for the local deployment, and the [Status](#status) table for what's next.

### 3. Explore the earlier web prototype (optional, legacy)

The React + Hardhat demo illustrates the original fee-transparency flows. It predates the pivot and is **not** the Logos production path.

```bash
cd prototype && npm install && npm run dev
```

---

## Repository layout

```text
.
├── README.md
├── LICENSE                     # MIT license
├── CONTRIBUTING.md             # ground rules for contributors and AI agents
├── logos-repo.json             # Basecamp package-repository entry (filled at Phase 6)
├── docs/
│   ├── roadmap-sacco-pivot.md  # plan of record
│   ├── architecture.md · idl.md · testing.md · deploy.md · basecamp-publishing.md
│   ├── screenshots/            # Basecamp demo captures (Phase 3)
│   ├── custody.md              # superseded proposal, historical only
│   └── superpowers/            # dated design history; older proposals may be superseded
├── artifacts/                  # program IDL
├── programs/                   # LEZ program: sacco_ledger (attestation ledger)
├── modules/
│   ├── sacco_core/             # headless core module, submits IDL instructions to LEZ
│   └── sacco_ui/               # QML UI module: Member, Committee and Auditor views
└── prototype/                  # superseded React + Hardhat custody demo — reference only
```

## Contributing

Contributions are welcome — from humans and AI coding agents alike. Read [CONTRIBUTING.md](./CONTRIBUTING.md) first: work is organized as phase issues, branches are named `phase<N>/<short-topic>`, commits follow [conventional commits](https://www.conventionalcommits.org/), and nothing lands on `main` without pull-request review.

## Research anchors

- **Makerere University** — 45,000+ students; the original fee-transparency study site
- **Ugandan SACCO landscape** — Wazalendo, Parliamentary SACCO, KACITA, Yetu, Bugongi, Obulamu and others informed the pivot (membership patterns, governance separation, the deceased-members recovery problem); see the roadmap's open questions

## License

Released under the [MIT License](./LICENSE).
