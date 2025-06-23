use crate::{helper, Errors, StakeManager, UnstakeAccount};
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(
        address = unstake_account.user @ Errors::UnstakeUserNotMatch
    )]
    pub user: Signer<'info>,

    #[account(mut)]
    pub rent_payer: Signer<'info>,

    #[account(mut)]
    pub stake_manager: Box<Account<'info, StakeManager>>,

    #[account(
        mut,
        close = rent_payer,
        has_one = stake_manager @Errors::InvalidUnstakeAccount,
    )]
    pub unstake_account: Account<'info, UnstakeAccount>,

    #[account(
        address = stake_manager.staking_token_mint @Errors::StakingTokenMintAccountNotMatch
    )]
    pub staking_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        init_if_needed,
        payer = rent_payer,
        associated_token::mint = staking_token_mint,
        associated_token::authority = user,
        associated_token::token_program = token_program,
    )]
    pub user_staking_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = staking_token_mint,
        associated_token::authority = stake_manager,
        associated_token::token_program = token_program,
    )]
    pub stake_manager_staking_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[event]
pub struct EventWithdraw {
    pub era: u64,
    pub user: Pubkey,
    pub unstake_account: Pubkey,
    pub withdraw_amount: u64,
    pub stake_manager: Pubkey,
}

impl<'info> Withdraw<'info> {
    pub fn process(&mut self) -> Result<()> {
        require_gt!(
            self.unstake_account.amount,
            0,
            Errors::UnstakeAccountAmountZero
        );
        require_gte!(
            self.stake_manager.latest_era,
            self.unstake_account.withdrawable_era,
            Errors::UnstakeAccountNotWithdrawable
        );

        let withdraw_amount = self.unstake_account.amount;

        require_gte!(
            self.stake_manager_staking_token_account.amount,
            withdraw_amount,
            Errors::PoolBalanceNotEnough
        );

        self.unstake_account.amount = 0;

        transfer_checked(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                TransferChecked {
                    from: self.stake_manager_staking_token_account.to_account_info(),
                    mint: self.staking_token_mint.to_account_info(),
                    to: self.user_staking_token_account.to_account_info(),
                    authority: self.stake_manager.to_account_info(),
                },
                &[&[
                    helper::STAKE_MANAGER_SEED,
                    &self.stake_manager.creator.to_bytes(),
                    &[self.stake_manager.index],
                    &[self.stake_manager.pool_seed_bump],
                ]],
            ),
            withdraw_amount,
            self.staking_token_mint.decimals,
        )?;

        emit!(EventWithdraw {
            era: self.stake_manager.latest_era,
            user: self.user.key(),
            unstake_account: self.unstake_account.key(),
            withdraw_amount,
            stake_manager: self.stake_manager.key()
        });
        Ok(())
    }
}
