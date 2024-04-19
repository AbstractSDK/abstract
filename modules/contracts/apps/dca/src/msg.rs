#![warn(missing_docs)]
//! # DCA App
//!
//! `abstract::cw-staking`

use abstract_std::objects::{AnsAsset, AssetEntry, DexName, PoolReference};
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Decimal, Uint128};
use croncat_app::croncat_integration_utils::CronCatInterval;

use crate::{
    contract::DCAApp,
    state::{DCAEntry, DCAId},
};

abstract_app::app_msg_types!(DCAApp, DCAExecuteMsg, DCAQueryMsg);

#[cosmwasm_schema::cw_serde]
#[non_exhaustive]
/// The frequency at which swaps are executed.
pub enum Frequency {
    /// Blocks will schedule the next DCA purchase every `n` blocks.
    EveryNBlocks(u64),
    /// Time will schedule the next DCA purchase using crontab.
    Cron(String),
}

impl Frequency {
    /// Convert `Frequency` to a croncat interval
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
    /// Native gas/stake asset for this chain
    /// used for covering gas expenses of croncat agents
    pub native_asset: AssetEntry,
    /// Initial amount in native asset that sent on creating/refilling DCA
    /// to croncat to cover gas usage of agents
    pub dca_creation_amount: Uint128,
    /// Threshold when task refill should happen
    /// if it's lower during [`DCAExecuteMsg::Convert`] DCA will refill croncat task
    /// TIP: you can put it to "0"
    pub refill_threshold: Uint128,
    /// Max trade spread
    pub max_spread: Decimal,
}

/// App execute messages
#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::ExecuteFns))]
#[cfg_attr(feature = "interface", impl_into(ExecuteMsg))]
pub enum DCAExecuteMsg {
    /// Used to update config of DCA App
    UpdateConfig {
        /// Native gas asset for this chain
        /// used for covering gas expenses of croncat agents
        native_asset: Option<AssetEntry>,
        /// New initial amount in native asset that sent on creating/refilling DCA
        /// to croncat to cover gas usage of agents
        new_dca_task_balance: Option<Uint128>,
        /// New threshold for refilling a task
        /// TIP: you can set it to "0" to disable refilling.
        /// The task will then stop running when it runs out of funds.
        task_refill_threshold: Option<Uint128>,
        /// Set the max trade spread
        max_spread: Option<Decimal>,
    },
    /// Used to create a new DCA
    CreateDCA {
        /// The name of the asset to be used for purchasing
        source_asset: AnsAsset,
        /// The name of the asset to be purchased
        target_asset: AssetEntry,
        /// The frequency of purchase
        frequency: Frequency,
        /// The DEX to be used for the swap
        dex: DexName,
    },
    /// Used to update an existing DCA
    UpdateDCA {
        /// Unique identifier for the DCA
        dca_id: DCAId,
        /// Optional new name of the asset to be used for purchasing
        new_source_asset: Option<AnsAsset>,
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
        dca_id: DCAId,
    },
    /// Internal method for triggering swap.
    /// It can be called only by the Croncat Manager
    Convert {
        /// Unique identifier for the DCA
        dca_id: DCAId,
    },
}

/// DCA query messages
#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
#[cfg_attr(feature = "interface", impl_into(QueryMsg))]
#[derive(QueryResponses)]
pub enum DCAQueryMsg {
    /// Get config of the DCA app
    /// Returns [`ConfigResponse`]
    #[returns(ConfigResponse)]
    Config {},
    /// Get DCA Entry
    /// Returns [`DCAResponse`]
    #[returns(DCAResponse)]
    DCA {
        /// Id of the DCA
        dca_id: DCAId,
    },
}

#[cosmwasm_schema::cw_serde]
/// Response for config query
pub struct ConfigResponse {
    /// Native gas/stake asset that used for attaching to croncat task
    pub native_asset: AssetEntry,
    /// Initial amount in native asset that sent on creating/refilling DCA
    /// to croncat to cover gas usage of agents
    pub dca_creation_amount: Uint128,
    /// Threshold when task refill should happen
    /// if it's lower during [`DCAExecuteMsg::Convert`] DCA will refill croncat task
    pub refill_threshold: Uint128,
    /// Max trade spread
    pub max_spread: Decimal,
}

#[cosmwasm_schema::cw_serde]
/// Response for d_c_a query
pub struct DCAResponse {
    /// DCA entry if there is any by this DCA Id
    pub dca: Option<DCAEntry>,
    /// Pools used for swapping assets by this DCA task
    pub pool_references: Vec<PoolReference>,
}
