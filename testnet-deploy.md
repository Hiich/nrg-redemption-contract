# NRG Redemption Contract — Testnet Deployment

**Date:** 2026-03-14 13:38 UTC

## Contract IDs

| Contract | ID |
|---|---|
| **SAC (NRG Token)** | `CDVDFTOTYIU7B7PWYQGI2KV6HWKIZI2PXRFBCVLQU2FBIJAZA2OGYNEL` |
| **Redemption Contract** | `CDSNPGRFJD2GB3IPWRTRF4EHN55ZR66QG5LKN5YP6QKLPLH2F6AA5LJ4` |

## Deployer

- **Public Key:** `GDBZR6WYKXACGHODXYAEMY3UKNTQAIYEVNO4BEVLEC6WGF6G73ZXSS7A`

## Explorer Links

- SAC (NRG Token): https://stellar.expert/explorer/testnet/contract/CDVDFTOTYIU7B7PWYQGI2KV6HWKIZI2PXRFBCVLQU2FBIJAZA2OGYNEL
- Redemption Contract: https://stellar.expert/explorer/testnet/contract/CDSNPGRFJD2GB3IPWRTRF4EHN55ZR66QG5LKN5YP6QKLPLH2F6AA5LJ4
- Deployer Account: https://stellar.expert/explorer/testnet/account/GDBZR6WYKXACGHODXYAEMY3UKNTQAIYEVNO4BEVLEC6WGF6G73ZXSS7A

## Transaction Hashes

1. **SAC Deploy:** `0890f5d3366a716c32e64b2277064da4bcbc43e9e795b549255775db382261f7`
2. **WASM Upload:** `4cf2197aaa894ff33ffc8c6744ad88620c9863e0e338c893fbcd1cc2f60890ba`
3. **Contract Deploy:** `7495bcc9a8b414eaacdc010456cb83f51c5610810c7068c2d1e5b501f7f09967`
4. **Initialize:** `633ad931bb0d3baf2b89a54ac16a79db97d7d3ebc9c0053fbd20fdc31b548032`

## Redemption Test (End-to-End)

**Date:** 2026-03-14 13:50 UTC

- **Fan Account:** `GBA3DT4ZVZVAG2ZPBS3P53O7UVAO7LZ7Y4VS4GPJGLUYSXGSCM4OLFM6`
- **Perk:** `backstage-scf-demo`
- **Amount burned:** 100 NRG (1000000000 stroops)

| Step | Tx Hash | Explorer |
|---|---|---|
| **Trustline (NRG)** | `b646d0f8b4dcbea234d19cc3c5029d3314a30e37724a59504e8f49f5cc0ae117` | [link](https://stellar.expert/explorer/testnet/tx/b646d0f8b4dcbea234d19cc3c5029d3314a30e37724a59504e8f49f5cc0ae117) |
| **Payment (1000 NRG)** | `f920452f1ad0110b9b55270e44bd5a8467d4ec8a133fb906d10bb78fb2917426` | [link](https://stellar.expert/explorer/testnet/tx/f920452f1ad0110b9b55270e44bd5a8467d4ec8a133fb906d10bb78fb2917426) |
| **Redeem (burn 100 NRG)** | `676f90051ac81eba2a9cfd6c49f1305684ed6f8455208807898045b4c25fd362` | [link](https://stellar.expert/explorer/testnet/tx/676f90051ac81eba2a9cfd6c49f1305684ed6f8455208807898045b4c25fd362) |
| **Fulfill** | `93b73d1c2297f7fd7993429e00c4e4613b8ee2a0bee7804c5296309eb59f3d09` | [link](https://stellar.expert/explorer/testnet/tx/93b73d1c2297f7fd7993429e00c4e4613b8ee2a0bee7804c5296309eb59f3d09) |

**Final state:**
```json
{"amount":"1000000000","created_at":1773496291,"id":0,"perk_id":"backstage-scf-demo","redeemer":"GBA3DT4ZVZVAG2ZPBS3P53O7UVAO7LZ7Y4VS4GPJGLUYSXGSCM4OLFM6","status":"Fulfilled","updated_at":1773496301}
```

## Technical Notes

- Built with Rust 1.94.0, target `wasm32v1-none` (required for Soroban VM compatibility)
- stellar-cli v25.2.0
- soroban-sdk v22.0.10
- WASM size: 24,812 bytes (optimized)
- Network: **testnet** (never mainnet)
