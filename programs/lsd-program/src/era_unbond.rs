use crate::{helper, EraStatus, Errors, StakeManager};
use anchor_lang::prelude::*;
use staking_program;

#[derive(Accounts)]
pub struct EraUnbond<'info> {
    #[account(mut)]
    pub fee_and_rent_payer: Signer<'info>,

    #[account(mut)]
    pub stake_manager: Box<Account<'info, StakeManager>>,

    #[account(
        mut,
        address = stake_manager.staking_pool @Errors::SpStakePoolNotMatch,
    )]
    pub staking_pool: Box<Account<'info, staking_program::StakingPool>>,

    #[account(mut)]
    pub staking_stake_account: Box<Account<'info, staking_program::StakeAccount>>,

    #[account(mut)]
    pub staking_unstake_account: Box<Account<'info, staking_program::UnstakeAccount>>,

    /// CHECK: staking_program
    #[account(
        address = stake_manager.staking_program @Errors::SpNotMatch
    )]
    pub staking_program: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

#[event]
pub struct EventEraUnbond {
    pub era: u64,
}

impl<'info> EraUnbond<'info> {
    pub fn process(&mut self) -> Result<()> {
        require!(
            self.stake_manager.era_status == EraStatus::EraUpdated,
            Errors::EraStatusNotMatch
        );
        require!(
            self.stake_manager.pending_bond < self.stake_manager.pending_unbond,
            Errors::EraStatusNotMatch
        );

        let diff = self.stake_manager.pending_unbond - self.stake_manager.pending_bond;

        let cpi_accounts = staking_program::cpi::accounts::Unstake {
            user: self.stake_manager.to_account_info(),
            rent_payer: self.fee_and_rent_payer.to_account_info(),
            staking_pool: self.staking_pool.to_account_info(),
            stake_account: self.staking_stake_account.to_account_info(),
            unstake_account: self.staking_unstake_account.to_account_info(),
            system_program: self.system_program.to_account_info(),
        };

        staking_program::cpi::unstake(
            CpiContext::new_with_signer(
                self.staking_program.to_account_info(),
                cpi_accounts,
                &[&[
                    helper::STAKE_MANAGER_SEED,
                    &self.stake_manager.creator.to_bytes(),
                    &[self.stake_manager.index],
                    &[self.stake_manager.pool_seed_bump],
                ]],
            ),
            diff,
        )?;

        self.stake_manager.pending_bond = 0;
        self.stake_manager.pending_unbond = 0;
        self.stake_manager.era_status = EraStatus::Unbonded;

        emit!(EventEraUnbond {
            era: self.stake_manager.latest_era,
        });

        Ok(())
    }
}
