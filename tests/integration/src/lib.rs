// Integration tests for the NRG Redemption Contract.
//
// These tests run against the Soroban test environment, which simulates
// ledger state, auth, and token behavior without requiring a live network.

#![cfg(test)]

use nrg_redemption::{RedemptionContract, RedemptionContractClient, RedemptionStatus};
use soroban_sdk::{
    testutils::{Address as _, Ledger, LedgerInfo},
    token::{StellarAssetClient, TokenClient},
    Address, Env, String,
};

// ---- Setup helpers ----------------------------------------------------------

struct TestEnv {
    env: Env,
    contract: Address,
    token: Address,
    admin: Address,
    user: Address,
}

fn setup() -> TestEnv {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    // Deploy a mock Stellar asset (SAC-compatible) as the NRG token.
    let token_admin = Address::generate(&env);
    let token = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token.address();

    // Mint 1000 NRG to the test user.
    let asset_client = StellarAssetClient::new(&env, &token_address);
    asset_client.mint(&user, &1_000_0000000); // 1000 NRG in stroops (7 decimals)

    // Deploy the redemption contract.
    let contract = env.register(RedemptionContract, ());
    let client = RedemptionContractClient::new(&env, &contract);
    client.initialize(&admin, &token_address);

    TestEnv {
        env,
        contract,
        token: token_address,
        admin,
        user,
    }
}

fn client<'a>(t: &'a TestEnv) -> RedemptionContractClient<'a> {
    RedemptionContractClient::new(&t.env, &t.contract)
}

fn token_client<'a>(t: &'a TestEnv) -> TokenClient<'a> {
    TokenClient::new(&t.env, &t.token)
}

fn perk(env: &Env, s: &str) -> String {
    String::from_str(env, s)
}

// ---- Unit tests (contract logic) --------------------------------------------

