# Architecture

## Token model

$NRG is issued as a classic Stellar asset. Classic assets are native to the Stellar protocol and require no smart contract code to issue, transfer, or burn. They are fast, cheap, and well-supported by all Stellar wallets and tooling.

To interact with a classic asset from a Soroban contract, Stellar provides the Stellar Asset Contract (SAC): a standardized wrapper that exposes the SEP-41 token interface. Any classic asset can have a SAC deployed for it with a single CLI command. The SAC lets Soroban contracts call functions like `burn`, `transfer`, and `balance` on the underlying asset.

This architecture uses both layers:

- Classic Stellar asset: handles token issuance, distribution, and all standard wallet operations.
- SAC: makes the asset callable from Soroban.
- RedemptionContract: contains all redemption business logic.

## Redemption flow

```
1. Fan connects Freighter wallet on the NRG website.
2. Fan selects a perk and submits a redemption request.
3. Frontend calls redeem(from, perk_id, amount) on the RedemptionContract.
   - The user signs the transaction in Freighter.
   - The contract calls burn(from, amount) on the SAC.
   - Tokens are destroyed. The burn is final.
   - A RedemptionRecord is written to persistent storage with status=Pending.
   - A `redeemed` event is emitted.
4. Backend (Node.js) polls Soroban RPC for `redeemed` events.
5. Backend finds the event, creates a fulfillment task for the team.
6. Team delivers the perk (sends ticket, grants backstage access, etc.).
7. Admin calls fulfill(id) on the contract.
   - Status changes to Fulfilled.
   - A `fulfilled` event is emitted.
8. Frontend shows the fan their updated redemption history.
```

## State transitions

```
             redeem()
[none] ---------> [Pending]
                      |
          fulfill()   |   cancel()
                      |
              [Fulfilled]  [Cancelled]
```

Once a record reaches Fulfilled or Cancelled it is terminal. No further transitions are possible.

## Storage design

Instance storage holds three keys that are read on nearly every call: Admin, Token, and NextId. Instance storage is cheaper and faster for frequently-accessed data.

Persistent storage holds RedemptionRecord entries keyed by ID. Persistent entries survive ledger TTL for extended periods and are appropriate for records that must remain queryable over the lifetime of the contract.

## Auth model

`redeem` requires the `from` address to authorize the invocation. This ensures only the token holder can initiate a burn.

`fulfill` and `cancel` require the admin address. The admin is set at initialization and can be transferred via `set_admin`.

`set_admin` requires both the current admin and the new admin to sign, preventing accidental transfers to non-custodied addresses.

## Event indexing

Events are the primary integration surface for the backend. The backend uses Soroban RPC (`getEvents`) to poll for contract events by topic. It filters on:

- Topic `redeemed` to detect new redemptions.
- Topic `fulfilled` / `cancelled` for audit logging.

Event payloads include the redemption ID, perk ID, and amount, which is sufficient to route fulfillment without a secondary database lookup.

## Production considerations

**TTL management**: Persistent storage entries expire. A background job should monitor active Pending records and call `stellar contract extend` before expiry. The current minimum persistent TTL on mainnet is approximately 30 days.

**Admin key security**: The admin key signs fulfillment transactions. Use a hardware wallet or multi-sig setup for mainnet. Consider rotating to a multisig admin after deployment.

**Perk ID format**: perk_id is a free-form string. The recommended format is `{perk-type}-{location}-{date}` (e.g., `backstage-paris-2026-03-15`). Validation of perk IDs against a catalog is intentionally out-of-scope for the contract; it is handled by the backend before display to the user.
