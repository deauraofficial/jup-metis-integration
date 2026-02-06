use solana_sdk::{pubkey, pubkey::{Pubkey}};

/// Deaura Program ID
pub const DEAURA_PROGRAM_ID: Pubkey = pubkey!("5ZcDxdRBiRe73S68BCHE7NwPt82evS5FyPPU9rfXwYBj");

/// Token Mints
pub const VNX_MINT: Pubkey = pubkey!("9TPL8droGJ7jThsq4momaoz6uhTcvX2SeMqipoPmNa8R");
pub const GOLDC_MINT: Pubkey = pubkey!("EhGYsb13zhso2xhQSd1H1xdu6bvcv88oLoVMWgfAV6tx");

/// Vault Accounts
/// Deposit vault: VNX -> GOLDC
pub const VNX_DEPOSIT_VAULT: Pubkey = pubkey!("CKixsXaerxYaaXuijWQFxKAyXHkAhfi2r9BBk6Wke4BH");
/// Redeem vault: GOLDC -> VNX
pub const VNX_REDEEM_VAULT: Pubkey = pubkey!("EUpqbEGhSPBegZJbk3HbdBNnMW7DTy7tb8fwnAejcfG1");

/// Anchor Instruction Discriminators
/// deposit(amount: u64)
pub const DEPOSIT_IX_DISC: [u8; 8] = [242, 35, 198, 137, 82, 225, 242, 182];
/// redeem(amount: u64)
pub const REDEEM_IX_DISC: [u8; 8] = [184, 12, 86, 149, 70, 196, 97, 225];
