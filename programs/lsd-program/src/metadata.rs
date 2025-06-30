use crate::{helper, Errors, StakeManager};
use anchor_lang::prelude::*;
use anchor_spl::token_interface::TokenInterface;
use anchor_spl::{metadata::Metadata, token_interface::Mint};
use mpl_token_metadata::{
    instructions::CreateV1Cpi, instructions::CreateV1CpiAccounts,
    instructions::CreateV1InstructionArgs, types::TokenStandard,
};

#[derive(Accounts)]
pub struct CreateMetadataV1<'info> {
    #[account(mut)]
    pub fee_and_rent_payer: Signer<'info>,

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

    /// CHECK: Validate address by deriving pda
    #[account(
         mut,
         seeds = [b"metadata", metadata_program.key().as_ref(), lsd_token_mint.key().as_ref()],
         bump,
         seeds::program = metadata_program.key(),
     )]
    pub metadata_account: UncheckedAccount<'info>,

    pub token_program: Interface<'info, TokenInterface>,
    pub metadata_program: Program<'info, Metadata>,
    pub system_program: Program<'info, System>,
    /// CHECK:
    pub sysvar_instruction: UncheckedAccount<'info>,
}

#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CreateMetadataParams {
    token_name: String,
    token_symbol: String,
    token_uri: String,
}

impl<'info> CreateMetadataV1<'info> {
    pub fn process(&mut self, config_metadata_params: CreateMetadataParams) -> Result<()> {
        msg!("Creating metadata v1");

        let signer_seeds: &[&[&[u8]]] = &[&[
            helper::STAKE_MANAGER_SEED,
            &self.stake_manager.creator.to_bytes(),
            &[self.stake_manager.index],
            &[self.stake_manager.pool_seed_bump],
        ]];
        CreateV1Cpi::new(
            &self.metadata_program.to_account_info(),
            CreateV1CpiAccounts {
                metadata: &self.metadata_account.to_account_info(),
                master_edition: None,
                mint: (&self.lsd_token_mint.to_account_info(), false),
                authority: &self.stake_manager.to_account_info(),
                payer: &self.fee_and_rent_payer.to_account_info(),
                update_authority: (&self.stake_manager.to_account_info(), false),
                system_program: &self.system_program.to_account_info(),
                sysvar_instructions: &self.sysvar_instruction.to_account_info(),
                spl_token_program: &self.token_program.to_account_info(),
            },
            CreateV1InstructionArgs {
                name: config_metadata_params.token_name,
                symbol: config_metadata_params.token_symbol,
                uri: config_metadata_params.token_uri,
                seller_fee_basis_points: 0,
                creators: None,
                primary_sale_happened: false,
                is_mutable: true,
                token_standard: TokenStandard::Fungible,
                collection: None,
                uses: None,
                collection_details: None,
                rule_set: None,
                decimals: None,
                print_supply: None,
            },
        )
        .invoke_signed(signer_seeds)?;

        Ok(())
    }
}
