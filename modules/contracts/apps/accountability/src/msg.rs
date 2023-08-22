use cosmwasm_std::{Decimal, Uint128};

#[cosmwasm_schema::cw_serde]
pub enum Frequency {
    Daily,
    Weekly,
    Monthly,
    EveryNBlocks(u64),
}

impl Frequency {
    pub fn to_interval(self) -> CronCatInterval {
        match self {
            Frequency::EveryNBlocks(blocks) => CronCatInterval::Block(blocks),
            Frequency::Daily => CronCatInterval::Cron("0 0 * * *".to_string()),
            Frequency::Weekly => CronCatInterval::Cron("0 0 * * 0".to_string()),
            Frequency::Monthly => CronCatInterval::Cron("0 0 1 * *".to_string()),
        }
    }
}

/// App instantiate message
#[cosmwasm_schema::cw_serde]
pub struct AppInstantiateMsg {
    /// Native gas/stake asset for this chain
    pub native_asset: AssetEntry,
    /// Amount in native coins for accountability creation task and refill amount
    pub forfeit_creation_amount: Uint128,
    /// Task balance threshold to trigger refill, put it at zero if you consider to never refill your tasks
    pub refill_threshold: Uint128,
}
