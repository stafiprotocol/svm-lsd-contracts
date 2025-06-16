pub use crate::errors::Errors;
pub use crate::StakeManager;
use crate::{helper, EraStatus};
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use staking_program;

#[derive(Accounts)]
#[instruction(params: InitializeStakeManagerParams)]
pub struct InitializeStakeManager<'info> {
    pub admin: Signer<'info>,

    #[account(mut)]
    pub rent_payer: Signer<'info>,

    #[account(
        init,
        space = 2048,
        payer = rent_payer,
        seeds = [
            helper::STAKE_MANAGER_SEED,
            &admin.key().to_bytes(),
            &[params.index],
        ],
        bump,
    )]
    pub stake_manager: Box<Account<'info, StakeManager>>,

    pub staking_pool: Box<Account<'info, staking_program::StakingPool>>,

    #[account(
        init,
        payer = rent_payer,
        mint::decimals = 9,
        mint::authority = stake_manager,
        mint::freeze_authority = stake_manager,
        seeds = [
            helper::TOKEN_MINT_SEED,
            &admin.key().to_bytes(),
            &[params.index],
        ],
        bump,
    )]
    pub lsd_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        address = staking_pool.token_mint @Errors::StakingTokenMintAccountNotMatch
    )]
    pub staking_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        init,
        payer  = rent_payer,
        associated_token::mint = staking_token_mint,
        associated_token::authority = stake_manager,
        associated_token::token_program = token_program,
    )]
    pub pool_staking_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InitializeStakeManagerParams {
    pub era_seconds: i64,
    pub index: u8,
}

impl<'info> InitializeStakeManager<'info> {
    pub fn process(
        &mut self,
        params: InitializeStakeManagerParams,
        pool_seed_bump: u8,
    ) -> Result<()> {
        require_gt!(params.era_seconds, 0, Errors::ParamsNotMatch);

        let timestamp = Clock::get().unwrap().unix_timestamp;
        let offset = 0 - timestamp / params.era_seconds;

        let unbonding_duration =
            self.staking_pool.unbonding_seconds / params.era_seconds as u64 + 1;

        self.stake_manager.set_inner(StakeManager {
            creator: self.admin.key(),
            admin: self.admin.key(),
            index: params.index,
            pending_admin: Pubkey::default(),
            lsd_token_mint: self.lsd_token_mint.key(),
            staking_token_mint: self.staking_pool.token_mint.key(),
            staking_program: self.staking_pool.to_account_info().owner.key(),
            staking_pool: self.staking_pool.key(),
            staking_min_stake_amount: self.staking_pool.min_stake_amount,
            pool_seed_bump,
            era_seconds: params.era_seconds,
            era_offset: offset,
            min_stake_amount: helper::DEFAULT_MIN_STAKE_AMOUNT,
            platform_fee_commission: helper::DEFAULT_PLATFORM_FEE_COMMISSION,
            rate_change_limit: helper::DEFAULT_RATE_CHANGE_LIMIT,
            unbonding_duration,
            era_status: EraStatus::ActiveUpdated,
            latest_era: 0,
            rate: helper::DEFAULT_RATE,
            era_bond: 0,
            era_unbond: 0,
            pending_bond: 0,
            pending_unbond: 0,
            active: 0,
            total_platform_fee: 0,
            era_rates: vec![],
            _reserved: [0u8; 256],
        });

        Ok(())
    }
}
