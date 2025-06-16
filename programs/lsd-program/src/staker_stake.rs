use crate::{helper, Errors, StakeManager};
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{
    mint_to, transfer_checked, Mint, MintTo, TokenAccount, TokenInterface, TransferChecked,
};

#[derive(Accounts)]
pub struct Stake<'info> {
    pub user: Signer<'info>,

    #[account(mut)]
    pub rent_payer: Signer<'info>,

    #[account(mut)]
    pub stake_manager: Box<Account<'info, StakeManager>>,

    #[account(
        mut,
        address = stake_manager.lsd_token_mint @Errors::LsdTokenMintAccountNotMatch
    )]
    pub lsd_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        address = stake_manager.staking_token_mint @Errors::StakingTokenMintAccountNotMatch
    )]
    pub staking_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        init_if_needed,
        payer  = rent_payer,
        associated_token::mint = lsd_token_mint,
        associated_token::authority = user,
        associated_token::token_program = token_program,
    )]
    pub user_lsd_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
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
    pub pool_staking_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[event]
pub struct EventStake {
    pub era: u64,
    pub staker: Pubkey,
    pub stake_amount: u64,
    pub lsd_token_amount: u64,
    pub stake_manager: Pubkey,
}

impl<'info> Stake<'info> {
    pub fn process(&mut self, stake_amount: u64) -> Result<()> {
        require_gte!(
            stake_amount,
            self.stake_manager.min_stake_amount,
            Errors::StakeAmountTooLow
        );

        // transfer staking token to the pool
        let transfer_from_user_to_pool_cpi_context = CpiContext::new(
            self.token_program.to_account_info(),
            TransferChecked {
                from: self.user_staking_token_account.to_account_info(),
                mint: self.staking_token_mint.to_account_info(),
                to: self.pool_staking_token_account.to_account_info(),
                authority: self.user.to_account_info(),
            },
        );
        transfer_checked(
            transfer_from_user_to_pool_cpi_context,
            stake_amount,
            self.staking_token_mint.decimals,
        )?;

        let lsd_token_amount = self.stake_manager.calc_lsd_token_amount(stake_amount)?;
        self.stake_manager.era_bond += stake_amount;
        self.stake_manager.active += stake_amount;

        // mint lsd token
        mint_to(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                MintTo {
                    mint: self.lsd_token_mint.to_account_info(),
                    to: self.user_lsd_token_account.to_account_info(),
                    authority: self.stake_manager.to_account_info(),
                },
                &[&[
                    helper::STAKE_MANAGER_SEED,
                    &self.stake_manager.creator.to_bytes(),
                    &[self.stake_manager.index],
                    &[self.stake_manager.pool_seed_bump],
                ]],
            ),
            lsd_token_amount,
        )?;

        emit!(EventStake {
            era: self.stake_manager.latest_era,
            staker: self.user.key(),
            stake_amount,
            lsd_token_amount,
            stake_manager: self.stake_manager.key(),
        });
        Ok(())
    }
}
