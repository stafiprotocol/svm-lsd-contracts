use crate::{helper, Errors, StakeManager};
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

#[derive(Accounts)]
pub struct EraWithdraw<'info> {
    #[account(mut)]
    pub fee_and_rent_payer: Signer<'info>,

    #[account(mut)]
    pub stake_manager: Box<Account<'info, StakeManager>>,

    #[account(
        mut,
        address = stake_manager.staking_pool @Errors::SpStakePoolNotMatch,
    )]
    pub staking_pool: Box<Account<'info, staking_program::StakingPool>>,

    #[account(
        address = stake_manager.staking_token_mint @Errors::StakingTokenMintAccountNotMatch
    )]
    pub staking_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(mut)]
    pub stake_manager_staking_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(mut)]
    pub staking_pool_staking_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(mut)]
    pub staking_unstake_account: Box<Account<'info, staking_program::UnstakeAccount>>,

    /// CHECK: staking_program
    #[account(
        address = stake_manager.staking_program @Errors::SpNotMatch
    )]
    pub staking_program: AccountInfo<'info>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[event]
pub struct EventEraWithdraw {
    pub era: u64,
}

impl<'info> EraWithdraw<'info> {
    pub fn process(&mut self) -> Result<()> {
        let timestamp = Clock::get()?.unix_timestamp as u64;
        require!(
            self.staking_unstake_account.withdrawable_timestamp <= timestamp,
            Errors::UnstakeAccountNotWithdrawable
        );

        let cpi_accounts = staking_program::cpi::accounts::Withdraw {
            user: self.stake_manager.to_account_info(),
            rent_payer: self.fee_and_rent_payer.to_account_info(),
            staking_pool: self.staking_pool.to_account_info(),
            token_mint: self.staking_token_mint.to_account_info(),
            user_token_account: self.stake_manager_staking_token_account.to_account_info(),
            pool_token_account: self.staking_pool_staking_token_account.to_account_info(),
            unstake_account: self.staking_unstake_account.to_account_info(),
            token_program: self.token_program.to_account_info(),
            associated_token_program: self.associated_token_program.to_account_info(),
            system_program: self.system_program.to_account_info(),
        };

        staking_program::cpi::withdraw(CpiContext::new_with_signer(
            self.staking_program.to_account_info(),
            cpi_accounts,
            &[&[
                helper::STAKE_MANAGER_SEED,
                &self.stake_manager.creator.to_bytes(),
                &[self.stake_manager.index],
                &[self.stake_manager.pool_seed_bump],
            ]],
        ))?;

        emit!(EventEraWithdraw {
            era: self.stake_manager.latest_era,
        });
        Ok(())
    }
}
