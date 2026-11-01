# `sacco_core` and `sacco_ledger` API (issue #5)

This contract follows the enum order and account metadata in
[`artifacts/sacco_ledger-idl.json`](../../artifacts/sacco_ledger-idl.json).
Account handles are supplied in the listed order. `*` means the wallet must
authorize that account; `+` means it must also be an unused account authorized
to initialize the program shard. Args use the exact IDL field names and types.

## Instructions

| Core operation | LEZ instruction | Ordered accounts | Args |
|---|---|---|---|
| `initialize` | `Initialize` | `config+` | — |
| `registerSacco` | `RegisterSacco` | `sacco+*`, `admin*` | `name`, `registration_number`, `treasury` |
| `verifySacco` | `VerifySacco` | `sacco`, `admin*` | — |
| `addApprover` | `AddApprover` | `sacco`, `admin*` | `approver` |
| `addSupervisor` | `AddSupervisor` | `sacco`, `admin*` | `supervisor` |
| `addMember` | `AddMember` | `sacco`, `member+*`, `admin*` | `identity` |
| `recordDeposit` | `RecordDeposit` | `sacco`, `member_account`, `deposit+*`, `recorder*` | `member`, `amount`, `deposit_type`, `reference` |
| `confirmDeposit` | `ConfirmDeposit` | `deposit`, `member_account`, `member_signer*` | — |
| `applyLoan` | `ApplyLoan` | `sacco`, `member_account`, `loan+*`, `member_signer*` | `amount`, `purpose`, `required_approvals` |
| `approveLoan` | `ApproveLoan` | `sacco`, `loan`, `approver*` | — |
| `rejectLoan` | `RejectLoan` | `sacco`, `loan`, `approver*` | — |
| `disburseLoan` | `DisburseLoan` | `sacco`, `member_account`, `loan`, `approver*` | `evidence_cid`, `principal` |
| `recordRepayment` | `RecordRepayment` | `sacco`, `member_account`, `loan_account`, `approver*` | `loan`, `amount`, `reference` |
| `proposeExpense` | `ProposeExpense` | `sacco`, `expense+*`, `proposer*` | `amount`, `purpose`, `evidence_cid`, `required_approvals` |
| `approveExpense` | `ApproveExpense` | `sacco`, `expense`, `approver*` | — |
| `rejectExpense` | `RejectExpense` | `sacco`, `expense`, `approver*` | — |
| `executeExpense` | `ExecuteExpense` | `sacco`, `expense`, `approver*` | `executed_evidence`, `amount` |
| `postPeriodSummary` | `PostPeriodSummary` | `sacco`, `summary+*`, `poster*` | `label`, `totals` |
| `setRequiredApprovals` | `SetRequiredApprovals` | `sacco`, `admin*` | `required_approvals` |

The exact field types, enum variants, and writable flags are defined by the IDL.
For example, `deposit_type` is `Savings` or `ShareCapital`; amounts are `u128`;
and `totals` contains the five `u128` summary fields. Approval thresholds are
pinned into loan and expense records when they are created. An expense proposer
cannot approve their own proposal; the required number of other approvers must
approve it before execution. Rejecting approvers veto the record.

## Local Rust wallet transport

The repository's LEZ v0.3.1 Rust client can submit an instruction to the
configured local sequencer:

```text
sacco_ledger_local_client call <program-id> <fee-payer-id> <accounts-json> <instruction-json>
```

`accounts-json` is an ordered array of `{ "id": "<base58-account-id>",
"signer": true|false }` objects. `instruction-json` is a serde representation
of the `Instruction` enum, for example
`{"ProposeExpense":{"amount":100,"purpose":"supplies","evidence_cid":"cid","required_approvals":2}}`.
The wallet signs only with account keys it owns. The program validates the
account count, order, initialization, and signer requirements; the IDL is the
source of truth for those checks. This client is the local LEZ transport and
verification path. Inside Basecamp, the C++ module's `dispatchInstruction()`
uses the same Rust IDL code through `programs/sacco_ledger/ffi` to encode the
instruction and resolve its account table, then submits it through the
configured LEZ wallet (`lez_core`).

The LEZ wallet accepts one mention per account. When `member_account` and
`member_signer` in `ConfirmDeposit` are the same account, the core merges them
into one signing entry; the program accepts that two-account form as well as
the three-account form.

## Reads and current limits

The Rust client can read and Borsh-decode a known account shard:

```text
sacco_ledger_local_client read <sacco|member|deposit|loan|expense|summary|config> <program-id> <account-id>
```

This is a direct read from the configured LEZ sequencer. In the C++ module,
the single-account queries (`getSacco`, `getMember`, `getDeposit`, `getLoan`,
`getExpense`) read the public account through the LEZ wallet's sequencer query
and decode its shard with the Rust IDL code.

The ledger has no account-discovery endpoint, and member shards do not retain
deposit account IDs. The list queries (`listMyDeposits`, `listMyLoans`,
`listPeriodSummaries`, `getAuditTrail`) therefore return the account's
transaction history from `lez_indexer_module` rather than decoded record lists;
decoding a specific record still needs its account ID.
