use cosmwasm_std::{Decimal, Uint128};
use cw_asset::AssetInfo;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use terra_rust_script_derive::CosmWasmContract;

use crate::common_module::api_msg::*;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, CosmWasmContract)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, CosmWasmContract)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Handles all the base query msgs
    Base(ApiQueryMsg),
}

pub fn cw_to_terraswap(cw: &cw_asset::AssetInfo) -> terraswap::asset::AssetInfo {
    match cw {
        AssetInfo::Cw20(contract_addr) => terraswap::asset::AssetInfo::Token {
            contract_addr: contract_addr.to_string(),
        },
        AssetInfo::Native(denom) => terraswap::asset::AssetInfo::NativeToken {
            denom: denom.to_owned(),
        },
    }
}
