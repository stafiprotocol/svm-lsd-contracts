use crate::{helper, Errors, StakeManager};
use anchor_lang::context::CpiContext;
use anchor_lang::prelude::*;

use anchor_spl::{
    metadata::{
        create_metadata_accounts_v3, mpl_token_metadata::types::DataV2, CreateMetadataAccountsV3,
        Metadata,
    },
    token_interface::Mint,
};

#[derive(Accounts)]
pub struct CreateMetadataAccountV3<'info> {
    #[account(mut)]
    pub fee_and_rent_payer: Signer<'info>,

    /// CHECK: Validate address by deriving pda
    #[account(
        mut,
        seeds = [b"metadata", token_metadata_program.key().as_ref(), lsd_token_mint.key().as_ref()],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    pub metadata_account: UncheckedAccount<'info>,

    pub admin: Signer<'info>,
    #[account(
        mut,
        has_one = admin @ Errors::AdminNotMatch
    )]
    pub stake_manager: Box<Account<'info, StakeManager>>,

    #[account(
        mut,
        address = stake_manager.lsd_token_mint @Errors::LsdTokenMintAccountNotMatch
    )]
    pub lsd_token_mint: Box<InterfaceAccount<'info, Mint>>,
    pub token_metadata_program: Program<'info, Metadata>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CreateMetadataParams {
    token_name: String,
    token_symbol: String,
    token_uri: String,
}

impl<'info> CreateMetadataAccountV3<'info> {
    pub fn process(&mut self, config_metadata_params: CreateMetadataParams) -> Result<()> {
        msg!("Creating metadata v3");

        let signer_seeds: &[&[&[u8]]] = &[&[
            helper::STAKE_MANAGER_SEED,
            &self.stake_manager.creator.to_bytes(),
            &[self.stake_manager.index],
            &[self.stake_manager.pool_seed_bump],
        ]];

        create_metadata_accounts_v3(
            CpiContext::new(
                self.token_metadata_program.to_account_info(),
                CreateMetadataAccountsV3 {
                    metadata: self.metadata_account.to_account_info(),
                    mint: self.lsd_token_mint.to_account_info(),
                    mint_authority: self.stake_manager.to_account_info(),
                    update_authority: self.stake_manager.to_account_info(),
                    payer: self.fee_and_rent_payer.to_account_info(),
                    system_program: self.system_program.to_account_info(),
                    rent: self.rent.to_account_info(),
                },
            )
            .with_signer(signer_seeds),
            DataV2 {
                name: config_metadata_params.token_name,
                symbol: config_metadata_params.token_symbol,
                uri: config_metadata_params.token_uri,
                seller_fee_basis_points: 0,
                creators: None,
                collection: None,
                uses: None,
            },
            true, // Is mutable
            true, // Update authority is signer
            None, // Collection details
        )?;

        Ok(())
    }
}
