#![warn(missing_docs)]
use abstract_core::{
    adapter,
    objects::{AnsAsset, AssetEntry, DexAssetPairing},
};
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{CosmosMsg, Decimal, Uint128};

#[cosmwasm_schema::cw_serde]
pub enum SwapRouter {
    /// Matrix router
    Matrix,
    /// Use a custom router (using String type for cross-chain compatibility)
    Custom(String),
}

// LP/protocol fees could be withheld from either input or output so commission asset must be included.
#[cosmwasm_schema::cw_serde]
pub struct SimulateSwapResponse {
    pub pool: DexAssetPairing,
    /// Amount you would receive when performing the swap.
    pub return_amount: Uint128,
    /// Spread in ask_asset for this swap
    pub spread_amount: Uint128,
    /// Commission charged for the swap
    pub commission: (AssetEntry, Uint128),
    /// Adapter fee charged for the swap (paid in offer asset)
    pub usage_fee: Uint128,
}

/// Response from GenerateMsgs
#[cosmwasm_schema::cw_serde]
pub struct GenerateMessagesResponse {
    /// messages generated for dex action
    pub messages: Vec<CosmosMsg>,
}
