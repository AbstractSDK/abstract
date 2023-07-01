use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, Uint128};
use cw_storage_plus::Item;

#[cw_serde]
pub struct MigrateConfig {
    /// This is the address that can run ExecuteMsg::MigrateTokens
    pub migrator: Addr,
    /// This is how long it will be staked on WYND DEX
    pub unbonding_period: u64,

    /// This is the junoswap pool where the LP will be withdrawn from
    pub junoswap_pool: Addr,

    /// Can be deposited in any pool created by this factory
    pub factory: Addr,
    /// If set, only can be deposited in this pool (which must also be created by the factory)
    pub wynddex_pool: Option<Addr>,

    /// This is set when token migration is finished.
    /// It is used to calculate the amount of LP tokens to give to each staker.
    pub migrate_stakers_config: Option<MigrateStakersConfig>,
}

/// The necessary information to migrate stakers.
#[cw_serde]
pub struct MigrateStakersConfig {
    /// The wyndex LP token contract
    pub lp_token: Addr,
    /// The wyndex LP staking contract
    pub staking_addr: Addr,
    /// The total amount of wyndex LP tokens this contract received after token migration.
    pub total_lp_tokens: Uint128,
    /// The total amount of staked junoswap LP tokens.
    pub total_staked: Uint128,
}

/// Stores the contract configuration at the given key
pub const MIGRATION: Item<MigrateConfig> = Item::new("migration");

/// This is set once MigrateTokens is called
pub const DESTINATION: Item<Addr> = Item::new("destination");

#[cw_serde]
pub struct ExchangeConfig {
    pub raw_to_wynd_exchange_rate: Decimal,
    pub raw_token: Addr,
    pub wynd_token: Addr,
}

pub const EXCHANGE_CONFIG: Item<ExchangeConfig> = Item::new("exchange_config");
