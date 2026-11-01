# sacco_ui

Logos **UI module** (QML) for Transparent SACCO.

The product records off-chain deposits, loans, and expenses on LEZ. SACCO
money is not deposited into or transferred by this application.

## Role

Desktop UI hosted by Basecamp or `logos-standalone-app`. Calls `sacco_core` through the Logos bridge (`logos.callModule`) — QML stays sandboxed (no direct network).

## Screens

`qml/Main.qml` sets the ledger context (program, SACCO, fee payer and signer accounts) and hosts three role tabs:

- **Member** (`MemberView.qml`): SACCO totals, the member's savings and share capital, loading and confirming a deposit, the member's loans, and applying for a loan
- **Committee** (`CommitteeView.qml`): record deposits, propose expenses, approve or reject loans and expenses, set the approval threshold, execute expenses, and record loan disbursements and repayments
- **Auditor** (`AuditorView.qml`): the audit trail, read as the SACCO account's transaction history from the indexer

Every action goes through `sacco_core`, which submits the matching `sacco_ledger` instruction. The Phase 3 demo flow is captured in [`docs/screenshots/issue-5/`](../../docs/screenshots/issue-5/). The React/Hardhat `prototype/` is a superseded custody demo, not a source for current product behavior.

## Build / preview

```bash
cd modules/sacco_ui
# After nix flake lock resolves local sacco_core:
nix run                 # preview via logos-standalone-app
nix build .#lgx         # package for Basecamp
```

Declare `sacco_core` as a dependency in `metadata.json` (already set). Point the flake input at a published GitHub URL when you stop using `path:../sacco_core`.
