//! Instruction planning: assert what metadata can prove, then name one effect per shard.

use nssa_core::program::{AccountMeta, Plan, PlanInput};
use sacco_ledger_core::effect::{
    ConfigEffect, DepositEffect, Effect, ExpenseEffect, LoanEffect, MemberEffect, SaccoEffect,
    SummaryEffect,
};
use sacco_ledger_core::{compute_config_pda, Deposit, Instruction};

/// The registry layout version this build writes and expects.
pub const CONFIG_VERSION: u8 = 1;

/// Reads an account's signer flag, panicking with the instruction's own rule text.
#[must_use]
pub fn signer(account: &AccountMeta, rule: &str) -> nssa_core::account::AccountId {
    assert!(account.is_authorized, "{rule}");
    account.account_id
}

/// Builds the effect plan for one instruction.
pub fn plan(input: &PlanInput, instruction: Instruction) -> Plan {
    let mut plan = Plan::new(input);
    match instruction {
        Instruction::Initialize => {
            let [config] = <&[_; 1]>::try_from(input.accounts.as_slice())
                .expect("Initialize requires exactly 1 account");
            assert_eq!(
                config.account_id,
                compute_config_pda(input.self_account_id),
                "Config account must match the program-derived address"
            );
            plan.effect(
                config,
                &Effect::Config(ConfigEffect::Init {
                    version: CONFIG_VERSION,
                }),
            );
        }
        Instruction::RegisterSacco {
            name,
            registration_number,
            treasury,
        } => {
            let [sacco, admin] = <&[_; 2]>::try_from(input.accounts.as_slice())
                .expect("RegisterSacco requires sacco and admin");
            assert!(sacco.is_authorized, "Sacco account must be authorized");
            let admin = signer(admin, "Admin must authorize registration");
            plan.effect(
                sacco,
                &Effect::Sacco(SaccoEffect::Register {
                    name,
                    registration_number,
                    treasury,
                    admin,
                }),
            );
        }
        Instruction::VerifySacco => {
            let [sacco, admin] = <&[_; 2]>::try_from(input.accounts.as_slice())
                .expect("VerifySacco requires sacco and admin");
            let admin = signer(admin, "Admin must authorize verification");
            plan.effect(sacco, &Effect::Sacco(SaccoEffect::Verify { admin }));
        }
        Instruction::AddApprover { approver } => {
            let [sacco, admin] = <&[_; 2]>::try_from(input.accounts.as_slice())
                .expect("AddApprover requires sacco and admin");
            let admin = signer(admin, "Admin must authorize");
            plan.effect(
                sacco,
                &Effect::Sacco(SaccoEffect::AddApprover { admin, approver }),
            );
        }
        Instruction::SetRequiredApprovals { required_approvals } => {
            let [sacco, admin] = <&[_; 2]>::try_from(input.accounts.as_slice())
                .expect("SetRequiredApprovals requires sacco and admin");
            let admin = signer(admin, "Admin must authorize");
            plan.effect(
                sacco,
                &Effect::Sacco(SaccoEffect::SetRequiredApprovals {
                    admin,
                    required_approvals,
                }),
            );
        }
        Instruction::AddSupervisor { supervisor } => {
            let [sacco, admin] = <&[_; 2]>::try_from(input.accounts.as_slice())
                .expect("AddSupervisor requires sacco and admin");
            let admin = signer(admin, "Admin must authorize");
            plan.effect(
                sacco,
                &Effect::Sacco(SaccoEffect::AddSupervisor { admin, supervisor }),
            );
        }
        Instruction::AddMember { identity } => {
            let [sacco, member, admin] = <&[_; 3]>::try_from(input.accounts.as_slice())
                .expect("AddMember requires sacco, member and admin");
            assert!(member.is_authorized, "Member account must be authorized");
            let admin = signer(admin, "Admin must authorize");
            plan.effect(
                sacco,
                &Effect::Sacco(SaccoEffect::AddMember { admin, identity }),
            );
            plan.effect(
                member,
                &Effect::Member(MemberEffect::Init {
                    sacco: sacco.account_id,
                    identity,
                }),
            );
        }
        Instruction::RecordDeposit {
            member,
            amount,
            deposit_type,
            reference,
        } => {
            let [sacco, member_account, deposit, recorder] =
                <&[_; 4]>::try_from(input.accounts.as_slice())
                    .expect("RecordDeposit requires sacco, member, deposit and recorder");
            assert!(deposit.is_authorized, "Deposit account must be authorized");
            assert_eq!(
                member_account.account_id, member,
                "Member account does not match the member id argument"
            );
            let recorder = signer(recorder, "Recorder must authorize");

            plan.effect(
                sacco,
                &Effect::Sacco(SaccoEffect::RecordDeposit {
                    recorder,
                    amount,
                    deposit_type,
                }),
            );
            plan.effect(
                member_account,
                &Effect::Member(MemberEffect::RecordDeposit {
                    sacco: sacco.account_id,
                    recorder,
                    amount,
                    deposit_type,
                }),
            );
            plan.effect(
                deposit,
                &Effect::Deposit(DepositEffect::Init(Deposit {
                    sacco_id: sacco.account_id,
                    member_id: member,
                    amount,
                    deposit_type,
                    reference,
                    recorded_by: recorder,
                    confirmed_by_member: false,
                })),
            );
        }
        Instruction::ConfirmDeposit => {
            // The LEZ wallet API accepts one mention per account identity. If
            // the member ledger account is the member's identity account, the
            // core merges those two IDL entries and promotes that mention to a
            // signer. Accept that two-account form and use the same authorized
            // account for both roles.
            let (deposit, member_account, member_signer) = match input.accounts.as_slice() {
                [deposit, member] => (deposit, member, member),
                [deposit, member, signer] => (deposit, member, signer),
                _ => panic!("ConfirmDeposit requires deposit, member and member signer"),
            };
            let signer = signer(member_signer, "Member must authorize confirmation");
            plan.effect(
                member_account,
                &Effect::Member(MemberEffect::AuthorizeIdentity { signer }),
            );
            plan.effect(
                deposit,
                &Effect::Deposit(DepositEffect::Confirm {
                    member: member_account.account_id,
                }),
            );
        }
        Instruction::ApplyLoan {
            amount,
            purpose,
            required_approvals,
        } => {
            let [sacco, member, loan, member_signer] =
                <&[_; 4]>::try_from(input.accounts.as_slice())
                    .expect("ApplyLoan requires sacco, member, loan and member signer");
            assert!(loan.is_authorized, "Loan account must be authorized");
            let signer = signer(member_signer, "Member must authorize the application");
            plan.effect(
                sacco,
                &Effect::Sacco(SaccoEffect::AuthorizeProposal { required_approvals }),
            );
            plan.effect(
                member,
                &Effect::Member(MemberEffect::AttachLoan {
                    sacco: sacco.account_id,
                    loan: loan.account_id,
                    signer,
                }),
            );
            plan.effect(
                loan,
                &Effect::Loan(LoanEffect::Init {
                    sacco: sacco.account_id,
                    member: member.account_id,
                    applicant: signer,
                    principal: amount,
                    purpose,
                    required_approvals,
                }),
            );
        }
        Instruction::ApproveLoan => {
            let [sacco, loan, approver] = <&[_; 3]>::try_from(input.accounts.as_slice())
                .expect("ApproveLoan requires sacco, loan and approver");
            let approver = signer(approver, "Approver must authorize");
            plan.effect(
                sacco,
                &Effect::Sacco(SaccoEffect::AuthorizeApprover { approver }),
            );
            plan.effect(
                loan,
                &Effect::Loan(LoanEffect::Approve {
                    sacco: sacco.account_id,
                    approver,
                }),
            );
        }
        Instruction::RejectLoan => {
            let [sacco, loan, approver] = <&[_; 3]>::try_from(input.accounts.as_slice())
                .expect("RejectLoan requires sacco, loan and approver");
            let approver = signer(approver, "Approver must authorize");
            plan.effect(
                sacco,
                &Effect::Sacco(SaccoEffect::AuthorizeApprover { approver }),
            );
            plan.effect(
                loan,
                &Effect::Loan(LoanEffect::Reject {
                    sacco: sacco.account_id,
                    approver,
                }),
            );
        }
        Instruction::DisburseLoan {
            evidence_cid,
            principal,
        } => {
            // Attestation model: outflows are not checked against recorded
            // deposits; the ledger records the off-chain transfer evidence.
            let [sacco, member, loan, approver] = <&[_; 4]>::try_from(input.accounts.as_slice())
                .expect("DisburseLoan requires sacco, member, loan and approver");
            let approver = signer(approver, "Approver must authorize");
            plan.effect(
                sacco,
                &Effect::Sacco(SaccoEffect::DisburseLoan {
                    approver,
                    principal,
                }),
            );
            plan.effect(
                loan,
                &Effect::Loan(LoanEffect::Disburse {
                    sacco: sacco.account_id,
                    member: member.account_id,
                    evidence_cid,
                    principal,
                }),
            );
        }
        Instruction::RecordRepayment {
            loan,
            amount,
            reference,
        } => {
            let [sacco, member, loan_account, approver] =
                <&[_; 4]>::try_from(input.accounts.as_slice())
                    .expect("RecordRepayment requires sacco, member, loan and approver");
            assert_eq!(
                loan_account.account_id, loan,
                "Loan account does not match the loan id argument"
            );
            let approver = signer(approver, "Approver must authorize");
            plan.effect(
                sacco,
                &Effect::Sacco(SaccoEffect::RecordRepayment { approver, amount }),
            );
            plan.effect(
                loan_account,
                &Effect::Loan(LoanEffect::Repay {
                    sacco: sacco.account_id,
                    member: member.account_id,
                    amount,
                    reference,
                }),
            );
        }
        Instruction::ProposeExpense {
            amount,
            purpose,
            evidence_cid,
            required_approvals,
        } => {
            let [sacco, expense, proposer] = <&[_; 3]>::try_from(input.accounts.as_slice())
                .expect("ProposeExpense requires sacco, expense and proposer");
            assert!(expense.is_authorized, "Expense account must be authorized");
            let proposer = signer(proposer, "Proposer must authorize");
            plan.effect(
                sacco,
                &Effect::Sacco(SaccoEffect::ProposeExpense {
                    proposer,
                    amount,
                    required_approvals,
                }),
            );
            plan.effect(
                expense,
                &Effect::Expense(ExpenseEffect::Init {
                    sacco: sacco.account_id,
                    amount,
                    purpose,
                    evidence_cid,
                    requested_by: proposer,
                    required_approvals,
                }),
            );
        }
        Instruction::ApproveExpense => {
            let [sacco, expense, approver] = <&[_; 3]>::try_from(input.accounts.as_slice())
                .expect("ApproveExpense requires sacco, expense and approver");
            let approver = signer(approver, "Approver must authorize");
            plan.effect(
                sacco,
                &Effect::Sacco(SaccoEffect::AuthorizeApprover { approver }),
            );
            plan.effect(
                expense,
                &Effect::Expense(ExpenseEffect::Approve {
                    sacco: sacco.account_id,
                    approver,
                }),
            );
        }
        Instruction::RejectExpense => {
            let [sacco, expense, approver] = <&[_; 3]>::try_from(input.accounts.as_slice())
                .expect("RejectExpense requires sacco, expense and approver");
            let approver = signer(approver, "Approver must authorize");
            plan.effect(
                sacco,
                &Effect::Sacco(SaccoEffect::AuthorizeApprover { approver }),
            );
            plan.effect(
                expense,
                &Effect::Expense(ExpenseEffect::Reject {
                    sacco: sacco.account_id,
                    approver,
                }),
            );
        }
        Instruction::ExecuteExpense {
            executed_evidence,
            amount,
        } => {
            // Attestation model: expense totals are not capped by deposits;
            // execution records the off-chain payment reference only.
            let [sacco, expense, approver] = <&[_; 3]>::try_from(input.accounts.as_slice())
                .expect("ExecuteExpense requires sacco, expense and approver");
            let approver = signer(approver, "Approver must authorize");
            plan.effect(
                sacco,
                &Effect::Sacco(SaccoEffect::ExecuteExpense { approver, amount }),
            );
            plan.effect(
                expense,
                &Effect::Expense(ExpenseEffect::Execute {
                    sacco: sacco.account_id,
                    executed_evidence,
                    amount,
                }),
            );
        }
        Instruction::PostPeriodSummary { label, totals } => {
            let [sacco, summary, poster] = <&[_; 3]>::try_from(input.accounts.as_slice())
                .expect("PostPeriodSummary requires sacco, summary and poster");
            assert!(summary.is_authorized, "Summary account must be authorized");
            let poster = signer(poster, "Poster must authorize");
            plan.effect(
                sacco,
                &Effect::Sacco(SaccoEffect::HoldTotals {
                    label: label.clone(),
                    totals,
                    poster,
                }),
            );
            plan.effect(
                summary,
                &Effect::Summary(SummaryEffect::Init {
                    sacco: sacco.account_id,
                    label,
                    totals,
                }),
            );
        }
    }
    let mut effected_accounts = Vec::new();
    for effect in &plan.output().effects {
        assert!(
            !effected_accounts.contains(&effect.selector.account_id),
            "Instruction cannot apply multiple effects to the same account"
        );
        effected_accounts.push(effect.selector.account_id);
    }
    plan
}
