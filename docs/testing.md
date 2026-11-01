# Testing Transparent SACCO

What you can verify today, and how Basecamp fits in.

## Reality check

| Layer | Testable now? | What “working” means |
|-------|---------------|----------------------|
| Legacy UX prototype (`prototype/`) | Yes, as a demo only | Old Hardhat fee-custody flows; does not test the current SACCO ledger |
| LEZ program (host handlers) | Yes | `RISC0_DEV_MODE=1 cargo test --workspace` in `programs/` passes; CI runs it with clippy and the IDL drift check |
| LEZ program on-chain | Yes, on a local sequencer | Guest built, deployed with `logos-scaffold`, and `Initialize` verified; see [deploy.md](./deploy.md) |
| `sacco_core` / `sacco_ui` in Basecamp | Yes, local `.lgx` | `lgs basecamp build --variant lgx` and `lgs basecamp install` build and install the modules declared in `scaffold.toml` |
| SACCO ledger flow | Yes, the Phase 3 demo flow | In Basecamp: record a deposit, the member sees and confirms it, an expense passes multi-sig and is executed; captured in [screenshots/issue-5/](./screenshots/issue-5/) |
| Private member accounts | Not yet | Phase 4 ([issue #6](https://github.com/LOGOS-UGANDA/transparent-sacco/issues/6)) |
| Published install from a package repository | Not yet | Phase 6 ([issue #8](https://github.com/LOGOS-UGANDA/transparent-sacco/issues/8)) |

Basecamp is where the product UI and modules are exercised, and it is also useful for learning the stack (wallet, LEZ, Package Manager).

---

## A. Use Basecamp now (stack rehearsal)

1. Install from [logos.co/basecamp](https://logos.co/basecamp).
2. Open Basecamp and confirm bundled modules load (Package Manager / Modules).
3. Optionally create an **LEZ wallet account** to learn how accounts and signing work. Token transfers and custom-token exercises are generic LEZ features, not Transparent SACCO flows: this project records off-chain deposits, loans and expenses and does not hold or transfer SACCO funds.
4. Open the **LEZ explorer / blockchain dashboard** if present — this is the environment `sacco_ledger` will target.
5. Optionally try a third-party module from a community repo:
   **Settings → Package Repositories → Add** a URL from the [forum thread](https://forum.logos.co/t/basecamp-module-repository-links/1843), then install from Applications.

This does **not** test Transparent SACCO itself; it proves your Basecamp + LEZ environment works.

---

## B. Explore the legacy web prototype

This React + Hardhat demo predates the SACCO attestation model. It simulates fee custody and transfers and is not the current product flow or a test of `sacco_ledger`.

```bash
cd prototype
npm install
npm run dev
```

Walk through Dashboard → Institutions → Pay Fees → Verify → Releases to inspect the historical demo only.

---

## C. Test the LEZ program (CLI / host)

Requires Rust + Logos LEZ dependencies (see [lez-programs](https://github.com/logos-blockchain/lez-programs)).

```bash
cd programs
RISC0_DEV_MODE=1 cargo test -p sacco_ledger_program
```

The program can be built and deployed using the steps in [`sacco_ledger/README.md`](../programs/sacco_ledger/README.md) and [deploy.md](./deploy.md). The repository's local Rust client (`programs/sacco_ledger/client`) submits IDL instructions and reads account shards; see [`modules/sacco_core/API.md`](../modules/sacco_core/API.md). These calls record and approve off-chain SACCO activity; they do not deposit or transfer SACCO funds.

Use the local sequencer for development. The current local workflow needs no public testnet wallet or test tokens. Public testnet deployment is deferred, is not needed to build the product, and is not part of the current test steps. See [architecture.md](./architecture.md), the [superseded custody proposal](./custody.md), and [idl.md](./idl.md).

---

## D. Test modules outside Basecamp (recommended first)

### UI preview (standalone)

```bash
cd modules/sacco_ui
nix run    # logos-standalone-app preview
```

Expect: the ledger-context form and the Member, Committee and Auditor tabs. Core calls fail unless `sacco_core` and its `lez_core` and `lez_indexer_module` dependencies are installed alongside; the Basecamp path in section E installs them together.

### Core via `logosctl`

```bash
cd modules/sacco_core
nix build .#lgx-portable
logosctl daemon start --detach
logosctl package install --file ./result/*.lgx --yes
logosctl module load sacco_core
logosctl call sacco_core getStatus
logosctl daemon stop
```

Use `getStatus` to verify module loading. Ledger methods need a configured wallet and ledger context (`setLedgerContext`) and the `lez_core` dependency; see [`modules/sacco_core/API.md`](../modules/sacco_core/API.md).

Official walkthrough: [Build and run a Logos core module](https://docs.logos.co/core/build-modules/build-and-run-a-logos-core-module).

---

## E. Test in Basecamp

The path used for the Phase 3 demo is `lgs basecamp build --variant lgx` followed by `lgs basecamp install`, which builds the modules declared in `scaffold.toml` and installs them into the local Basecamp profiles. The manual steps below install the same packages by hand.

### Prerequisites

- Nix with flakes (`experimental-features = nix-command flakes`)
- Basecamp installed (portable download or Nix build)
- Matching LGX variant:
  - Dev Basecamp → need `-dev` variant (often via dual bundler)
  - Portable Basecamp → portable `.lgx`

### 1. Build packages

```bash
cd modules/sacco_core
nix build .#lgx
# Prefer dual bundle for Basecamp flexibility:
# nix bundle --bundler github:logos-co/nix-bundle-lgx/tutorial-v3#dual .#lib

cd ../sacco_ui
nix build .#lgx
```

### 2. Install into Basecamp

In Basecamp, use **Package Manager → Install LGX Package** and select the `.lgx` files.

### 3. Load and smoke-test

1. Restart Basecamp (or use Modules view → Load).
2. Confirm `sacco_core` under Core Modules and `sacco_ui` under UI Modules / Applications.
3. Open Transparent SACCO UI.
4. Set the ledger context (program, SACCO, fee payer and signer accounts) against the local sequencer from [deploy.md](./deploy.md).

### 4. Pass criteria (Phase 3 demo flow)

- Modules install without variant errors and the UI opens with the Member, Committee and Auditor tabs
- Committee tab: `RecordDeposit` saves a savings deposit
- Member tab: the deposit is visible and **Confirm deposit** marks it confirmed
- Committee tab: an expense is proposed, approved by the required approvers, and executed; its status shows the approval count
- Deposits and expenses are readable as public accounts on the local sequencer

These were last verified for the Phase 3 demo; see [screenshots/issue-5/](./screenshots/issue-5/). Private member balances (Phase 4) are not covered yet.

---

## F. Discovery test (published repo — later)

When `logos-repo.json` has a real `indexUrl`:

1. Basecamp → **Settings → Package Repositories → Add** your repo URL
2. Install Transparent SACCO from Applications
3. Same smoke tests as section E

---

## Suggested order

1. Basecamp stack rehearsal (A)
2. Prototype UX (B) if you care about screens
3. LEZ program tests + local deploy (C)
4. Package + install modules in Basecamp and run the demo flow (E)

The product records off-chain activity; do not add SACCO fund custody or transfers.
