# sacco_ledger (LEZ program)

Transparent SACCO attestation ledger on the Logos Execution Zone. Deposits,
loans and expenses are recorded and approved on chain; **value stays off chain**,
so the program holds no tokens and issues no transfers.

Runs on `lee_core` v0.3.1 using the sharded `plan` / `apply` ABI: `plan` maps an
instruction to one typed effect per mutated shard, and `apply` is the pure
per-shard transition. No effect can read another shard, so every value that
crosses a shard boundary is passed as an argument and validated by the shard
that owns the data.

## Layout

```text
sacco_ledger/
├── core/           # Instruction, Effect, record types, PDA helpers
├── src/            # plan() + apply() shims and per-shard transitions
├── idl/            # IDL table, generator and drift check
└── methods/        # risc0 guest binary
```

## Instructions

| Instruction | Purpose |
|-------------|---------|
| `Initialize` | Program config shard |
| `RegisterSacco` | Register a SACCO with its treasury |
| `VerifySacco` | Admin self-verifies the SACCO (v1) |
| `AddApprover` / `AddSupervisor` | Governance roles |
| `SetRequiredApprovals` | Admin sets a threshold from 2 through one less than the approver count |
| `AddMember` | Member account shard |
| `RecordDeposit` / `ConfirmDeposit` | Savings or share-capital deposit, then member confirmation (accepts the merged two-account form when the member account also signs) |
| `ApplyLoan` / `ApproveLoan` / `RejectLoan` | Loan application with pinned approval threshold |
| `DisburseLoan` | Marks the loan disbursed, carrying the principal for cross-checking |
| `RecordRepayment` | Repayment against a loan |
| `ProposeExpense` / `ApproveExpense` / `RejectExpense` | Expense proposal with pinned approval threshold |
| `ExecuteExpense` | Marks the expense executed, carrying the amount for cross-checking |
| `PostPeriodSummary` | Writes a period summary from caller-supplied totals |

Approval thresholds are pinned into `Loan` / `Expense` when they are created and
validated against the live SACCO config at that moment, because the approval
transition runs in a different shard from the config.

New SACCOs start with a threshold of one for setup. Verification is blocked
until the admin adds at least two more approvers and sets the threshold to at
least two. The upper bound leaves enough approvers to approve an expense after
excluding its proposer. Any non-proposer approver rejection is an immediate
veto. Period summary labels must use the canonical
`YYYY-MM` calendar-month format, and a month cannot be posted twice.
To keep Borsh shard sizes bounded in this release, each SACCO supports at most
128 members, 32 approvers and 32 supervisors, and each member can reference at
most 128 loans. Each SACCO can retain 128 unique period summaries. Approval and
rejection lists are bounded by the 32-approver limit. Larger deployments need
registry shards before these limits are raised.

## IDL

The JSON artifact is generated from the hand-maintained instruction table in
`idl/src/lib.rs`; it is not derived automatically from the Rust enum. CI fails
when the committed artifact is stale, and tests compare the table with the enum
and planner.

```bash
cd programs
cargo run -p sacco_ledger_idl -- --check
```

## Build / test

```bash
cd programs
RISC0_DEV_MODE=1 cargo test -p sacco_ledger_program
cargo test -p sacco_ledger_core -p sacco_ledger_idl
```

Guest ELF without Docker (the `risc0` rustup toolchain carries the target):

```bash
cd programs/sacco_ledger/methods/guest
cargo +risc0 build --release --target riscv32im-risc0-zkvm-elf
```

## Local deploy

The valid RISC Zero `ProgramBinary` is produced by building the methods crate.
For the pinned LEZ/SPEL/circuits pairing and the scaffold 0.4.x local block-size
workaround, follow [`docs/deploy.md`](../../docs/deploy.md).

```bash
RUSTFLAGS='--cfg getrandom_backend="custom"' cargo +risc0 build --release \
  --target riscv32im-risc0-zkvm-elf \
  --manifest-path programs/sacco_ledger/methods/guest/Cargo.toml
```

Follow the tested scaffold 0.4.0 workflow in [`docs/deploy.md`](../../docs/deploy.md)
for local setup, the 1 MiB block-size preparation, deployment, and Initialize
verification. The latest recorded local run deployed header/program account
`Ge4zuh3t2Kkg9y59YR9HCmaKwrqX18DvJSV7XRwg55eQ`; Initialize succeeded with
transaction `074afc038e148a98213f2b6f01a1e04ca38914045baeb0b51ad160ebaaa246fc`
and verified `ProgramConfig.version = 1` at config PDA
`GHmW3j89uVbH3NnLzGNUEzfuPLLCa1cBWZdRpczq6vXm`. Program account IDs are
deployment outputs and may change with guest builds; the runbook preserves the
commands and full result for this deployment.

## Known gaps

1. Member, deposit and loan records are still public accounts; moving them to
   private accounts is Phase 4 work
   ([issue #6](https://github.com/LOGOS-UGANDA/transparent-sacco/issues/6)).
2. Refunds are handled off chain; the current ledger has no instruction to record a refund yet.
