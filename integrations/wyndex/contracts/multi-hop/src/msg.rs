use cosmwasm_schema::{cw_serde, QueryResponses};

use cosmwasm_std::{Decimal, Uint128};
use cw20::Cw20ReceiveMsg;

use wyndex::asset::{AssetInfo, AssetValidated};

pub const MAX_SWAP_OPERATIONS: usize = 50;

/// This structure holds the parameters used for creating a contract.
#[cw_serde]
pub struct InstantiateMsg {
    /// The wyndex factory contract address
    pub wyndex_factory: String,
}

/// This enum describes a swap operation.
/// It currently only has one variant, but is designed to be extensible,
/// so we can add other AMMs in the future.
#[cw_serde]
pub enum SwapOperation {
    /// Wyndex swap
    WyndexSwap {
        /// Information about the asset being swapped
        offer_asset_info: AssetInfo,
        /// Information about the asset we swap to
        ask_asset_info: AssetInfo,
    },
}

impl SwapOperation {
    pub fn get_target_asset_info(&self) -> AssetInfo {
        match self {
            SwapOperation::WyndexSwap { ask_asset_info, .. } => ask_asset_info.clone(),
        }
    }
}

/// This structure describes the execute messages available in the contract.
#[cw_serde]
pub enum ExecuteMsg {
    /// Receive receives a message of type [`Cw20ReceiveMsg`] and processes it depending on the received template
    Receive(Cw20ReceiveMsg),

    /// ExecuteSwapOperations processes multiple swaps while mentioning the minimum amount of tokens to receive for the last swap operation
    ExecuteSwapOperations {
        /// All swap operations to perform
        operations: Vec<SwapOperation>,
        /// Guarantee that the ask amount is above or equal to a minimum amount
        minimum_receive: Option<Uint128>,
        /// Recipient of the ask tokens
        receiver: Option<String>,
        max_spread: Option<Decimal>,
        /// The address that should receive the referral commission
        referral_address: Option<String>,
        /// The commission for the referral.
        /// This is capped by the configured max commission
        referral_commission: Option<Decimal>,
    },

    /// Internal use
    /// ExecuteSwapOperation executes a single swap operation
    ExecuteSwapOperation {
        /// Swap operation to perform
        operation: SwapOperation,
        /// Recipient of the ask tokens
        receiver: Option<String>,
        max_spread: Option<Decimal>,
        /// Whether this swap is single or part of a multi hop route
        single: bool,
        /// The address that should receive the referral commission
        referral_address: Option<String>,
        /// The commission for the referral.
        /// This is capped by the configured max commission
        referral_commission: Option<Decimal>,
    },
    /// Internal use
    /// AssertMinimumReceive checks that a receiver will get a minimum amount of tokens from a swap
    AssertMinimumReceive {
        asset_info: AssetInfo,
        prev_balance: Uint128,
        minimum_receive: Uint128,
        receiver: String,
    },
}

#[cw_serde]
pub enum Cw20HookMsg {
    ExecuteSwapOperations {
        /// A vector of swap operations
        operations: Vec<SwapOperation>,
        /// The minimum amount of tokens to get from a swap
        minimum_receive: Option<Uint128>,
        ///
        receiver: Option<String>,
        /// Max spread
        max_spread: Option<Decimal>,
        /// The address that should receive the referral commission
        referral_address: Option<String>,
        /// The commission for the referral. Only used if `referral_address` is set.
        /// This is capped by and defaulting to the configured max commission.
        /// The commission is only applied to the first of these swap operations,
        /// so the referrer will get a portion of the asset the swap starts with.
        referral_commission: Option<Decimal>,
    },
}

/// This structure describes the query messages available in the contract.
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Config returns configuration parameters for the contract using a custom [`ConfigResponse`] structure
    #[returns(ConfigResponse)]
    Config {},
    /// SimulateSwapOperations simulates multi-hop swap operations
    #[returns(SimulateSwapOperationsResponse)]
    SimulateSwapOperations {
        /// The amount of tokens to swap
        offer_amount: Uint128,
        /// The swap operations to perform, each swap involving a specific pool
        operations: Vec<SwapOperation>,
        /// Whether to simulate referral
        referral: bool,
        /// The commission for the referral. Only used if `referral` is set to `true`.
        /// This is capped by and defaulting to the configured max commission.
        /// The commission is only applied to the first of these swap operations,
        /// so the referrer will get a portion of the asset the swap starts with.
        referral_commission: Option<Decimal>,
    },
    #[returns(SimulateSwapOperationsResponse)]
    SimulateReverseSwapOperations {
        /// The amount of tokens to receive
        ask_amount: Uint128,
        /// The swap operations to perform, each swap involving a specific pool.
        /// This is *not* in reverse order. It starts with the offer asset and ends with the ask asset.
        operations: Vec<SwapOperation>,
        /// Whether to simulate referral
        referral: bool,
        /// The commission for the referral. Only used if `referral` is set to `true`.
        /// This is capped by and defaulting to the configured max commission.
        /// The commission is only applied to the first of these swap operations,
        /// so the referrer will get a portion of the asset the swap starts with.
        referral_commission: Option<Decimal>,
    },
}

/// This structure describes a custom struct to return a query response containing the base contract configuration.
#[cw_serde]
pub struct ConfigResponse {
    /// The Wyndex factory contract address
    pub wyndex_factory: String,
}

/// This structure describes a custom struct to return a query response containing the end amount of a swap simulation
#[cw_serde]
pub struct SimulateSwapOperationsResponse {
    /// The amount of tokens received / offered in a swap simulation
    pub amount: Uint128,

    /// The spread percentage for the whole all swap operations as a whole.
    /// This is the percentage by which the returned `amount` is worse than the ideal one.
    pub spread: Decimal,

    /// The absolute amounts of spread for each swap operation.
    /// This contains one entry per swap operation in the same order as the `operations` parameter,
    /// and each entry is denominated in the asset that is swapped to (`ask_asset_info`).
    pub spread_amounts: Vec<AssetValidated>,

    /// The absolute amounts of commission for each swap operation.
    /// This contains one entry per swap operation in the same order as the `operations` parameter,
    /// and each entry is denominated in the asset that is swapped to (`ask_asset_info`).
    pub commission_amounts: Vec<AssetValidated>,

    /// The absolute amount of referral commission. This is always denominated in `offer_asset_info`.
    pub referral_amount: AssetValidated,
}

#[cw_serde]
pub struct MigrateMsg {}
