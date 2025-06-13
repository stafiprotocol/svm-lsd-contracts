use anchor_lang::{prelude::*, Bumps};

pub mod admin;
pub mod era_active;
pub mod era_bond;
pub mod era_new;
pub mod era_unbond;
pub mod era_withdraw;
pub mod errors;
pub mod helper;
pub mod initialize_stake_manager;
pub mod metadata;
pub mod staker_stake;
pub mod staker_unstake;
pub mod staker_withdraw;
pub mod states;

pub use crate::admin::*;
pub use crate::era_active::*;
pub use crate::era_bond::*;
pub use crate::era_new::*;
pub use crate::era_unbond::*;
pub use crate::era_withdraw::*;
pub use crate::errors::Errors;
pub use crate::helper::*;
pub use crate::initialize_stake_manager::*;
pub use crate::metadata::*;
pub use crate::staker_stake::*;
pub use crate::staker_unstake::*;
pub use crate::staker_withdraw::*;
pub use crate::states::*;

declare_id!("CEftts4KkpMPiJg9hccAgqZHvUc3t1hx9VssMUGkX3ec");

fn check_context<T: Bumps>(ctx: &Context<T>) -> Result<()> {
    if !check_id(ctx.program_id) {
        return err!(Errors::ProgramIdNotMatch);
    }

    if !ctx.remaining_accounts.is_empty() {
        return err!(Errors::RemainingAccountsNotMatch);
    }

    Ok(())
}

#[program]
pub mod lsd_program {

    use super::*;

    // initialize account

    pub fn initialize_stake_manager(
        ctx: Context<InitializeStakeManager>,
        params: InitializeStakeManagerParams,
    ) -> Result<()> {
        check_context(&ctx)?;

        ctx.accounts.process(params, ctx.bumps.stake_pool)?;

        Ok(())
    }

    // admin of stake manager

    pub fn transfer_stake_manager_admin(
        ctx: Context<TransferStakeManagerAdmin>,
        new_admin: Pubkey,
    ) -> Result<()> {
        check_context(&ctx)?;

        ctx.accounts.process(new_admin)?;

        Ok(())
    }

    pub fn accept_stake_manager_admin(ctx: Context<AcceptStakeManagerAdmin>) -> Result<()> {
        check_context(&ctx)?;

        ctx.accounts.process()?;

        Ok(())
    }

    pub fn config_stake_manager(
        ctx: Context<ConfigStakeManager>,
        params: ConfigStakeManagerParams,
    ) -> Result<()> {
        check_context(&ctx)?;

        ctx.accounts.process(params)?;

        Ok(())
    }

    // metadata
    pub fn create_metadata(
        ctx: Context<CreateMetadataAccountV3>,
        params: CreateMetadataParams,
    ) -> Result<()> {
        check_context(&ctx)?;
        ctx.accounts.process(params)
    }

    // staker

    pub fn stake(ctx: Context<Stake>, stake_amount: u64) -> Result<()> {
        check_context(&ctx)?;

        ctx.accounts.process(stake_amount)?;

        Ok(())
    }

    pub fn unstake(ctx: Context<Unstake>, unstake_amount: u64) -> Result<()> {
        check_context(&ctx)?;

        ctx.accounts.process(unstake_amount)?;

        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        check_context(&ctx)?;

        ctx.accounts.process()?;

        Ok(())
    }

    // era

    pub fn era_new(ctx: Context<EraNew>) -> Result<()> {
        check_context(&ctx)?;

        ctx.accounts.process()?;

        Ok(())
    }

    pub fn era_bond(ctx: Context<EraBond>) -> Result<()> {
        check_context(&ctx)?;

        ctx.accounts.process()?;

        Ok(())
    }

    pub fn era_unbond(ctx: Context<EraUnbond>) -> Result<()> {
        check_context(&ctx)?;

        ctx.accounts.process()?;

        Ok(())
    }

    pub fn era_withdraw(ctx: Context<EraWithdraw>) -> Result<()> {
        check_context(&ctx)?;

        ctx.accounts.process()?;

        Ok(())
    }

    pub fn era_active(ctx: Context<EraActive>) -> Result<()> {
        check_context(&ctx)?;

        ctx.accounts.process()?;

        Ok(())
    }
}
