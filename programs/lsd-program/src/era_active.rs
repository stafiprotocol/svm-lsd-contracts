use crate::{helper, EraRate, EraStatus, Errors, StakeManager};
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{mint_to, Mint, MintTo, TokenAccount, TokenInterface};

#[derive(Accounts)]
pub struct EraActive<'info> {
    #[account(mut)]
    pub rent_payer: Signer<'info>,

    /// CHECK: stake_manager
    #[account(
        mut,
        address = stake_manager.admin @Errors::AdminNotMatch
    )]
    pub admin: AccountInfo<'info>,

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
        mut,
        associated_token::mint = staking_token_mint,
        associated_token::authority = stake_manager,
        associated_token::token_program = token_program,
    )]
    pub pool_staking_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = rent_payer,
        associated_token::mint = lsd_token_mint,
        associated_token::authority = admin,
        associated_token::token_program = token_program,
    )]
    pub platform_fee_recipient: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        address = stake_manager.staking_program @Errors::SpStakePoolNotMatch,
    )]
    pub staking_pool: Box<Account<'info, staking_program::StakingPool>>,

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
pub struct EventEraActive {
    pub era: u64,
    pub rate: u64,
    pub platform_fee: u64,
}

impl<'info> EraActive<'info> {
    pub fn process(&mut self) -> Result<()> {
        require!(
            self.stake_manager.era_status == EraStatus::Bonded
                || self.stake_manager.era_status == EraStatus::Unbonded,
            Errors::EraStatusNotMatch
        );

        let cpi_accounts = staking_program::cpi::accounts::Claim {
            user: self.stake_manager.to_account_info(),
            rent_payer: self.rent_payer.to_account_info(),
            staking_pool: self.staking_pool.to_account_info(),
            token_mint: self.staking_token_mint.to_account_info(),
            user_token_account: self.pool_staking_token_account.to_account_info(),
            pool_token_account: self.staking_pool_token_account.to_account_info(),
            stake_account: self.staking_stake_account.to_account_info(),
            token_program: self.token_program.to_account_info(),
            associated_token_program: self.associated_token_program.to_account_info(),
            system_program: self.system_program.to_account_info(),
        };

        staking_program::cpi::claim(
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
            true,
        )?;

        let total_bond_and_reward = self.staking_stake_account.amount
            + self.stake_manager.pending_bond
            + self.stake_manager.era_bond;
        let total_unbond = self.stake_manager.era_unbond;

        require_gte!(total_bond_and_reward, total_unbond, Errors::CalculationFail);

        let new_active = total_bond_and_reward - total_unbond;

        let reward = if new_active > self.stake_manager.active {
            new_active - self.stake_manager.active
        } else {
            0
        };

        let platform_fee = self.stake_manager.calc_platform_fee(reward)?;
        if platform_fee > 0 {
            mint_to(
                CpiContext::new_with_signer(
                    self.token_program.to_account_info(),
                    MintTo {
                        mint: self.lsd_token_mint.to_account_info(),
                        to: self.platform_fee_recipient.to_account_info(),
                        authority: self.stake_manager.to_account_info(),
                    },
                    &[&[
                        helper::STAKE_MANAGER_SEED,
                        &self.stake_manager.creator.to_bytes(),
                        &[self.stake_manager.index],
                        &[self.stake_manager.pool_seed_bump],
                    ]],
                ),
                platform_fee,
            )?;

            self.stake_manager.total_platform_fee += platform_fee;
            self.lsd_token_mint.reload()?;
        }

        let new_rate = self
            .stake_manager
            .calc_rate(new_active, self.lsd_token_mint.supply)?;
        let rate_change = self
            .stake_manager
            .calc_rate_change(self.stake_manager.rate, new_rate)?;

        if self.stake_manager.rate_change_limit > 0 {
            require_gte!(
                self.stake_manager.rate_change_limit,
                rate_change,
                Errors::RateChangeOverLimit
            );
        }
        self.stake_manager.active = new_active;
        self.stake_manager.rate = new_rate;

        let latest_era = self.stake_manager.latest_era;
        self.stake_manager.era_rates.push(EraRate {
            era: latest_era,
            rate: new_rate,
        });
        if self.stake_manager.era_rates.len() > helper::ERA_RATES_LEN_LIMIT as usize {
            self.stake_manager.era_rates.remove(0);
        }

        self.stake_manager.era_status = EraStatus::ActiveUpdated;

        emit!(EventEraActive {
            era: self.stake_manager.latest_era,
            rate: new_rate,
            platform_fee: platform_fee
        });

        Ok(())
    }
}
