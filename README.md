# nrg-redemption-contract

Soroban smart contract implementing burn-to-redeem logic for the $NRG fan token on Stellar.

Fans burn NRG tokens to claim perks: concert tickets, backstage access, VIP tables, unreleased music. Redemptions are recorded on-chain. An admin account marks them as fulfilled once delivery is confirmed.

## How it works

$NRG is issued as a classic Stellar asset. A Stellar Asset Contract (SAC) is deployed from it, making the token callable from Soroban. This contract uses the SAC's `burn` function to destroy tokens at redemption time, then writes a `RedemptionRecord` to persistent storage and emits an on-chain event.

The backend listens for `redeemed` events via Soroban RPC and triggers perk fulfillment. Once delivered, the admin calls `fulfill` to close the record.

```
Fan                   RedemptionContract            SAC ($NRG)
 |-- redeem(perk) -->  |-- burn(from, amount) -->    |
                       |<-- ok --                    |
                       |-- write RedemptionRecord    |
                       |-- emit redeemed event       |

Backend               RedemptionContract
 |<-- poll events --   |
 |-- deliver perk      |
 |-- fulfill(id) -->   |-- update status = Fulfilled
```

## Storage

| Scope      | Key                  | Value                              |
|------------|----------------------|------------------------------------|
| Instance   | Admin                | Address                            |
| Instance   | Token                | Address (SAC)                      |
| Instance   | NextId               | u64                                |
| Persistent | Redemption(id: u64)  | RedemptionRecord                   |

## Contract interface

```rust
fn initialize(env, admin: Address, token: Address)
fn redeem(env, from: Address, perk_id: String, amount: i128) -> u64
fn fulfill(env, id: u64)
fn cancel(env, id: u64)
fn set_admin(env, new_admin: Address)
fn get_redemption(env, id: u64) -> RedemptionRecord
fn admin(env) -> Address
fn token(env) -> Address
fn redemption_count(env) -> u64
```

## Events

| Topic      | Data                              | When                     |
|------------|-----------------------------------|--------------------------|
| redeemed   | (id, perk_id, amount)             | Successful burn          |
| fulfilled  | (id,)                             | Admin confirms delivery  |
| cancelled  | (id,)                             | Admin cancels request    |

## Requirements

- Rust 1.70+
- `wasm32-unknown-unknown` target
- `stellar-cli` for deployment

```sh
rustup target add wasm32-unknown-unknown
cargo install --locked stellar-cli --features opt
```

## Build

```sh
cargo build --release --target wasm32-unknown-unknown \
  -p nrg-redemption
```

The compiled WASM will be at:
`target/wasm32-unknown-unknown/release/nrg_redemption.wasm`

## Test

```sh
cargo test
```

18 tests covering unit behavior, edge cases, and UAT scenarios.

## Deploy to testnet

### 1. Configure stellar-cli

```sh
stellar network add testnet \
  --rpc-url https://soroban-testnet.stellar.org \
  --network-passphrase "Test SDF Network ; September 2015"

stellar keys generate deployer --network testnet --fund
```

### 2. Deploy the SAC for $NRG

If you have the $NRG asset issuer secret key:

```sh
stellar contract asset deploy \
  --network testnet \
  --source deployer \
  --asset NRG:<ISSUER_PUBLIC_KEY>
```

Note the SAC contract ID returned.

### 3. Deploy the redemption contract

```sh
stellar contract deploy \
  --network testnet \
  --source deployer \
  --wasm target/wasm32-unknown-unknown/release/nrg_redemption.wasm
```

Note the contract ID returned.

### 4. Initialize

```sh
stellar contract invoke \
  --network testnet \
  --source deployer \
  --id <REDEMPTION_CONTRACT_ID> \
  -- initialize \
  --admin <ADMIN_PUBLIC_KEY> \
  --token <SAC_CONTRACT_ID>
```

### 5. Invoke redeem (example)

```sh
stellar contract invoke \
  --network testnet \
  --source <USER_SECRET_KEY> \
  --id <REDEMPTION_CONTRACT_ID> \
  -- redeem \
  --from <USER_PUBLIC_KEY> \
  --perk_id "backstage-paris-2026-03-15" \
  --amount 1000000000
```

Amounts use 7 decimal places. 1000000000 = 100 NRG.

### 6. Fulfill (admin)

```sh
stellar contract invoke \
  --network testnet \
  --source <ADMIN_SECRET_KEY> \
  --id <REDEMPTION_CONTRACT_ID> \
  -- fulfill \
  --id 0
```

## Architecture notes

**Why classic Stellar asset + SAC rather than a native Soroban token?**

SAC-wrapped classic assets use 97% less CPU and 98% less RAM than equivalent pure Soroban tokens (Cheesecake Labs benchmark, Feb 2024). The token itself is simple: issuance, transfer, burn. Classic assets handle this at the protocol level. Soroban is used where it adds value: the redemption registry, on-chain events, and admin controls.

**Why tokens are not refunded on cancellation**

The burn is executed at redemption time and is irreversible. Cancellation flags a request as undeliverable. Token compensation, if warranted, is handled by the admin out-of-band. This keeps the contract state machine simple and avoids re-entrancy considerations.

**TTL and state archival**

Redemption records use persistent storage. On mainnet, entries approaching expiry must be extended via `stellar contract extend`. For production deployments, run a background job that queries `getExpirationLedger` for active pending records and extends them before archival.

## License

MIT

---

## Testnet Deployment

**Deployed:** 2026-03-14 · Stellar Testnet

### Contract IDs

| Contract | ID |
|---|---|
| **SAC (NRG Token)** | `CDVDFTOTYIU7B7PWYQGI2KV6HWKIZI2PXRFBCVLQU2FBIJAZA2OGYNEL` |
| **Redemption Contract** | `CDSNPGRFJD2GB3IPWRTRF4EHN55ZR66QG5LKN5YP6QKLPLH2F6AA5LJ4` |

### Explorer

- [SAC (NRG Token)](https://stellar.expert/explorer/testnet/contract/CDVDFTOTYIU7B7PWYQGI2KV6HWKIZI2PXRFBCVLQU2FBIJAZA2OGYNEL)
- [Redemption Contract](https://stellar.expert/explorer/testnet/contract/CDSNPGRFJD2GB3IPWRTRF4EHN55ZR66QG5LKN5YP6QKLPLH2F6AA5LJ4)

### Build environment

- Rust 1.94.0
- Target: `wasm32v1-none` (required for Soroban VM — `wasm32-unknown-unknown` emits reference-types that Soroban rejects)
- stellar-cli v25.2.0
- soroban-sdk v22.0.10
- WASM: 24,812 bytes optimized

### Build note

The workspace `Cargo.toml` had `testutils` as a workspace-level feature dependency which prevents WASM builds. If you encounter build errors, ensure `testutils` is only enabled in `[dev-dependencies]`, not at the workspace feature level.
