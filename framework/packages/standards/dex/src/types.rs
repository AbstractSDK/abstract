#![warn(missing_docs)]
//! Types for the dex standard
use abstract_core::objects::{AssetEntry, DexAssetPairing};
use cosmwasm_std::{CosmosMsg, Uint128};

/// Response for simulating a swap.
#[cosmwasm_schema::cw_serde]
pub struct SimulateSwapResponse {
    /// The pool on which the swap was simulated
    pub pool: DexAssetPairing,
    /// Amount you would receive when performing the swap.
    pub return_amount: Uint128,
    /// Spread in ask_asset for this swap
    pub spread_amount: Uint128,
    // LP/protocol fees could be withheld from either input or output so commission asset must be included.
    /// Commission charged for the swap
    pub commission: (AssetEntry, Uint128),
    /// Adapter fee charged for the swap (paid in offer asset)
    pub usage_fee: Uint128,
}

/// Response from GenerateMsgs
#[cosmwasm_schema::cw_serde]
pub struct GenerateMessagesResponse {
    /// Messages generated for dex action
    pub messages: Vec<CosmosMsg>,
}
