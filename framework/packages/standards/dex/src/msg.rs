#![warn(missing_docs)]
//! # Dex Adapter API
use abstract_std::{
    adapter,
    objects::{
        fee::{Fee, UsageFee},
        pool_id::{PoolAddressBase, UncheckedPoolAddress},
        AnsAsset, AssetEntry, DexAssetPairing,
    },
    AbstractError, AbstractResult,
};
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Addr, Api, CosmosMsg, Decimal, Uint128};
use cw_asset::{AssetBase, AssetInfoBase};

pub use crate::action::DexAction;

/// Max fee for the dex adapter actions
pub const MAX_FEE: Decimal = Decimal::percent(5);

/// The name of the dex to trade on.
pub type DexName = String;

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
pub struct SimulateSwapResponse<A = AssetEntry> {
    /// The pool on which the swap was simulated
    pub pool: DexAssetPairing<A>,
    /// Amount you would receive when performing the swap.
    pub return_amount: Uint128,
    /// Spread in ask_asset for this swap
    pub spread_amount: Uint128,
    // LP/protocol fees could be withheld from either input or output so commission asset must be included.
    /// Commission charged for the swap
    pub commission: (A, Uint128),
    /// Adapter fee charged for the swap (paid in offer asset)
    pub usage_fee: Uint128,
}

/// Response from GenerateMsgs
#[cosmwasm_schema::cw_serde]
pub struct GenerateMessagesResponse {
    /// Messages generated for dex action
    pub messages: Vec<CosmosMsg>,
}

/// Response for Dex Fees
#[cosmwasm_schema::cw_serde]
pub struct DexFeesResponse {
    /// Fee for using swap action
    pub swap_fee: Fee,
    /// Address where all fees will go
    pub recipient: Addr,
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
    /// Action to perform on the DEX with raw asset denominations
    Action {
        /// The name of the dex to interact with
        dex: DexName,
        /// The action to perform
        action: DexAction,
    },
}

#[cosmwasm_schema::cw_serde]
/// Swap node for swap route
pub struct SwapNode<T: cw_address_like::AddressLike> {
    /// Pool id of the swap
    pub pool_id: PoolAddressBase<T>,
    /// Asset in return from the swap
    pub ask_asset: AssetInfoBase<T>,
}

impl SwapNode<String> {
    /// Validate data contained in an _unchecked_ **swap node** instance; return a new _checked_
    /// **swap node** instance:
    /// * For Contract addresses, assert its address is valid
    ///
    ///
    /// ```rust,no_run
    /// use cosmwasm_std::{Addr, Api};
    /// use abstract_dex_standard::msg::SwapNode;
    /// use abstract_std::AbstractResult;
    ///
    /// fn validate_swap_node(api: &dyn Api, swap_node_unchecked: SwapNode<String>) {
    ///     match swap_node_unchecked.check(api) {
    ///         Ok(info) => println!("swap node is valid: {}", info.pool_id.to_string()),
    ///         Err(err) => println!("swap node is invalid! reason: {:?}", err),
    ///     }
    /// }
    /// ```
    pub fn check(self, api: &dyn Api) -> AbstractResult<SwapNode<Addr>> {
        Ok(SwapNode {
            pool_id: self.pool_id.check(api)?,
            ask_asset: self.ask_asset.check(api, None)?,
        })
    }
}

/// Query messages for the dex adapter
#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses, cw_orch::QueryFns)]
pub enum DexQueryMsg {
    /// Simulate a swap between two assets
    /// Returns [`SimulateSwapResponse`]
    #[returns(SimulateSwapResponse)]
    SimulateSwap {
        /// The asset to offer
        offer_asset: AnsAsset,
        /// The asset to receive
        ask_asset: AssetEntry,
        /// Name of the dex to simulate the swap on
        dex: DexName,
    },
    /// Simulate a swap between two assets
    /// Returns [`SimulateSwapResponse`]
    #[returns(SimulateSwapResponse<AssetInfoBase<String>>)]
    SimulateSwapRaw {
        /// The asset to offer
        offer_asset: AssetBase<String>,
        /// The asset to receive
        ask_asset: AssetInfoBase<String>,
        /// Identifies of the pool to simulate the swap on.
        pool: UncheckedPoolAddress,
        /// Name of the dex to simulate the swap on
        dex: DexName,
    },
    /// Endpoint can be used by front-end to easily interact with contracts.
    /// Returns [`GenerateMessagesResponse`]
    #[returns(GenerateMessagesResponse)]
    GenerateMessages {
        /// Execute message to generate messages for
        message: DexExecuteMsg,
        /// Sender Addr generate messages for
        addr_as_sender: String,
    },
    /// Fee info for using the different dex actions
    #[returns(DexFeesResponse)]
    Fees {},
}

/// Fees for using the dex adapter
#[cosmwasm_schema::cw_serde]
pub struct DexFees {
    /// Fee for using swap action
    swap_fee: Fee,
    /// Address where all fees will go
    pub recipient: Addr,
}

impl DexFees {
    /// Create checked DexFees
    pub fn new(swap_fee_share: Decimal, recipient: Addr) -> AbstractResult<Self> {
        Self::check_fee_share(swap_fee_share)?;
        Ok(Self {
            swap_fee: Fee::new(swap_fee_share)?,
            recipient,
        })
    }

    /// Update swap share
    pub fn set_swap_fee_share(&mut self, new_swap_fee_share: Decimal) -> AbstractResult<()> {
        Self::check_fee_share(new_swap_fee_share)?;
        self.swap_fee = Fee::new(new_swap_fee_share)?;
        Ok(())
    }

    /// Get swap fee
    pub fn swap_fee(&self) -> Fee {
        self.swap_fee
    }

    /// Usage fee for swap
    pub fn swap_usage_fee(&self) -> AbstractResult<UsageFee> {
        UsageFee::new(self.swap_fee.share(), self.recipient.clone())
    }

    fn check_fee_share(fee: Decimal) -> AbstractResult<()> {
        if fee > MAX_FEE {
            return Err(AbstractError::Fee(format!(
                "fee share can't be bigger than {MAX_FEE}"
            )));
        }
        Ok(())
    }
}
