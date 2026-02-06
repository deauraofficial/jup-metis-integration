use anyhow::{anyhow, ensure, Result};
use rust_decimal::Decimal;
use spl_token::state::Account as TokenAccount;

use crate::constants::{
    DEAURA_PROGRAM_ID, DEPOSIT_IX_DISC, GOLDC_MINT, REDEEM_IX_DISC, VNX_DEPOSIT_VAULT,
    VNX_MINT, VNX_REDEEM_VAULT,
};
use jupiter_amm_interface::{
    try_get_account_data, AccountMap, Amm, AmmContext, KeyedAccount, Quote, QuoteParams,
    Swap, SwapAndAccountMetas, SwapParams,
};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    program_pack::Pack,
    pubkey::Pubkey,
};

#[derive(Clone, Copy, Debug)]
enum DeauraDirection {
    Deposit, // VNX -> GOLDC
    Redeem,  // GOLDC -> VNX
}

pub struct DeauraAmm {
    /// Unique identifier for this AMM instance (we use the vault pubkey)
    key: Pubkey,
    /// Human label
    label: String,
    /// Program
    program_id: Pubkey,
    /// Which vault account this instance uses
    vnx_vault: Pubkey,
    /// Direction associated with this instance (only used for update/reserve checks)
    direction: DeauraDirection,

    /// Cached reserve (only meaningful for redeem direction, where vault must have VNX)
    vnx_reserve: u128,
}

impl DeauraAmm {
    fn derive_global_state() -> Pubkey {
        Pubkey::find_program_address(&[b"global_state"], &DEAURA_PROGRAM_ID).0
    }

    fn derive_vault_authority() -> Pubkey {
        Pubkey::find_program_address(&[b"vault_authority"], &DEAURA_PROGRAM_ID).0
    }

    fn derive_user_data(payer: &Pubkey) -> Pubkey {
        Pubkey::find_program_address(&[b"user_state", payer.as_ref()], &DEAURA_PROGRAM_ID).0
    }

    /// Instruction data = discriminator (8) + amount (u64 LE)
    fn ix_data(discriminator: [u8; 8], amount: u64) -> Vec<u8> {
        let mut data = Vec::with_capacity(16);
        data.extend_from_slice(&discriminator);
        data.extend_from_slice(&amount.to_le_bytes());
        data
    }

    /// Build account metas in the exact order required by your Anchor instruction.
    ///
    /// IDL order (deposit/redeem) is:
    /// payer, global_state, vault_authority, goldc_mint, payer_goldc_token_account,
    /// vnx_mint, payer_vnx_token_account, vnx_vault, user_data,
    /// token_program, associated_token_program, system_program
    fn account_metas(
        payer: Pubkey,
        payer_goldc_ata: Pubkey,
        payer_vnx_ata: Pubkey,
        vnx_vault: Pubkey,
    ) -> Vec<AccountMeta> {
        vec![
            AccountMeta::new(payer, true), // payer signer + writable

            AccountMeta::new(Self::derive_global_state(), false),
            AccountMeta::new(Self::derive_vault_authority(), false),

            AccountMeta::new(GOLDC_MINT, false),
            AccountMeta::new(payer_goldc_ata, false),

            AccountMeta::new(VNX_MINT, false),
            AccountMeta::new(payer_vnx_ata, false),

            AccountMeta::new(vnx_vault, false),

            AccountMeta::new(Self::derive_user_data(&payer), false),

            AccountMeta::new_readonly(spl_token::ID, false),
            AccountMeta::new_readonly(spl_associated_token_account::ID, false),
            AccountMeta::new_readonly(solana_sdk::system_program::ID, false),
        ]
    }

    /// Which direction is implied by the swap params source mint
    fn direction_from_source_mint(source_mint: Pubkey) -> Result<DeauraDirection> {
        if source_mint == VNX_MINT {
            Ok(DeauraDirection::Deposit)
        } else if source_mint == GOLDC_MINT {
            Ok(DeauraDirection::Redeem)
        } else {
            Err(anyhow!("Unsupported source mint for DeauraAmm: {source_mint}"))
        }
    }
}

impl Amm for DeauraAmm {
    fn from_keyed_account(keyed_account: &KeyedAccount, _amm_context: &AmmContext) -> Result<Self> {
        // We create two AMM instances by listing both vault accounts as "markets" to Jupiter.
        // The aggregator will call this constructor per keyed account.
        let key = keyed_account.key;

        let (direction, label) = if key == VNX_DEPOSIT_VAULT {
            (DeauraDirection::Deposit, "Deaura Vault (VNX→GOLDC)".to_string())
        } else if key == VNX_REDEEM_VAULT {
            (DeauraDirection::Redeem, "Deaura Vault (GOLDC→VNX)".to_string())
        } else {
            return Err(anyhow!(
                "Unknown Deaura vault account passed into from_keyed_account: {key}"
            ));
        };

        Ok(Self {
            key,
            label,
            program_id: DEAURA_PROGRAM_ID,
            vnx_vault: key,
            direction,
            vnx_reserve: 0,
        })
    }

