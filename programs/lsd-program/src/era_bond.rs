use crate::{helper, EraStatus, Errors, StakeManager};
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use staking_program;

#[derive(Accounts)]
pub struct EraBond<'info> {
    #[account(mut)]
    pub fee_and_rent_payer: Signer<'info>,

    #[account(mut)]
    pub stake_manager: Box<Account<'info, StakeManager>>,

    #[account(
        seeds = [
            helper::POOL_SEED,
            &stake_manager.key().to_bytes(),
        ],
        bump = stake_manager.pool_seed_bump
    )]
    pub stake_pool: SystemAccount<'info>,

    #[account(
        address = stake_manager.staking_token_mint @Errors::StakingTokenMintAccountNotMatch
    )]
    pub staking_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        associated_token::mint = staking_token_mint,
        associated_token::authority = stake_pool,
        associated_token::token_program = token_program,
    )]
    pub pool_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        address = stake_manager.staking_program @Errors::SpStakePoolNotMatch,
    )]
    pub staking_pool: Box<Account<'info, staking_program::StakingPool>>,

    pub staking_pool_vault_signer: SystemAccount<'info>,

    #[account(mut)]
    pub staking_pool_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(mut)]
    pub staking_stake_account: Box<Account<'info, staking_program::StakeAccount>>,

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
pub struct EventEraBond {
    pub era: u64,
}

impl<'info> EraBond<'info> {
    pub fn process(&mut self) -> Result<()> {
        require!(
            self.stake_manager.era_status == EraStatus::EraUpdated,
            Errors::EraStatusNotMatch
        );
        require!(
            self.stake_manager.pending_bond > self.stake_manager.pending_unbond,
            Errors::EraStatusNotMatch
        );

        let diff = self.stake_manager.pending_bond - self.stake_manager.pending_unbond;
        require!(
            diff >= self.stake_manager.staking_min_stake_amount,
            Errors::EraStatusNotMatch
        );

        let cpi_accounts = staking_program::cpi::accounts::Stake {
            user: self.stake_pool.to_account_info(),
            rent_payer: self.fee_and_rent_payer.to_account_info(),
            staking_pool: self.staking_pool.to_account_info(),
            pool_vault_signer: self.staking_pool_vault_signer.to_account_info(),
            token_mint: self.staking_token_mint.to_account_info(),
            user_token_account: self.pool_token_account.to_account_info(),
            pool_token_account: self.staking_pool_token_account.to_account_info(),
            stake_account: self.staking_stake_account.to_account_info(),
            token_program: self.token_program.to_account_info(),
            associated_token_program: self.associated_token_program.to_account_info(),
            system_program: self.system_program.to_account_info(),
        };

        staking_program::cpi::stake(
            CpiContext::new_with_signer(
                self.staking_program.to_account_info(),
                cpi_accounts,
                &[&[
                    helper::POOL_SEED,
                    &self.stake_manager.key().to_bytes(),
                    &[self.stake_manager.pool_seed_bump],
                ]],
            ),
            diff,
        )?;

        self.stake_manager.pending_bond = 0;
        self.stake_manager.pending_unbond = 0;
        self.stake_manager.era_status = EraStatus::Bonded;

        emit!(EventEraBond {
            era: self.stake_manager.latest_era,
        });

        Ok(())
    }
}
