# sacco_core

Headless Logos core module for the Transparent SACCO attestation ledger. The product records off-chain deposits, loan activity, and expenses; it never receives, holds, or transfers SACCO money.

The module contract is documented in [API.md](API.md). The C++ module encodes every `sacco_ledger` instruction through the Rust IDL client (`programs/sacco_ledger/ffi`) and submits it through the configured LEZ wallet (`lez_core`). Single-account queries read the live shard from the sequencer and decode it; list queries return the account's transaction history from `lez_indexer_module`. The Basecamp UI uses these calls for the Phase 3 demo flow (record a deposit, the member confirms it, a multi-sig expense is executed); see [`docs/screenshots/issue-5/`](../../docs/screenshots/issue-5/).

## Build

Requires Nix + [`logos-module-builder`](https://github.com/logos-co/logos-module-builder).

```bash
cd modules/sacco_core
nix build .#lgx
nix build .#lgx-portable
```

The tested Basecamp path builds and installs the modules declared in the root `scaffold.toml` (`sacco_core`, `sacco_ui`, and the `lez_core` and `lez_indexer_module` dependencies):

```bash
lgs basecamp build --variant lgx
lgs basecamp install
```

Install and preview steps: [docs/basecamp-publishing.md](../../docs/basecamp-publishing.md).