    fn label(&self) -> String {
        self.label.clone()
    }

    fn program_id(&self) -> Pubkey {
        self.program_id
    }

    fn key(&self) -> Pubkey {
        self.key
    }

    fn get_reserve_mints(&self) -> Vec<Pubkey> {
        // Treat this AMM as supporting VNX <-> GOLDC
        vec![VNX_MINT, GOLDC_MINT]
    }

    fn get_accounts_to_update(&self) -> Vec<Pubkey> {
        // Only real "liquidity" gating here is VNX vault balance (for redeem direction).
        // For deposit direction, vault balance isn't required to mint GOLDC.
        vec![self.vnx_vault]
    }

    fn update(&mut self, account_map: &AccountMap) -> Result<()> {
        let vnx_vault_acc_data = try_get_account_data(account_map, &self.vnx_vault)?;
        let token_acc = TokenAccount::unpack(vnx_vault_acc_data)?;
        self.vnx_reserve = token_acc.amount.into();
        Ok(())
    }

    fn quote(&self, quote_params: &QuoteParams) -> Result<Quote> {
        // This is a placeholder 1:1 quote (same behavior you described).
        // If you have a dynamic conversion rate or fees, update here.

        // If redeeming, optionally enforce vault liquidity:
        if quote_params.input_mint == GOLDC_MINT {
            ensure!(
                (quote_params.amount as u128) <= self.vnx_reserve,
                "Insufficient VNX liquidity in redeem vault"
            );
        }

        Ok(Quote {
            fee_pct: Decimal::ZERO,
            in_amount: quote_params.amount,
            out_amount: quote_params.amount,
            fee_amount: 0,
            fee_mint: quote_params.input_mint,
        })
    }

    fn get_accounts_len(&self) -> usize {
        // 12 accounts as per IDL order
        12
    }

    fn get_swap_and_account_metas(&self, swap_params: &SwapParams) -> Result<SwapAndAccountMetas> {
        let SwapParams {
            source_mint,
            source_token_account,
            destination_token_account,
            token_transfer_authority,
            in_amount,
            ..
        } = swap_params;

        // Jupiter passes user ATAs in swap_params.
        // Your program requires payer's GOLDC ATA and payer's VNX ATA explicitly.
        //
        // IMPORTANT:
        // - For Deposit (VNX->GOLDC): source_token_account should be payer_vnx_ata, destination should be payer_goldc_ata
        // - For Redeem (GOLDC->VNX): source_token_account should be payer_goldc_ata, destination should be payer_vnx_ata
        let direction = Self::direction_from_source_mint(*source_mint)?;

        let (payer_vnx_ata, payer_goldc_ata, vnx_vault, ix_disc) = match direction {
            DeauraDirection::Deposit => (
                *source_token_account,
                *destination_token_account,
                VNX_DEPOSIT_VAULT,
                DEPOSIT_IX_DISC,
            ),
            DeauraDirection::Redeem => (
                *destination_token_account,
                *source_token_account,
                VNX_REDEEM_VAULT,
                REDEEM_IX_DISC,
            ),
        };

        // In Jupiter, `token_transfer_authority` is the signer PDA/authority used to move user tokens.
        // Your program expects `payer` to be a signer. In Jupiter integrations, the route's "user"
        // is the authority that signs the full transaction (wallet), so we set payer = token_transfer_authority
        // ONLY if your integration is configured to make that be the user's signer.
        //
        // Typically, token_transfer_authority == user wallet in Jupiter's direct swap flow.
        // If not, you must ensure swap_params provides the actual user signer.
        let payer = *token_transfer_authority;

        let metas = Self::account_metas(payer, payer_goldc_ata, payer_vnx_ata, vnx_vault);

        // Single CPI call to your program, which internally performs deposit or redeem.
        let ix = Instruction {
            program_id: DEAURA_PROGRAM_ID,
            accounts: metas.clone(),
            data: Self::ix_data(ix_disc, *in_amount),
        };

        Ok(SwapAndAccountMetas {
            // Use TokenSwap as a generic swap type for custom AMM implementations
            swap: Swap::TokenSwap,
            account_metas: ix.accounts,
        })
    }

    fn clone_amm(&self) -> Box<dyn Amm + Send + Sync> {
        Box::new(Self {
            key: self.key,
            label: self.label.clone(),
            program_id: self.program_id,
            vnx_vault: self.vnx_vault,
            direction: self.direction,
            vnx_reserve: self.vnx_reserve,
        })
    }
}
