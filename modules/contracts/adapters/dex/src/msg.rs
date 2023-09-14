#![warn(missing_docs)]
//! # Dex Adapter API
use crate::contract::DexAdapter;
use abstract_core::objects::{AnsAsset, AssetEntry};
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Decimal, Uint128};
// re-export response types
pub use abstract_dex_adapter_traits::types::*;

pub type DexName = String;
pub type OfferAsset = AnsAsset;
pub type AskAsset = AnsAsset;

pub const IBC_DEX_ID: u32 = 11335;

abstract_adapter::adapter_msg_types!(DexAdapter, DexExecuteMsg, DexQueryMsg);

#[cosmwasm_schema::cw_serde]
pub struct DexInstantiateMsg {
    pub swap_fee: Decimal,
    pub recipient_account: u32,
}

/// Dex Execute msg
#[cosmwasm_schema::cw_serde]
pub enum DexExecuteMsg {
    UpdateFee {
        swap_fee: Option<Decimal>,
        recipient_account: Option<u32>,
    },
    Action {
        dex: DexName,
        action: DexAction,
    },
}

/// Possible actions to perform on the DEX
#[cosmwasm_schema::cw_serde]
pub enum DexAction {
    /// Provide arbitrary liquidity
    ProvideLiquidity {
        // support complex pool types
        /// Assets to add
        assets: Vec<OfferAsset>,
        max_spread: Option<Decimal>,
    },
    /// Provide liquidity equally between assets to a pool
    ProvideLiquiditySymmetric {
        offer_asset: OfferAsset,
        // support complex pool types
        /// Assets that are paired with the offered asset
        /// Should exclude the offer asset
        paired_assets: Vec<AssetEntry>,
    },
    /// Withdraw liquidity from a pool
    WithdrawLiquidity {
        lp_token: AssetEntry,
        amount: Uint128,
    },
    /// Standard swap between one asset to another
    Swap {
        offer_asset: OfferAsset,
        ask_asset: AssetEntry,
        max_spread: Option<Decimal>,
        belief_price: Option<Decimal>,
    },
    /// Allow alternative swap routers and methods
    CustomSwap {
        offer_assets: Vec<OfferAsset>,
        ask_assets: Vec<AskAsset>,
        max_spread: Option<Decimal>,
        /// Optionally supply a router to use
        router: Option<SwapRouter>,
    },
}

#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses)]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
#[cfg_attr(feature = "interface", impl_into(QueryMsg))]
pub enum DexQueryMsg {
    #[returns(SimulateSwapResponse)]
    SimulateSwap {
        offer_asset: OfferAsset,
        ask_asset: AssetEntry,
        dex: Option<DexName>,
    },
    /// Endpoint can be used by front-end to easily interact with contracts.
    #[returns(GenerateMessagesResponse)]
    GenerateMessages { message: DexExecuteMsg },
}