#[test]
fn test_initialize_sets_admin_and_token() {
    let t = setup();
    let c = client(&t);
    assert_eq!(c.admin(), t.admin);
    assert_eq!(c.token(), t.token);
    assert_eq!(c.redemption_count(), 0);
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_cannot_initialize_twice() {
    let t = setup();
    let c = client(&t);
    c.initialize(&t.admin, &t.token);
}

#[test]
fn test_redeem_burns_tokens_and_creates_record() {
    let t = setup();
    let c = client(&t);

    let initial_balance = token_client(&t).balance(&t.user);
    let burn_amount: i128 = 10_0000000; // 10 NRG

    let id = c.redeem(&t.user, &perk(&t.env, "backstage-paris-2026"), &burn_amount);

    assert_eq!(id, 0);
    assert_eq!(c.redemption_count(), 1);

    let record = c.get_redemption(&id);
    assert_eq!(record.redeemer, t.user);
    assert_eq!(record.amount, burn_amount);
    assert_eq!(record.status, RedemptionStatus::Pending);

    let final_balance = token_client(&t).balance(&t.user);
    assert_eq!(final_balance, initial_balance - burn_amount);
}

#[test]
fn test_multiple_redemptions_increment_ids() {
    let t = setup();
    let c = client(&t);

    let id0 = c.redeem(&t.user, &perk(&t.env, "vip-ibiza"), &1_0000000);
    let id1 = c.redeem(&t.user, &perk(&t.env, "backstage-london"), &2_0000000);
    let id2 = c.redeem(&t.user, &perk(&t.env, "unreleased-track-01"), &5_0000000);

    assert_eq!(id0, 0);
    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
    assert_eq!(c.redemption_count(), 3);
}

#[test]
fn test_fulfill_sets_status_to_fulfilled() {
    let t = setup();
    let c = client(&t);

    let id = c.redeem(&t.user, &perk(&t.env, "backstage-miami"), &5_0000000);
    c.fulfill(&id);

    let record = c.get_redemption(&id);
    assert_eq!(record.status, RedemptionStatus::Fulfilled);
}

#[test]
fn test_cancel_sets_status_to_cancelled() {
    let t = setup();
    let c = client(&t);

    let id = c.redeem(&t.user, &perk(&t.env, "vip-berlin"), &3_0000000);
    c.cancel(&id);

    let record = c.get_redemption(&id);
    assert_eq!(record.status, RedemptionStatus::Cancelled);
}

#[test]
#[should_panic(expected = "redemption is not pending")]
fn test_cannot_fulfill_already_fulfilled() {
    let t = setup();
    let c = client(&t);

    let id = c.redeem(&t.user, &perk(&t.env, "ticket-amsterdam"), &2_0000000);
    c.fulfill(&id);
    c.fulfill(&id); // should panic
}

#[test]
#[should_panic(expected = "redemption is not pending")]
fn test_cannot_cancel_already_fulfilled() {
    let t = setup();
    let c = client(&t);

    let id = c.redeem(&t.user, &perk(&t.env, "ticket-amsterdam"), &2_0000000);
    c.fulfill(&id);
    c.cancel(&id); // should panic
}

#[test]
#[should_panic(expected = "redemption is not pending")]
fn test_cannot_fulfill_cancelled_redemption() {
    let t = setup();
    let c = client(&t);

    let id = c.redeem(&t.user, &perk(&t.env, "ticket-amsterdam"), &2_0000000);
    c.cancel(&id);
    c.fulfill(&id); // should panic
}

#[test]
#[should_panic(expected = "amount must be positive")]
fn test_redeem_rejects_zero_amount() {
    let t = setup();
    let c = client(&t);
    c.redeem(&t.user, &perk(&t.env, "vip-ticket"), &0);
}

#[test]
#[should_panic(expected = "amount must be positive")]
fn test_redeem_rejects_negative_amount() {
    let t = setup();
    let c = client(&t);
    c.redeem(&t.user, &perk(&t.env, "vip-ticket"), &-100);
}

#[test]
#[should_panic(expected = "redemption not found")]
fn test_get_nonexistent_redemption_panics() {
    let t = setup();
    let c = client(&t);
    c.get_redemption(&99);
}

// ---- Timestamp tests --------------------------------------------------------

#[test]
fn test_redemption_timestamps_are_set() {
    let t = setup();
    let c = client(&t);

    t.env.ledger().set(LedgerInfo {
        timestamp: 1_700_000_000,
        protocol_version: 22,
        sequence_number: 100,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 16,
        min_persistent_entry_ttl: 100_000,
        max_entry_ttl: 6_312_000,
    });

    let id = c.redeem(&t.user, &perk(&t.env, "backstage-paris-2026"), &1_0000000);
    let record = c.get_redemption(&id);

    assert_eq!(record.created_at, 1_700_000_000);
    assert_eq!(record.updated_at, 1_700_000_000);
}

#[test]
fn test_fulfill_updates_timestamp() {
    let t = setup();
    let c = client(&t);

    // Use a high persistent entry TTL so advancing the sequence by a few
    // ledgers does not archive entries between calls.
    t.env.ledger().set(LedgerInfo {
        timestamp: 1_700_000_000,
        protocol_version: 22,
        sequence_number: 100,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 16,
        min_persistent_entry_ttl: 100_000,
        max_entry_ttl: 6_312_000,
    });

    let id = c.redeem(&t.user, &perk(&t.env, "vip-pass"), &2_0000000);

    // Advance timestamp only; keep sequence close enough that persistent
    // entries are not archived.
    t.env.ledger().set(LedgerInfo {
        timestamp: 1_700_001_000,
        protocol_version: 22,
        sequence_number: 101,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 16,
        min_persistent_entry_ttl: 100_000,
        max_entry_ttl: 6_312_000,
    });

    c.fulfill(&id);

    let record = c.get_redemption(&id);
    assert_eq!(record.created_at, 1_700_000_000);
    assert_eq!(record.updated_at, 1_700_001_000);
}

// ---- Admin transfer tests ---------------------------------------------------

#[test]
fn test_set_admin_transfers_control() {
    let t = setup();
    let c = client(&t);
    let new_admin = Address::generate(&t.env);

    c.set_admin(&new_admin);
    assert_eq!(c.admin(), new_admin);
}

// ---- UAT scenarios ----------------------------------------------------------
//
// These tests simulate realistic end-to-end user flows as described in the
// product spec: fan redeems perk, backstage team confirms delivery.

#[test]
fn uat_fan_redeems_backstage_pass() {
    let t = setup();
    let c = client(&t);

    // Fan holds 500 NRG and wants a backstage pass (costs 100 NRG).
    let fan = t.user.clone();
    let perk_id = perk(&t.env, "backstage-paris-2026-03-15");
    let cost: i128 = 100_0000000;

    let balance_before = token_client(&t).balance(&fan);
    let id = c.redeem(&fan, &perk_id, &cost);

    let record = c.get_redemption(&id);
    assert_eq!(record.redeemer, fan);
    assert_eq!(record.status, RedemptionStatus::Pending);
    assert_eq!(token_client(&t).balance(&fan), balance_before - cost);

    // Backstage team confirms access was granted, marks fulfilled.
    c.fulfill(&id);
    assert_eq!(c.get_redemption(&id).status, RedemptionStatus::Fulfilled);
}

#[test]
fn uat_invalid_perk_request_is_cancelled() {
    let t = setup();
    let c = client(&t);

    // Fan submits request for a perk that no longer exists.
    let id = c.redeem(&t.user, &perk(&t.env, "expired-perk-2024"), &5_0000000);
    assert_eq!(c.get_redemption(&id).status, RedemptionStatus::Pending);

    // Admin cancels after reviewing.
    c.cancel(&id);
    assert_eq!(c.get_redemption(&id).status, RedemptionStatus::Cancelled);
}

#[test]
fn uat_fan_redeems_multiple_perks_sequentially() {
    let t = setup();
    let c = client(&t);

    let id0 = c.redeem(&t.user, &perk(&t.env, "unreleased-track-001"), &10_0000000);
    let id1 = c.redeem(&t.user, &perk(&t.env, "vip-table-miami-2026"), &50_0000000);

    c.fulfill(&id0);
    // id1 still pending - delivery in progress.

    assert_eq!(c.get_redemption(&id0).status, RedemptionStatus::Fulfilled);
    assert_eq!(c.get_redemption(&id1).status, RedemptionStatus::Pending);
    assert_eq!(c.redemption_count(), 2);
}
