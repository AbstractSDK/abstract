#![warn(missing_docs)]
//! # Dex Adapter Raw Action Definition

use abstract_std::objects::pool_id::UncheckedPoolAddress;
use cosmwasm_std::Decimal;
use cw_asset::{AssetBase, AssetInfoBase};

use crate::msg::SwapNode;

/// Possible raw actions to perform on the DEX
#[cosmwasm_schema::cw_serde]
pub enum DexAction {
    /// Provide arbitrary liquidity
    ProvideLiquidity {
        /// Pool to provide liquidity to
        pool: UncheckedPoolAddress,
        // support complex pool types
        /// Assets to add
        assets: Vec<AssetBase<String>>,
        /// Max spread to accept, is a percentage represented as a decimal.
        max_spread: Option<Decimal>,
    },
    /// Withdraw liquidity from a pool
    WithdrawLiquidity {
        /// Pool to withdraw liquidity from
        pool: UncheckedPoolAddress,
        /// The asset LP token that is provided.
        lp_token: AssetBase<String>,
    },
    /// Standard swap between one asset to another
    Swap {
        /// Pool used to swap
        pool: UncheckedPoolAddress,
        /// The asset to offer
        offer_asset: AssetBase<String>,
        /// The asset to receive
        ask_asset: AssetInfoBase<String>,
        /// The percentage of spread compared to pre-swap price or belief price (if provided)
        max_spread: Option<Decimal>,
        /// The belief price when submitting the transaction.
        belief_price: Option<Decimal>,
    },
    /// Swap by route between multiple assets
    RouteSwap {
        /// Pool routes used to swap
        route: Vec<SwapNode<String>>,
        /// The asset to offer
        offer_asset: AssetBase<String>,
        /// The percentage of spread compared to pre-swap price or belief price (if provided)
        max_spread: Option<Decimal>,
        /// The belief price when submitting the transaction.
        belief_price: Option<Decimal>,
    },
}
