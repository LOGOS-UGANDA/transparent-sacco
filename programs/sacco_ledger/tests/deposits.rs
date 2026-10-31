//! Deposits: typed totals, verification gating, member confirmation.

mod support;

use sacco_ledger_core::{DepositType, Instruction};
use support::{add_verified_member, aid, meta, verified_sacco, Ledger};

fn record(ledger: &mut Ledger, deposit: u8) {
    ledger.execute(
        &[meta(aid(1)), meta(aid(6)), meta(aid(deposit)), meta(aid(2))],
        Instruction::RecordDeposit {
            member: aid(6),
            amount: 100_000,
            deposit_type: DepositType::Savings,
            reference: "MPZA-1001".into(),
        },
    );
}

#[test]
fn record_deposit_updates_member_and_sacco_totals() {
    let mut ledger = Ledger::new();
    verified_sacco(&mut ledger);
    add_verified_member(&mut ledger, aid(6));
    record(&mut ledger, 7);

    let sacco = ledger.sacco(aid(1));
    assert_eq!(sacco.total_deposits_savings, 100_000);
    assert_eq!(sacco.total_deposits_share_capital, 0);

    let member = ledger.member(aid(6));
    assert_eq!(member.savings_balance, 100_000);
    assert_eq!(member.share_capital_balance, 0);

    let deposit = ledger.deposit(aid(7));
    assert_eq!(deposit.amount, 100_000);
    assert_eq!(deposit.deposit_type, DepositType::Savings);
    assert_eq!(deposit.reference, "MPZA-1001");
    assert_eq!(deposit.recorded_by, aid(2));
    assert!(!deposit.confirmed_by_member);
}

#[test]
fn record_deposit_splits_totals_by_type() {
    let mut ledger = Ledger::new();
    verified_sacco(&mut ledger);
    add_verified_member(&mut ledger, aid(6));
    ledger.execute(
        &[meta(aid(1)), meta(aid(6)), meta(aid(7)), meta(aid(2))],
        Instruction::RecordDeposit {
            member: aid(6),
            amount: 40_000,
            deposit_type: DepositType::ShareCapital,
            reference: "MPZA-1002".into(),
        },
    );
    assert_eq!(ledger.sacco(aid(1)).total_deposits_savings, 0);
    assert_eq!(ledger.sacco(aid(1)).total_deposits_share_capital, 40_000);
    assert_eq!(ledger.member(aid(6)).savings_balance, 0);
    assert_eq!(ledger.member(aid(6)).share_capital_balance, 40_000);
}

#[test]
#[should_panic(expected = "Sacco is not verified")]
fn record_deposit_rejected_for_unverified_sacco() {
    let mut ledger = Ledger::new();
    let (sacco, admin) = (aid(1), aid(2));
    ledger.execute(
        &[meta(sacco), meta(admin)],
        Instruction::RegisterSacco {
            name: "Kampala Traders SACCO".into(),
            registration_number: "UG-SACCO-001".into(),
            treasury: aid(9),
        },
    );
    ledger.execute(
        &[meta(sacco), meta(aid(6)), meta(admin)],
        Instruction::AddMember { identity: aid(6) },
    );
    record(&mut ledger, 7);
}

#[test]
#[should_panic(expected = "Only SACCO approvers can record deposits")]
fn record_deposit_requires_approver() {
    let mut ledger = Ledger::new();
    verified_sacco(&mut ledger);
    add_verified_member(&mut ledger, aid(6));
    ledger.execute(
        &[meta(aid(1)), meta(aid(6)), meta(aid(7)), meta(aid(3))],
        Instruction::RecordDeposit {
            member: aid(6),
            amount: 1_000,
            deposit_type: DepositType::Savings,
            reference: "MPZA-3".into(),
        },
    );
}

#[test]
#[should_panic(expected = "Deposit reference must not be empty")]
fn record_deposit_rejects_empty_reference() {
    let mut ledger = Ledger::new();
    verified_sacco(&mut ledger);
    add_verified_member(&mut ledger, aid(6));
    ledger.execute(
        &[meta(aid(1)), meta(aid(6)), meta(aid(7)), meta(aid(2))],
        Instruction::RecordDeposit {
            member: aid(6),
            amount: 1_000,
            deposit_type: DepositType::Savings,
            reference: String::new(),
        },
    );
}

