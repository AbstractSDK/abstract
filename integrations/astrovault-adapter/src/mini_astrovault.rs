//! Minimalistic versions of astrovault types that was created to reduce wasm size
//! Currently execute/query messages not in use

use cosmwasm_std::{Decimal, Uint128};

pub enum AstrovaultPoolType {
    Standard,
    Stable { is_xasset: bool },
    Ratio,
}

#[derive(cosmwasm_schema::serde::Deserialize)]
#[serde(rename_all = "snake_case", crate = "::cosmwasm_schema::serde")]
// Ignoring unknown fields
pub struct ConfigResponse {
    pub factory: String,
    // To determine if pool is xAsset
    pub pool_settings: Option<PoolSettings>,
}

#[derive(cosmwasm_schema::serde::Deserialize)]
#[serde(rename_all = "snake_case", crate = "::cosmwasm_schema::serde")]
pub struct PoolSettings {
    pub xasset_mode_minter: Option<String>,
}

#[derive(cosmwasm_schema::serde::Deserialize)]
#[serde(rename_all = "snake_case", crate = "::cosmwasm_schema::serde")]
pub struct LpBalanceResponse {
    pub locked: Uint128,
}

/// This is enum that includes all of the messages we use in astrovault cw20 hook
/// It's separated for minimizing the size of the wasm
#[cosmwasm_schema::cw_serde]
pub enum AstrovaultCw20HookMsg {
    WithdrawLiquidity {
        to: Option<String>,
    },
    WithdrawalXassetMode {
        to: Option<String>,
        expected_return: Option<Vec<Uint128>>,
    },
    #[serde(rename(serialize = "withdrawal_to_lockup"))]
    WithdrawalToLockupStable {
        withdrawal_lockup_assets_amount: Vec<Uint128>,
        to: Option<String>,
        is_instant_withdrawal: Option<bool>,
        expected_return: Option<Vec<Uint128>>,
    },
    #[serde(rename(serialize = "withdrawal_to_lockup"))]
    WithdrawalToLockupRatio {
        to: Option<String>,
        is_instant_withdrawal: Option<bool>,
        expected_return: Option<Vec<Uint128>>,
    },
}

/// This is enum that includes all of the messages we use in astrovault execution
/// It's separated for minimizing the size of the wasm
#[cosmwasm_schema::cw_serde]
pub enum AstrovaultExecuteMsg {
    #[serde(rename(serialize = "swap"))]
    SwapStandard {
        // cw20 hook don't have this field, need to skip this one
        #[serde(skip_serializing_if = "Option::is_none")]
        offer_asset: Option<astrovault::assets::asset::Asset>,
        belief_price: Option<Decimal>,
        max_spread: Option<Decimal>,
        expected_return: Option<Uint128>,
        to: Option<String>,
    },
    #[serde(rename(serialize = "swap"))]
    SwapStable {
        swap_to_asset_index: u32,
        to: Option<String>,
        expected_return: Option<Uint128>,
    },
    #[serde(rename(serialize = "swap"))]
    SwapRatio {
        to: Option<String>,
        expected_return: Option<Uint128>,
    },
    ProvideLiquidity {
        assets: [astrovault::assets::asset::Asset; 2],
        slippage_tolerance: Option<Decimal>,
        receiver: Option<String>,
        direct_staking: Option<cosmwasm_std::Empty>,
    },
    #[serde(rename(serialize = "deposit"))]
    DepositStable {
        assets_amount: Vec<Uint128>,
        receiver: Option<String>,
        direct_staking: Option<cosmwasm_std::Empty>,
    },
    #[serde(rename(serialize = "deposit"))]
    DepositRatio {
        assets_amount: [Uint128; 2],
        receiver: Option<String>,
        direct_staking: Option<cosmwasm_std::Empty>,
        expected_return: Option<Uint128>,
    },
}

/// This is enum that includes all of the messages we use in astrovault queries
/// It's separated for minimizing the size of the wasm
#[cosmwasm_schema::cw_serde]
pub enum AstrovaultQueryMsg {
    Config {},
    PoolInfo {},
    Pool {},
    Simulation {
        offer_asset: astrovault::assets::asset::Asset,
    },
    #[serde(rename(serialize = "swap_simulation"))]
    SwapSimulationStable {
        amount: Uint128,
        swap_from_asset_index: u32,
        swap_to_asset_index: u32,
    },
    #[serde(rename(serialize = "swap_simulation"))]
    SwapSimulationRatio {
        amount: Uint128,
        swap_from_asset_index: u8,
    },
}

/// Response for [`AstrovaultQueryMsg::Pool`]
#[cosmwasm_schema::cw_serde]
pub struct PoolResponse {
    pub assets: Vec<astrovault::assets::asset::Asset>,
    pub total_share: Uint128,
}

#[derive(cosmwasm_schema::serde::Deserialize)]
#[serde(rename_all = "snake_case", crate = "::cosmwasm_schema::serde")]
pub struct RewardSourceResponse {
    pub info: RewardSource,
}

#[derive(cosmwasm_schema::serde::Deserialize)]
#[serde(rename_all = "snake_case", crate = "::cosmwasm_schema::serde")]
pub struct RewardSource {
    pub reward_asset: astrovault::assets::asset::AssetInfo,
}

#[derive(cosmwasm_schema::serde::Deserialize)]
#[serde(rename_all = "snake_case", crate = "::cosmwasm_schema::serde")]
pub struct PoolInfo {
    pub asset_infos: Vec<astrovault::assets::asset::AssetInfo>,
}
