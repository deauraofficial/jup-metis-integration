# Integration Guide

This guide explains how to integrate the Deaura AMM into Jupiter's aggregator.

## Step 1: Add Dependency

In your Jupiter aggregator's `Cargo.toml`, add:

```toml
[dependencies]
deaura-amm = { path = "../deaura_jup/deaura-amm" }
```

Or if published to crates.io:

```toml
[dependencies]
deaura-amm = "0.1.0"
```

## Step 2: Register the AMM

In your aggregator's main configuration file (typically in `jupiter-core/src/amms/mod.rs` or similar), add:

```rust
use deaura_amm::{DeauraAmm, DEAURA_PROGRAM_ID};
use jupiter_amm_interface::Amm;

// In PROGRAM_ID_TO_AMM_LABEL_WITH_AMM_FROM_KEYED_ACCOUNT:
(DEAURA_PROGRAM_ID, "Deaura", DeauraAmm::from_keyed_account)
```

## Step 3: Add Vault Accounts to Monitoring

Jupiter needs to monitor both vault accounts. Add them to your account monitoring list:

```rust
use deaura_amm::{VNX_DEPOSIT_VAULT, VNX_REDEEM_VAULT};

// Add to accounts to monitor:
vec![
    VNX_DEPOSIT_VAULT,
    VNX_REDEEM_VAULT,
    // ... other accounts
]
```

## Step 4: Test Integration

Run Jupiter's integration tests:

```bash
cargo test
```

## Example Usage

Once integrated, Jupiter will automatically:
1. Detect swaps between VNX and GOLDC
2. Route through Deaura vaults when optimal
3. Execute swaps using the appropriate deposit/redeem instructions

## Notes

- The Deaura AMM creates two separate instances (one per vault) for bidirectional swaps
- Deposit vault handles VNX → GOLDC conversions
- Redeem vault handles GOLDC → VNX conversions
- Current implementation uses 1:1 conversion rate (no fees)