#[test]
#[should_panic(expected = "Deposit account must be uninitialized")]
fn record_deposit_cannot_overwrite_a_deposit() {
    let mut ledger = Ledger::new();
    verified_sacco(&mut ledger);
    add_verified_member(&mut ledger, aid(6));
    record(&mut ledger, 7);
    record(&mut ledger, 7);
}

#[test]
fn confirm_deposit_flips_member_confirmation() {
    let mut ledger = Ledger::new();
    verified_sacco(&mut ledger);
    add_verified_member(&mut ledger, aid(6));
    record(&mut ledger, 7);
    ledger.execute(
        &[meta(aid(7)), meta(aid(6)), meta(aid(6))],
        Instruction::ConfirmDeposit,
    );
    assert!(ledger.deposit(aid(7)).confirmed_by_member);
}

#[test]
fn confirm_deposit_accepts_merged_member_account_and_identity_signer() {
    let mut ledger = Ledger::new();
    verified_sacco(&mut ledger);
    add_verified_member(&mut ledger, aid(6));
    record(&mut ledger, 7);

    // LEZ permits a single mention per identity, so the member record and its
    // signer are represented by the same authorized account handle.
    ledger.execute(&[meta(aid(7)), meta(aid(6))], Instruction::ConfirmDeposit);

    assert!(ledger.deposit(aid(7)).confirmed_by_member);
}

#[test]
#[should_panic(expected = "Only the member can confirm their deposit")]
fn confirm_deposit_member_only() {
    let mut ledger = Ledger::new();
    verified_sacco(&mut ledger);
    add_verified_member(&mut ledger, aid(6));
    record(&mut ledger, 7);
    ledger.execute(
        &[meta(aid(7)), meta(aid(6)), meta(aid(3))],
        Instruction::ConfirmDeposit,
    );
}

#[test]
#[should_panic(expected = "Deposit already confirmed")]
fn confirm_deposit_rejects_double_confirm() {
    let mut ledger = Ledger::new();
    verified_sacco(&mut ledger);
    add_verified_member(&mut ledger, aid(6));
    record(&mut ledger, 7);
    ledger.execute(
        &[meta(aid(7)), meta(aid(6)), meta(aid(6))],
        Instruction::ConfirmDeposit,
    );
    ledger.execute(
        &[meta(aid(7)), meta(aid(6)), meta(aid(6))],
        Instruction::ConfirmDeposit,
    );
}

#[test]
fn only_the_member_identity_can_confirm_a_deposit() {
    let mut ledger = Ledger::new();
    verified_sacco(&mut ledger);
    ledger.execute(
        &[meta(aid(1)), meta(aid(6)), meta(aid(2))],
        Instruction::AddMember { identity: aid(16) },
    );
    record(&mut ledger, 7);

    ledger.execute(
        &[meta(aid(7)), meta(aid(6)), meta(aid(16))],
        Instruction::ConfirmDeposit,
    );
    assert!(ledger.deposit(aid(7)).confirmed_by_member);
}

#[test]
#[should_panic(expected = "Only the member can confirm their deposit")]
fn member_account_key_cannot_confirm_instead_of_identity() {
    let mut ledger = Ledger::new();
    verified_sacco(&mut ledger);
    ledger.execute(
        &[meta(aid(1)), meta(aid(6)), meta(aid(2))],
        Instruction::AddMember { identity: aid(16) },
    );
    record(&mut ledger, 7);

    ledger.execute(
        &[meta(aid(7)), meta(aid(6)), meta(aid(6))],
        Instruction::ConfirmDeposit,
    );
}

#[test]
#[should_panic(expected = "A member cannot record their own deposit")]
fn recorder_cannot_record_a_deposit_to_their_own_member_account() {
    let mut ledger = Ledger::new();
    verified_sacco(&mut ledger);
    ledger.execute(
        &[meta(aid(1)), meta(aid(6)), meta(aid(2))],
        Instruction::AddMember { identity: aid(2) },
    );
    record(&mut ledger, 7);
}
