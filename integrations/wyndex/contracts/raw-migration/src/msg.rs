use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Decimal;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Checks whether all stakers have been migrated
    #[returns(bool)]
    MigrationFinished {},
}

/// For existing contract, we need to specify which pool it can be withdrawn into
#[cw_serde]
pub struct MigrateMsg {
    /// This must be Some the first migration (from JunoSwap contracts).
    /// This must be None if upgrading from raw-migration to raw-migration
    pub init: Option<OrigMigrateMsg>,
}

/// For existing contract, we need to specify which pool it can be withdrawn into
#[cw_serde]
pub struct OrigMigrateMsg {
    /// This is the address that can run ExecuteMsg::MigrateTokens
    pub migrator: String,
    /// This is how long it will be staked on WYND DEX
    pub unbonding_period: u64,

    /// This is the junoswap pool where the LP will be withdrawn from
    pub junoswap_pool: String,

    /// Can be deposited in any pool created by this factory
    pub factory: String,
    /// If set, only can be deposited in this pool (which must also be created by the factory)
    pub wynddex_pool: Option<String>,
    // Multiplier applied on amount of RAW to get WYND tokens.
    // Example:
    // exchange_rate = 1.5
    // 10 RAW * exchange_rate = 15 WYND
    pub raw_to_wynd_exchange_rate: Decimal,
    // Address of cw20 RAW token
    pub raw_address: String,
    // Address of cw20 WYND token
    pub wynd_address: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Migrate tokens to this pool.
    /// This moves the LP tokens to this contract, which are later given to the stakers in `MigrateStakers`.
    /// Must be called by migrator.
    /// Target pool must match constraints above
    MigrateTokens { wynddex_pool: String },

    /// Give the next `limit` stakers their LP tokens.
    /// Must be called by migrator.
    MigrateStakers { limit: u32 },
}
