# Deaura Jupiter AMM Integration

This repository contains the Jupiter AMM integration for the Deaura token swap vault on Solana.

## Overview

The Deaura AMM integration enables the Deaura vault to be used as a liquidity source in Jupiter's routing system. The vault supports two operations:
- **Deposit**: Convert VNX tokens to GOLDC tokens
- **Redeem**: Convert GOLDC tokens back to VNX tokens

## Architecture

This is a Rust workspace containing:
- `deaura-amm/`: The main AMM implementation crate that implements Jupiter's `Amm` trait

## Program Details

- **Program ID**: `5ZcDxdRBiRe73S68BCHE7NwPt82evS5FyPPU9rfXwYBj`
- **VNX Mint**: `9TPL8droGJ7jThsq4momaoz6uhTcvX2SeMqipoPmNa8R`
- **GOLDC Mint**: `EhGYsb13zhso2xhQSd1H1xdu6bvcv88oLoVMWgfAV6tx`
- **Deposit Vault**: `CKixsXaerxYaaXuijWQFxKAyXHkAhfi2r9BBk6Wke4BH`
- **Redeem Vault**: `EUpqbEGhSPBegZJbk3HbdBNnMW7DTy7tb8fwnAejcfG1`

## Integration with Jupiter Router

To integrate this AMM into Jupiter's router, you need to:

1. Register the Deaura program ID in Jupiter's aggregator configuration
2. Add both vault accounts (`VNX_DEPOSIT_VAULT` and `VNX_REDEEM_VAULT`) to the list of accounts Jupiter monitors
3. Ensure Jupiter's aggregator calls `DeauraAmm::from_keyed_account` for each vault

### Registration Example

In Jupiter's aggregator code, you would typically add:

```rust
use deaura_amm::DeauraAmm;
use jupiter_amm_interface::Amm;

// In PROGRAM_ID_TO_AMM_LABEL_WITH_AMM_FROM_KEYED_ACCOUNT:
(DeauraAmm::DEAURA_PROGRAM_ID, "Deaura", DeauraAmm::from_keyed_account)
```

## Building

```bash
cargo build --release
```

## Testing

```bash
cargo test
```

## Structure

```
deaura_jup/
├── Cargo.toml              # Workspace configuration
├── README.md               # This file
├── .gitignore             # Git ignore rules
└── deaura-amm/            # Main AMM implementation crate
    ├── Cargo.toml         # Crate dependencies
    └── src/
        ├── lib.rs         # Crate exports
        └── amm.rs         # DeauraAmm implementation
```

## Dependencies

- `jupiter-amm-interface`: Jupiter's AMM interface trait
- `solana-sdk`: Solana SDK for account and instruction handling
- `spl-token`: SPL Token program types
- `anchor-lang`: Anchor framework (for program IDL compatibility)

## Notes

- The current implementation uses a 1:1 conversion rate (no fees)
- Reserve checking is performed for redeem operations to ensure sufficient VNX liquidity
- The implementation creates two separate AMM instances (one per vault) to handle bidirectional swaps

## References

- [Jupiter AMM Implementation Guide](https://github.com/jup-ag/jupiter-amm-implementation)
- [Jupiter AMM Interface Documentation](https://docs.rs/jupiter-amm-interface)
