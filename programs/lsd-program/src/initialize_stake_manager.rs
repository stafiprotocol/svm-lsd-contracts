pub use crate::errors::Errors;
pub use crate::StakeManager;
use crate::{helper, EraStatus};
use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;
use staking_program;

#[derive(Accounts)]
pub struct InitializeStakeManager<'info> {
    pub admin: Signer<'info>,

    #[account(mut)]
    pub rent_payer: Signer<'info>,

    #[account(
        zero,
        rent_exempt = enforce,
    )]
    pub stake_manager: Box<Account<'info, StakeManager>>,

    #[account(
        seeds = [
            helper::POOL_SEED,
            &stake_manager.key().to_bytes(),
        ],
        bump,
    )]
    pub stake_pool: SystemAccount<'info>,

    pub staking_pool: Box<Account<'info, staking_program::StakingPool>>,

    pub lsd_token_mint: Box<InterfaceAccount<'info, Mint>>,

    pub system_program: Program<'info, System>,
}

#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InitializeStakeManagerParams {
    pub era_seconds: i64,
}

impl<'info> InitializeStakeManager<'info> {
    pub fn process(
        &mut self,
        params: InitializeStakeManagerParams,
        pool_seed_bump: u8,
    ) -> Result<()> {
        require_gt!(params.era_seconds, 0, Errors::ParamsNotMatch);

        require!(
            self.lsd_token_mint
                .mint_authority
                .contains(&self.stake_pool.key()),
            Errors::MintAuthorityNotMatch
        );
        require!(
            self.lsd_token_mint.freeze_authority.is_none(),
            Errors::FreezeAuthorityNotMatch
        );
        require!(self.lsd_token_mint.supply == 0, Errors::MintSupplyNotEmpty);
        let timestamp = Clock::get().unwrap().unix_timestamp;
        let offset = 0 - timestamp / params.era_seconds;

        let unbonding_duration =
            self.staking_pool.unbonding_seconds / params.era_seconds as u64 + 1;

        self.stake_manager.set_inner(StakeManager {
            admin: self.admin.key(),
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
