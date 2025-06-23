use anchor_lang::prelude::*;

#[error_code]
pub enum Errors {
    #[msg("Program id not match")]
    ProgramIdNotMatch,

    #[msg("Remaining accounts not match")]
    RemainingAccountsNotMatch,

    #[msg("Admin not match")]
    AdminNotMatch,

    #[msg("params not match")]
    ParamsNotMatch,

    #[msg("Platform fee recipient not match")]
    PlatformFeeRecipientNotMatch,

    #[msg("Stake amount too low")]
    StakeAmountTooLow,

    #[msg("Unstake user not match")]
    UnstakeUserNotMatch,

    #[msg("Balance not enough")]
    BalanceNotEnough,

    #[msg("Calulation fail")]
    CalculationFail,

    #[msg("Era is latest")]
    EraIsLatest,

    #[msg("Era status not match")]
    EraStatusNotMatch,

    #[msg("Invalid unstake account")]
    InvalidUnstakeAccount,

    #[msg("Unstake account not withdrawable")]
    UnstakeAccountNotWithdrawable,

    #[msg("Unstake account amount zero")]
    UnstakeAccountAmountZero,

    #[msg("Pool balance not enough")]
    PoolBalanceNotEnough,

    #[msg("Unstake amount is zero")]
    UnstakeAmountIsZero,

    #[msg("Rate change over limit")]
    RateChangeOverLimit,

    #[msg("Lsd token mint account not match")]
    LsdTokenMintAccountNotMatch,

    #[msg("Staking token mint account not match")]
    StakingTokenMintAccountNotMatch,

    #[msg("staking_program not match")]
    SpNotMatch,

    #[msg("staking_program stake pool not match")]
    SpStakePoolNotMatch,

    #[msg("staking_program stake account not match")]
    SpStakeAccountNotMatch,

    #[msg("Mint authority not match")]
    MintAuthorityNotMatch,

    #[msg("Freeze authority not match")]
    FreezeAuthorityNotMatch,

    #[msg("Mint supply not empty")]
    MintSupplyNotEmpty,

    #[msg("Pending admin not match")]
    PendingAdminNotMatch,
}
