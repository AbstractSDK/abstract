use cosmwasm_std::{Decimal, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::common_module::api_msg::*;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Base(ApiExecuteMsg),
    // Add dapp-specific messages here
    /// Constructs a provide liquidity msg and forwards it to the proxy
    /// Calculates the required asset amount for the second asset in the pool.
    ProvideLiquidity {
        pool_id: String,
        main_asset_id: String,
        amount: Uint128,
    },
    /// Constructs a provide liquidity msg and forwards it to the proxy.
    DetailedProvideLiquidity {
        assets: Vec<(String, Uint128)>,
        pool_id: String,
        slippage_tolerance: Option<Decimal>,
    },
    /// Constructs a withdraw liquidity msg and forwards it to the proxy
    WithdrawLiquidity {
        lp_token_id: String,
        amount: Uint128,
    },
    /// Constructs a swap msg and forwards it to the proxy
    SwapAsset {
        offer_id: String,
        pool_id: String,
        amount: Uint128,
        max_spread: Option<Decimal>,
        belief_price: Option<Decimal>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Base(ApiQueryMsg),
    // Add dapp-specific queries here
}
