pub use crate::errors::Errors;
use crate::helper;
use anchor_lang::prelude::*;

#[account]
#[derive(Debug)]
pub struct StakeManager {
    pub creator: Pubkey,
    pub index: u8,
    pub admin: Pubkey,
    pub pending_admin: Pubkey,
    pub pool_seed_bump: u8,

    pub lsd_token_mint: Pubkey,

    // staking program related
    pub staking_program: Pubkey,
    pub staking_token_mint: Pubkey,
    pub staking_pool: Pubkey,
    pub staking_min_stake_amount: u64,

    pub era_seconds: i64,
    pub era_offset: i64,
    pub min_stake_amount: u64,
    pub platform_fee_commission: u64, // decimals 9
    pub rate_change_limit: u64,       // decimals 9
    pub unbonding_duration: u64,      // decimals 9

    pub era_status: EraStatus,
    pub latest_era: u64,
    pub rate: u64, // decimals 9
    pub era_bond: u64,
    pub era_unbond: u64,
    pub pending_bond: u64,
    pub pending_unbond: u64,
    pub active: u64,
    pub total_platform_fee: u64,

    pub era_rates: Vec<EraRate>,

    /// Reserved space for future upgrades. Do not use.
    pub _reserved: [u8; 256],
}

#[derive(Clone, Debug, Default, AnchorSerialize, AnchorDeserialize)]
pub struct EraRate {
    pub era: u64,
    pub rate: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum EraStatus {
    EraUpdated,
    Bonded,
    Unbonded,
    ActiveUpdated,
}

impl StakeManager {
    pub fn calc_lsd_token_amount(&self, staking_token_amount: u64) -> Result<u64> {
        u64::try_from(
            (staking_token_amount as u128) * (helper::CAL_BASE as u128) / (self.rate as u128),
        )
        .map_err(|_| error!(Errors::CalculationFail))
    }

    pub fn calc_staking_token_amount(&self, lsd_token_amount: u64) -> Result<u64> {
        u64::try_from((lsd_token_amount as u128) * (self.rate as u128) / (helper::CAL_BASE as u128))
            .map_err(|_| error!(Errors::CalculationFail))
    }

    pub fn calc_platform_fee(&self, reward: u64) -> Result<u64> {
        u64::try_from(
            (reward as u128) * (self.platform_fee_commission as u128) / (self.rate as u128),
        )
        .map_err(|_| error!(Errors::CalculationFail))
    }

    pub fn calc_rate(&self, staking_token_amount: u64, lsd_token_amount: u64) -> Result<u64> {
        if staking_token_amount == 0 || lsd_token_amount == 0 {
            return Ok(helper::CAL_BASE);
        }

        u64::try_from(
            (staking_token_amount as u128) * (helper::CAL_BASE as u128)
                / (lsd_token_amount as u128),
        )
        .map_err(|_| error!(Errors::CalculationFail))
    }

    pub fn calc_rate_change(&self, old_rate: u64, new_rate: u64) -> Result<u64> {
        if old_rate == 0 {
            return Ok(0);
        }
        let diff = if old_rate > new_rate {
            old_rate - new_rate
        } else {
            new_rate - old_rate
        };

        u64::try_from((diff as u128) * (helper::CAL_BASE as u128) / (old_rate as u128))
            .map_err(|_| error!(Errors::CalculationFail))
    }

    pub fn calc_current_era(&self, timestamp: i64) -> Result<u64> {
        u64::try_from(timestamp / self.era_seconds + self.era_offset)
            .map_err(|_| error!(Errors::CalculationFail))
    }
}

#[account]
#[derive(Debug)]
pub struct UnstakeAccount {
    pub stake_manager: Pubkey,
    pub user: Pubkey,
    pub amount: u64,
    pub withdrawable_era: u64,

    /// Reserved space for future upgrades. Do not use.
    pub _reserved: [u8; 128],
}
