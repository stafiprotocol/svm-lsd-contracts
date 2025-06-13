use crate::{Errors, StakeManager, UnstakeAccount};
use anchor_lang::{prelude::*, solana_program::system_program};
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{burn, Burn, Mint, TokenAccount, TokenInterface};

#[derive(Accounts)]
pub struct Unstake<'info> {
    pub user: Signer<'info>,

    #[account(
        mut,
        owner = system_program::ID
    )]
    pub rent_payer: Signer<'info>,

    #[account(mut)]
    pub stake_manager: Box<Account<'info, StakeManager>>,

    #[account(
        mut,
        address = stake_manager.lsd_token_mint @Errors::LsdTokenMintAccountNotMatch,
    )]
    pub lsd_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        associated_token::mint = lsd_token_mint,
        associated_token::authority = user,
        associated_token::token_program = token_program,
    )]
    pub user_lsd_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init,
        space = 8 + std::mem::size_of::<UnstakeAccount>(),
        payer = rent_payer,
        rent_exempt = enforce,
    )]
    pub unstake_account: Box<Account<'info, UnstakeAccount>>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[event]
pub struct EventUnstake {
    pub era: u64,
    pub staker: Pubkey,
    pub unstake_account: Pubkey,
    pub unstake_amount: u64,
    pub staking_token_amount: u64,
    pub stake_manager: Pubkey,
}

impl<'info> Unstake<'info> {
    pub fn process(&mut self, unstake_amount: u64) -> Result<()> {
        require_gt!(unstake_amount, 0, Errors::UnstakeAmountIsZero);

        require_gte!(
            self.user_lsd_token_account.amount,
            unstake_amount,
            Errors::BalanceNotEnough
        );

        let staking_token_amount = self
            .stake_manager
            .calc_staking_token_amount(unstake_amount)?;
        require_gt!(staking_token_amount, 0, Errors::UnstakeAccountAmountZero);

        self.stake_manager.era_unbond += staking_token_amount;
        self.stake_manager.active -= staking_token_amount;

        // burn lsd token
        burn(
            CpiContext::new(
                self.token_program.to_account_info(),
                Burn {
                    mint: self.lsd_token_mint.to_account_info(),
                    from: self.user_lsd_token_account.to_account_info(),
                    authority: self.user.to_account_info(),
                },
            ),
            unstake_amount,
        )?;

        self.unstake_account.set_inner(UnstakeAccount {
            stake_manager: self.stake_manager.key(),
            user: self.user.key(),
            amount: staking_token_amount,
            withdrawable_era: self.stake_manager.latest_era + self.stake_manager.unbonding_duration,
        });

        emit!(EventUnstake {
            era: self.stake_manager.latest_era,
            staker: self.user.key(),
            unstake_account: self.unstake_account.key(),
            unstake_amount,
            staking_token_amount,
            stake_manager: self.stake_manager.key(),
        });

        Ok(())
    }
}
