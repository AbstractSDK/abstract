//! # Decentralized Exchange API
//!
//! `abstract_os::dex` is a generic dex-interfacing contract that handles address retrievals and dex-interactions.

use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Decimal, Uint128};

use crate::objects::{AssetEntry, ContractEntry};

type DexName = String;
pub type OfferAsset = (AssetEntry, Uint128);

pub const IBC_DEX_ID: u32 = 11335;

#[cosmwasm_schema::cw_serde]
/// Possible actions to perform on the DEX
pub enum DexAction {
    ProvideLiquidity {
        // support complex pool types
        /// Assets to add
        assets: Vec<OfferAsset>,
        max_spread: Option<Decimal>,
    },
    ProvideLiquiditySymmetric {
        offer_asset: OfferAsset,
        // support complex pool types
        /// Assets that are paired with the offered asset
        paired_assets: Vec<AssetEntry>,
    },
    WithdrawLiquidity {
        lp_token: AssetEntry,
        amount: Uint128,
    },
    Swap {
        offer_asset: OfferAsset,
        ask_asset: AssetEntry,
        max_spread: Option<Decimal>,
        belief_price: Option<Decimal>,
    },
}

/// Dex Execute msg
#[cosmwasm_schema::cw_serde]
pub struct RequestMsg {
    pub dex: DexName,
    pub action: DexAction,
}

#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses)]
pub enum ApiQueryMsg {
    #[returns(SimulateSwapResponse)]
    SimulateSwap {
        offer_asset: OfferAsset,
        ask_asset: AssetEntry,
        dex: Option<DexName>,
    },
}

// LP/protocol fees could be withheld from either input or output so commission asset must be included.
#[cosmwasm_schema::cw_serde]
pub struct SimulateSwapResponse {
    pub pool: ContractEntry,
    /// Amount you would receive when performing the swap.
    pub return_amount: Uint128,
    /// Spread in ask_asset for this swap
    pub spread_amount: Uint128,
    /// Commission charged for the swap
    pub commission: (AssetEntry, Uint128),
}
