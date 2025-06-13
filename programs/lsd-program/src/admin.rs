use crate::{Errors, StakeManager};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct TransferStakeManagerAdmin<'info> {
    pub admin: Signer<'info>,

    #[account(
        mut,
        has_one = admin @ Errors::AdminNotMatch
    )]
    pub stake_manager: Box<Account<'info, StakeManager>>,
}

impl<'info> TransferStakeManagerAdmin<'info> {
    pub fn process(&mut self, new_admin: Pubkey) -> Result<()> {
        self.stake_manager.pending_admin = new_admin;

        msg!("NewAdmin: {}", new_admin);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct AcceptStakeManagerAdmin<'info> {
    pub pending_admin: Signer<'info>,

    #[account(
        mut,
        has_one = pending_admin @ Errors::PendingAdminNotMatch
    )]
    pub stake_manager: Box<Account<'info, StakeManager>>,
}

impl<'info> AcceptStakeManagerAdmin<'info> {
    pub fn process(&mut self) -> Result<()> {
        self.stake_manager.admin = self.stake_manager.pending_admin;
        self.stake_manager.pending_admin = Pubkey::default();

        msg!("AcceptAdmin: {}", self.stake_manager.admin);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct ConfigStakeManager<'info> {
    pub admin: Signer<'info>,

    #[account(
        mut,
        has_one = admin @ Errors::AdminNotMatch
    )]
    pub stake_manager: Box<Account<'info, StakeManager>>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub struct ConfigStakeManagerParams {
    pub min_stake_amount: Option<u64>,
    pub platform_fee_commission: Option<u64>,
    pub rate_change_limit: Option<u64>,
}

impl<'info> ConfigStakeManager<'info> {
    pub fn process(&mut self, config_stake_manager_params: ConfigStakeManagerParams) -> Result<()> {
        if let Some(min_stake_amount) = config_stake_manager_params.min_stake_amount {
            self.stake_manager.min_stake_amount = min_stake_amount;
            msg!("min_stake_amount: {}", min_stake_amount);
        }

        if let Some(platform_fee_commission) = config_stake_manager_params.platform_fee_commission {
            require!(
                platform_fee_commission < 1_000_000_000,
                Errors::ParamsNotMatch
            );

            self.stake_manager.platform_fee_commission = platform_fee_commission;
            msg!("platform_fee_commission: {}", platform_fee_commission);
        }

        if let Some(rate_change_limit) = config_stake_manager_params.rate_change_limit {
            self.stake_manager.rate_change_limit = rate_change_limit;
            msg!("rate_change_limit: {}", rate_change_limit);
        }

        Ok(())
    }
}
