//! NRG Redemption Contract
//!
//! Manages burn-to-redeem logic for the $NRG fan token on Stellar.
//! Users burn NRG tokens to claim perks (concert tickets, backstage, VIP, etc.).
//! An admin account marks redemptions as fulfilled once delivery is confirmed.
//!
//! Architecture:
//!   - $NRG is a classic Stellar asset wrapped via SAC (Stellar Asset Contract).
//!   - This contract holds no tokens. It calls `burn` on the SAC on behalf of the user.
//!   - All redemption state is stored on-chain for auditability.
//!
//! Storage layout:
//!   Instance: Admin (Address), TokenAddress (Address), NextId (u64)
//!   Persistent: Redemption(u64) -> RedemptionRecord

#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short,
    token::Client as TokenClient,
    Address, Env, String,
};

// ---- Types ------------------------------------------------------------------

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum RedemptionStatus {
    Pending,
    Fulfilled,
    Cancelled,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct RedemptionRecord {
    pub id: u64,
    pub redeemer: Address,
    pub perk_id: String,
    pub amount: i128,
    pub status: RedemptionStatus,
    pub created_at: u64,
    pub updated_at: u64,
}

// ---- Storage keys -----------------------------------------------------------

#[contracttype]
enum DataKey {
    Admin,
    Token,
    NextId,
    Redemption(u64),
}

// ---- Events -----------------------------------------------------------------

fn emit_redeemed(env: &Env, id: u64, redeemer: &Address, perk_id: &String, amount: i128) {
    env.events().publish(
        (symbol_short!("redeemed"), redeemer.clone()),
        (id, perk_id.clone(), amount),
    );
}

fn emit_fulfilled(env: &Env, id: u64) {
    env.events()
        .publish((symbol_short!("fulfilled"),), (id,));
}

fn emit_cancelled(env: &Env, id: u64) {
    env.events()
        .publish((symbol_short!("cancelled"),), (id,));
}

// ---- Internal helpers -------------------------------------------------------

fn get_admin(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::Admin).unwrap()
}

fn get_token(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::Token).unwrap()
}

fn next_id(env: &Env) -> u64 {
    let id: u64 = env
        .storage()
        .instance()
        .get(&DataKey::NextId)
        .unwrap_or(0);
    env.storage()
        .instance()
        .set(&DataKey::NextId, &(id + 1));
    id
}

fn get_redemption(env: &Env, id: u64) -> RedemptionRecord {
    env.storage()
        .persistent()
        .get(&DataKey::Redemption(id))
        .unwrap_or_else(|| panic!("redemption not found"))
}

fn save_redemption(env: &Env, record: &RedemptionRecord) {
    env.storage()
        .persistent()
        .set(&DataKey::Redemption(record.id), record);
}

// ---- Contract ---------------------------------------------------------------

#[contract]
pub struct RedemptionContract;

#[contractimpl]
impl RedemptionContract {
    /// Initialize the contract.
    ///
    /// Must be called once after deployment. `token` is the SAC address
    /// for the $NRG classic Stellar asset.
    pub fn initialize(env: Env, admin: Address, token: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Token, &token);
        env.storage().instance().set(&DataKey::NextId, &0u64);
    }

    /// Burn `amount` NRG tokens from the caller and register a redemption.
    ///
    /// The caller must have authorized this invocation. The SAC `burn` call
    /// will fail if the caller has insufficient balance.
    ///
    /// Returns the redemption ID.
    pub fn redeem(env: Env, from: Address, perk_id: String, amount: i128) -> u64 {
        from.require_auth();

        if amount <= 0 {
            panic!("amount must be positive");
        }

        let token_address = get_token(&env);
        let token = TokenClient::new(&env, &token_address);

        // Burn the tokens. This call will revert if the user lacks funds
        // or has not approved the contract.
        token.burn(&from, &amount);

        let id = next_id(&env);
        let now = env.ledger().timestamp();

        let record = RedemptionRecord {
            id,
            redeemer: from.clone(),
            perk_id: perk_id.clone(),
            amount,
            status: RedemptionStatus::Pending,
            created_at: now,
            updated_at: now,
        };

        save_redemption(&env, &record);
        emit_redeemed(&env, id, &from, &perk_id, amount);

        id
    }

    /// Mark a redemption as fulfilled. Admin only.
    ///
    /// Called once the perk has been delivered to the fan.
    pub fn fulfill(env: Env, id: u64) {
        let admin = get_admin(&env);
        admin.require_auth();

        let mut record = get_redemption(&env, id);

        if record.status != RedemptionStatus::Pending {
            panic!("redemption is not pending");
        }

        record.status = RedemptionStatus::Fulfilled;
        record.updated_at = env.ledger().timestamp();

        save_redemption(&env, &record);
        emit_fulfilled(&env, id);
    }

    /// Cancel a pending redemption. Admin only.
    ///
    /// Note: tokens are already burned at redemption time and are not
    /// returned on cancellation. Cancellation is a record-keeping operation
    /// to signal that the perk will not be delivered (e.g., invalid request).
    /// Any token compensation must be handled out-of-band.
    pub fn cancel(env: Env, id: u64) {
        let admin = get_admin(&env);
        admin.require_auth();

        let mut record = get_redemption(&env, id);

        if record.status != RedemptionStatus::Pending {
            panic!("redemption is not pending");
        }

        record.status = RedemptionStatus::Cancelled;
        record.updated_at = env.ledger().timestamp();

        save_redemption(&env, &record);
        emit_cancelled(&env, id);
    }

    /// Transfer admin rights to a new address.
    pub fn set_admin(env: Env, new_admin: Address) {
        let admin = get_admin(&env);
        admin.require_auth();
        new_admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &new_admin);
    }

    /// Query a single redemption record by ID.
    pub fn get_redemption(env: Env, id: u64) -> RedemptionRecord {
        get_redemption(&env, id)
    }

    /// Return the current admin address.
    pub fn admin(env: Env) -> Address {
        get_admin(&env)
    }

    /// Return the NRG token (SAC) address this contract is bound to.
    pub fn token(env: Env) -> Address {
        get_token(&env)
    }

    /// Return the next redemption ID (i.e., total number of redemptions so far).
    pub fn redemption_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::NextId)
            .unwrap_or(0)
    }
}
