//! IDL signer declarations must match the plan's authorization checks.

use nssa_core::account::AccountId;
use nssa_core::program::{AccountMeta, PlanInput};
use sacco_ledger_core::effect::{
    ConfigEffect, DepositEffect, Effect, ExpenseEffect, LoanEffect, MemberEffect, SaccoEffect,
    SummaryEffect,
};
use sacco_ledger_core::{compute_config_pda, DepositType, Instruction, Totals};
use sacco_ledger_idl::INSTRUCTIONS;
use sacco_ledger_program::plan;

const PROGRAM: AccountId = AccountId::new([0xAA; 32]);

#[test]
fn initialize_config_pda_is_not_a_signer() {
    assert!(!INSTRUCTIONS[0].accounts[0].signer);
}

#[test]
fn confirm_deposit_marks_only_the_member_identity_as_signer() {
    let instruction = INSTRUCTIONS
        .iter()
        .find(|instruction| instruction.name == "confirm_deposit")
        .expect("ConfirmDeposit IDL entry");
    let accounts = instruction.accounts;

    assert_eq!(accounts.len(), 3);
    assert_eq!(accounts[0].name, "deposit");
    assert!(accounts[0].writable);
    assert!(!accounts[0].signer);
    assert_eq!(accounts[1].name, "member_account");
    assert!(!accounts[1].writable);
    assert!(!accounts[1].signer);
    assert_eq!(accounts[2].name, "member_signer");
    assert!(!accounts[2].writable);
    assert!(accounts[2].signer);
}

fn samples() -> Vec<Instruction> {
    let id = AccountId::new([7; 32]);
    vec![
        Instruction::Initialize,
        Instruction::RegisterSacco {
            name: "n".into(),
            registration_number: "r".into(),
            treasury: id,
        },
        Instruction::VerifySacco,
        Instruction::AddApprover { approver: id },
        Instruction::AddSupervisor { supervisor: id },
        Instruction::AddMember { identity: id },
        Instruction::RecordDeposit {
            member: id,
            amount: 1,
            deposit_type: DepositType::Savings,
            reference: "r".into(),
        },
        Instruction::ConfirmDeposit,
        Instruction::ApplyLoan {
            amount: 1,
            purpose: "p".into(),
            required_approvals: 1,
        },
        Instruction::ApproveLoan,
        Instruction::RejectLoan,
        Instruction::DisburseLoan {
            evidence_cid: "c".into(),
            principal: 1,
        },
        Instruction::RecordRepayment {
            loan: id,
            amount: 1,
            reference: "r".into(),
        },
        Instruction::ProposeExpense {
            amount: 1,
            purpose: "p".into(),
            evidence_cid: "c".into(),
            required_approvals: 1,
        },
        Instruction::ApproveExpense,
        Instruction::RejectExpense,
        Instruction::ExecuteExpense {
            executed_evidence: "e".into(),
            amount: 1,
        },
        Instruction::PostPeriodSummary {
            label: "l".into(),
            totals: Totals::default(),
        },
        Instruction::SetRequiredApprovals {
            required_approvals: 2,
        },
    ]
}

