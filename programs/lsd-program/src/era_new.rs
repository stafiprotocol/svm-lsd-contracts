use crate::{EraStatus, Errors, StakeManager};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct EraNew<'info> {
    #[account(mut)]
    pub stake_manager: Box<Account<'info, StakeManager>>,
}

#[event]
pub struct EventEraNew {
    pub new_era: u64,
}

impl<'info> EraNew<'info> {
    pub fn process(&mut self) -> Result<()> {
        require!(
            self.stake_manager.era_status == EraStatus::ActiveUpdated,
            Errors::EraStatusNotMatch
        );

        let timestamp = Clock::get().unwrap().unix_timestamp;
        let new_era = self.stake_manager.latest_era + 1;
        let current_era = self.stake_manager.calc_current_era(timestamp)?;

        require_gte!(current_era, new_era, Errors::EraIsLatest);

        self.stake_manager.pending_unbond += self.stake_manager.era_unbond;
        self.stake_manager.pending_bond += self.stake_manager.era_bond;

        self.stake_manager.era_status = EraStatus::EraUpdated;
        if self.stake_manager.pending_bond >= self.stake_manager.pending_unbond
            && (self.stake_manager.pending_bond - self.stake_manager.pending_unbond)
                < self.stake_manager.staking_min_stake_amount
        {
            self.stake_manager.pending_bond =
                self.stake_manager.pending_bond - self.stake_manager.pending_unbond;
            self.stake_manager.pending_unbond = 0;
            self.stake_manager.era_status = EraStatus::Bonded;
        }

        self.stake_manager.latest_era = new_era;
        self.stake_manager.era_bond = 0;
        self.stake_manager.era_unbond = 0;

        emit!(EventEraNew { new_era });

        Ok(())
    }
}
