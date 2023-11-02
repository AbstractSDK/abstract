#![warn(missing_docs)]
//! # Dex Adapter API
use abstract_core::{
    adapter,
    objects::{AnsAsset, AssetEntry},
};
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Decimal, Uint128};
// re-export response types
use abstract_core::objects::DexAssetPairing;
use cosmwasm_std::CosmosMsg;

/// The name of the dex to trade on.
pub type DexName = String;
/// Name of the asset you want to offer
pub type OfferAsset = AnsAsset;
/// Name of the asset you want to receive
pub type AskAsset = AnsAsset;

/// The callback id for interacting with a dex over ibc
pub const IBC_DEX_PROVIDER_ID: &str = "IBC_DEX_ACTION";

/// Top-level Abstract Adapter execute message. This is the message that is passed to the `execute` entrypoint of the smart-contract.
pub type ExecuteMsg = adapter::ExecuteMsg<DexExecuteMsg>;
/// Top-level Abstract Adapter instantiate message. This is the message that is passed to the `instantiate` entrypoint of the smart-contract.
pub type InstantiateMsg = adapter::InstantiateMsg<DexInstantiateMsg>;
/// Top-level Abstract Adapter query message. This is the message that is passed to the `query` entrypoint of the smart-contract.
pub type QueryMsg = adapter::QueryMsg<DexQueryMsg>;

impl adapter::AdapterExecuteMsg for DexExecuteMsg {}
impl adapter::AdapterQueryMsg for DexQueryMsg {}

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

/// Instantiation message for dex adapter
#[cosmwasm_schema::cw_serde]
pub struct DexInstantiateMsg {
    /// Fee charged on each swap.
    pub swap_fee: Decimal,
    /// Recipient account for fees.
    pub recipient_account: u32,
}

/// Dex Execute msg
#[cosmwasm_schema::cw_serde]
pub enum DexExecuteMsg {
    /// Update the fee
    UpdateFee {
        /// New fee to set
        swap_fee: Option<Decimal>,
        /// New recipient account for fees
        recipient_account: Option<u32>,
    },
    /// Action to perform on the DEX
    Action {
        /// The name of the dex to interact with
        dex: DexName,
        /// The action to perform
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
        /// Max spread to accept, is a percentage represented as a decimal.
        max_spread: Option<Decimal>,
    },
    /// Provide liquidity equally between assets to a pool
    ProvideLiquiditySymmetric {
        /// The asset to offer
        offer_asset: OfferAsset,
        // support complex pool types
        /// Assets that are paired with the offered asset
        /// Should exclude the offer asset
        paired_assets: Vec<AssetEntry>,
    },
    /// Withdraw liquidity from a pool
    WithdrawLiquidity {
        /// The asset LP token name that is provided.
        lp_token: AssetEntry,
        /// The amount of LP tokens to redeem.
        amount: Uint128,
    },
    /// Standard swap between one asset to another
    Swap {
        /// The asset to offer
        offer_asset: OfferAsset,
        /// The asset to receive
        ask_asset: AssetEntry,
        /// The percentage of spread compared to pre-swap price or belief price (if provided)
        max_spread: Option<Decimal>,
        /// The belief price when submitting the transaction.
        belief_price: Option<Decimal>,
    },
}

/// Query messages for the dex adapter
#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses)]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
#[cfg_attr(feature = "interface", impl_into(QueryMsg))]
pub enum DexQueryMsg {
    /// Simulate a swap between two assets
    #[returns(SimulateSwapResponse)]
    SimulateSwap {
        /// The asset to offer
        offer_asset: OfferAsset,
        /// The asset to receive
        ask_asset: AssetEntry,
        /// Name of the dex to simulate the swap on
        dex: Option<DexName>,
    },
    /// Endpoint can be used by front-end to easily interact with contracts.
    #[returns(GenerateMessagesResponse)]
    GenerateMessages {
        /// Execute message to generate messages for
        message: DexExecuteMsg,
    },
}