#[test]
fn idl_account_flags_match_planned_effects_and_signer_authorization() {
    let samples = samples();
    assert_eq!(samples.len(), INSTRUCTIONS.len());

    for (index, (instruction, spec)) in samples.iter().zip(INSTRUCTIONS).enumerate() {
        let accounts: Vec<_> = spec
            .accounts
            .iter()
            .enumerate()
            .map(|(slot, account)| {
                let account_id = if spec.name == "initialize" && account.name == "config" {
                    compute_config_pda(PROGRAM)
                } else if spec.name == "record_deposit" && account.name == "member_account"
                    || spec.name == "record_repayment" && account.name == "loan_account"
                {
                    AccountId::new([7; 32])
                } else {
                    AccountId::new([(index * 16 + slot + 1) as u8; 32])
                };
                AccountMeta::new(account_id, account.signer, PROGRAM)
            })
            .collect();
        let input = PlanInput {
            self_account_id: PROGRAM,
            caller_account_id: None,
            accounts: accounts.clone(),
            instruction_data: borsh::to_vec(instruction).expect("instruction encodes"),
        };
        let planned = plan(&input, instruction.clone());

        for (slot, account) in spec.accounts.iter().enumerate() {
            let account_id = accounts[slot].account_id;
            let effect = planned
                .output()
                .effects
                .iter()
                .find(|effect| effect.selector.account_id == account_id)
                .map(|effect| {
                    borsh::from_slice::<Effect>(&effect.data).expect("planned effect decodes")
                });
            assert_eq!(
                account.writable,
                effect.as_ref().is_some_and(effect_writes_shard),
                "{} account {} writable flag must match its planned effect",
                spec.name,
                account.name
            );
            assert_eq!(
                account.init,
                effect.as_ref().is_some_and(effect_initializes_shard),
                "{} account {} init flag must match its planned effect",
                spec.name,
                account.name
            );
            if !account.signer {
                continue;
            }
            let mut unsigned_accounts = accounts.clone();
            unsigned_accounts[slot].is_authorized = false;
            let unsigned_input = PlanInput {
                accounts: unsigned_accounts,
                ..input.clone()
            };
            assert!(
                std::panic::catch_unwind(|| plan(&unsigned_input, instruction.clone())).is_err(),
                "{} account {} is declared signer but plan accepts it unsigned",
                spec.name,
                account.name
            );
        }
    }
}

fn effect_writes_shard(effect: &Effect) -> bool {
    match effect {
        Effect::Config(ConfigEffect::Init { .. }) => true,
        Effect::Sacco(effect) => !matches!(
            effect,
            SaccoEffect::AuthorizeProposal { .. }
                | SaccoEffect::AuthorizeApprover { .. }
                | SaccoEffect::ProposeExpense { .. }
        ),
        Effect::Member(effect) => !matches!(effect, MemberEffect::AuthorizeIdentity { .. }),
        Effect::Deposit(_) | Effect::Loan(_) | Effect::Expense(_) | Effect::Summary(_) => true,
    }
}

fn effect_initializes_shard(effect: &Effect) -> bool {
    match effect {
        Effect::Config(ConfigEffect::Init { .. })
        | Effect::Sacco(SaccoEffect::Register { .. })
        | Effect::Member(MemberEffect::Init { .. })
        | Effect::Deposit(DepositEffect::Init(_))
        | Effect::Loan(LoanEffect::Init { .. })
        | Effect::Expense(ExpenseEffect::Init { .. })
        | Effect::Summary(SummaryEffect::Init { .. }) => true,
        Effect::Sacco(
            SaccoEffect::Verify { .. }
            | SaccoEffect::AddApprover { .. }
            | SaccoEffect::AddSupervisor { .. }
            | SaccoEffect::AddMember { .. }
            | SaccoEffect::RecordDeposit { .. }
            | SaccoEffect::AuthorizeProposal { .. }
            | SaccoEffect::AuthorizeApprover { .. }
            | SaccoEffect::DisburseLoan { .. }
            | SaccoEffect::RecordRepayment { .. }
            | SaccoEffect::ProposeExpense { .. }
            | SaccoEffect::ExecuteExpense { .. }
            | SaccoEffect::HoldTotals { .. }
            | SaccoEffect::SetRequiredApprovals { .. },
        )
        | Effect::Member(
            MemberEffect::RecordDeposit { .. }
            | MemberEffect::AttachLoan { .. }
            | MemberEffect::AuthorizeIdentity { .. },
        )
        | Effect::Deposit(DepositEffect::Confirm { .. })
        | Effect::Loan(_)
        | Effect::Expense(_) => false,
    }
}
