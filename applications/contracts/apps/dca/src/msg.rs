use abstract_core::objects::{AssetEntry, DexName, PoolReference};
use abstract_dex_adapter::msg::OfferAsset;
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Decimal, Uint128};
use croncat_app::croncat_integration_utils::CronCatInterval;

use crate::{
    contract::DCAApp,
    state::{Config, DCAEntry},
};

// This is used for type safety
// The second part is used to indicate the messages are used as the apps messages
// This is equivalent to
// pub type InstantiateMsg = <App as abstract_sdk::base::InstantiateEndpoint>::InstantiateMsg;
// pub type ExecuteMsg = <App as abstract_sdk::base::ExecuteEndpoint>::ExecuteMsg;
// pub type QueryMsg = <App as abstract_sdk::base::QueryEndpoint>::QueryMsg;
// pub type MigrateMsg = <App as abstract_sdk::base::MigrateEndpoint>::MigrateMsg;

// impl app::AppExecuteMsg for AppExecuteMsg {}
// impl app::AppQueryMsg for AppQueryMsg {}
abstract_app::app_msg_types!(DCAApp, DCAExecuteMsg, DCAQueryMsg);

#[cosmwasm_schema::cw_serde]
#[non_exhaustive]
pub enum Frequency {
    /// Blocks will schedule the next DCA purchase every `n` blocks.
    EveryNBlocks(u64),
    /// Time will schedule the next DCA purchase using crontab.
    Cron(String),
}

impl Frequency {
    pub fn to_interval(self) -> CronCatInterval {
        match self {
            Frequency::EveryNBlocks(blocks) => CronCatInterval::Block(blocks),
            Frequency::Cron(cron_tab) => CronCatInterval::Cron(cron_tab),
        }
    }
}
/// App instantiate message
#[cosmwasm_schema::cw_serde]
pub struct AppInstantiateMsg {
    /// Native denom for this chain
    pub native_denom: String,
    /// Amount in native coins for creation dca task and refill amount
    pub dca_creation_amount: Uint128,
    /// Task balance threshold to trigger refill, put it at zero if you consider to never refill your tasks
    pub refill_threshold: Uint128,
    /// Max spread
    pub max_spread: Decimal,
}

/// App execute messages
#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::ExecuteFns))]
#[cfg_attr(feature = "interface", impl_into(ExecuteMsg))]
pub enum DCAExecuteMsg {
    UpdateConfig {
        new_native_denom: Option<String>,
        new_dca_creation_amount: Option<Uint128>,
        new_refill_threshold: Option<Uint128>,
        new_max_spread: Option<Decimal>,
    },
    /// Used to create a new DCA
    CreateDCA {
        /// The name of the asset to be used for purchasing
        source_asset: OfferAsset,
        /// The name of the asset to be purchased
        target_asset: AssetEntry,
        /// The frequency of purchase
        frequency: Frequency,
        /// The DEX to be used for the swap
        dex: DexName,
    },
    // MultipleCreateDcas
    /// Used to update an existing DCA
    UpdateDCA {
        /// Unique identifier for the DCA
        dca_id: String,
        /// Optional new name of the asset to be used for purchasing
        new_source_asset: Option<OfferAsset>,
        /// Optional new name of the asset to be purchased
        new_target_asset: Option<AssetEntry>,
        /// Optional new frequency of purchase
        new_frequency: Option<Frequency>,
        /// Optional new DEX to be used for the swap
        new_dex: Option<DexName>,
    },

    /// Used to cancel an existing DCA
    CancelDCA {
        /// Unique identifier for the DCA
        dca_id: String,
    },
    /// Internal method for triggering swap.
    /// It can be called only by the Croncat Manager
    Convert { dca_id: String },
}

#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
#[cfg_attr(feature = "interface", impl_into(QueryMsg))]
#[derive(QueryResponses)]
pub enum DCAQueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(DCAResponse)]
    DCA { dca_id: String },
}

#[cosmwasm_schema::cw_serde]
pub enum AppMigrateMsg {}

#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    pub config: Config,
}

#[cosmwasm_schema::cw_serde]
pub struct DCAResponse {
    pub dca: Option<DCAEntry>,
    pub pool_references: Vec<PoolReference>,
}
