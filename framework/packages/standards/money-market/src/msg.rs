#![warn(missing_docs)]
//! # MoneyMarket Adapter API
// re-export response types
use crate::{ans_action::MoneyMarketAnsAction, raw_action::MoneyMarketRawAction};
use abstract_core::objects::AssetEntry;
use abstract_core::{adapter, objects::fee::UsageFee};
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{CosmosMsg, Decimal, StdError, StdResult, Uint128};
use cw_asset::AssetInfoBase;

/// Max fee for the dex adapter actions
pub const MAX_FEE: Decimal = Decimal::percent(5);

/// The name of the dex to trade on.
pub type MoneyMarketName = String;

/// The callback id for interacting with a dex over ibc
pub const IBC_DEX_PROVIDER_ID: &str = "IBC_DEX_ACTION";

/// Top-level Abstract Adapter execute message. This is the message that is passed to the `execute` entrypoint of the smart-contract.
pub type ExecuteMsg = adapter::ExecuteMsg<MoneyMarketExecuteMsg>;
/// Top-level Abstract Adapter instantiate message. This is the message that is passed to the `instantiate` entrypoint of the smart-contract.
pub type InstantiateMsg = adapter::InstantiateMsg<MoneyMarketInstantiateMsg>;
/// Top-level Abstract Adapter query message. This is the message that is passed to the `query` entrypoint of the smart-contract.
pub type QueryMsg = adapter::QueryMsg<MoneyMarketQueryMsg>;

impl adapter::AdapterExecuteMsg for MoneyMarketExecuteMsg {}
impl adapter::AdapterQueryMsg for MoneyMarketQueryMsg {}

/// Response from GenerateMsgs
#[cosmwasm_schema::cw_serde]
pub struct GenerateMessagesResponse {
    /// Messages generated for dex action
    pub messages: Vec<CosmosMsg>,
}

/// Response for MoneyMarket Fees
pub type MoneyMarketFeesResponse = UsageFee;

/// Instantiation message for dex adapter
#[cosmwasm_schema::cw_serde]
pub struct MoneyMarketInstantiateMsg {
    /// Fee charged on each swap.
    pub fee: Decimal,
    /// Recipient account for fees.
    pub recipient_account: u32,
}

/// MoneyMarket Execute msg
#[cosmwasm_schema::cw_serde]
pub enum MoneyMarketExecuteMsg {
    /// Update the fee
    UpdateFee {
        /// New fee to set
        money_market_fee: Option<Decimal>,
        /// New recipient account for fees
        recipient_account: Option<u32>,
    },
    /// Action to perform on the DEX with ans asset denomination
    AnsAction {
        /// The name of the dex to interact with
        money_market: MoneyMarketName,
        /// The action to perform
        action: MoneyMarketAnsAction,
    },
    /// Action to perform on the DEX with raw asset denominations
    RawAction {
        /// The name of the dex to interact with
        money_market: MoneyMarketName,
        /// The action to perform
        action: MoneyMarketRawAction,
    },
}

/// Query messages for the dex adapter
#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses)]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
#[cfg_attr(feature = "interface", impl_into(QueryMsg))]
pub enum MoneyMarketQueryMsg {
    /// Endpoint can be used by front-end to easily interact with contracts.
    /// Returns [`GenerateMessagesResponse`]
    #[returns(GenerateMessagesResponse)]
    GenerateMessages {
        /// Execute message to generate messages for
        message: MoneyMarketExecuteMsg,
        /// Sender Addr generate messages for
        addr_as_sender: String,
    },

    /// Query using raw asset denoms and addresses
    /// Deposited funds for lending
    #[returns(Uint128)]
    RawUserDeposit {
        /// User that has deposited some funds
        user: String,
        /// Lended asset to query
        asset: AssetInfoBase<String>,
        contract_addr: String,
        money_market: MoneyMarketName,
    },
    #[returns(Uint128)]
    /// Deposited Collateral funds
    RawUserCollateral {
        /// User that has deposited some collateral
        user: String,
        /// Collateral asset to query
        collateral_asset: AssetInfoBase<String>,
        /// Borrowed asset to query
        borrowed_asset: AssetInfoBase<String>,
        contract_addr: String,
        money_market: MoneyMarketName,
    },
    #[returns(Uint128)]
    /// Borrowed funds
    RawUserBorrow {
        /// User that has borrowed some funds
        user: String,
        /// Collateral asset to query
        collateral_asset: AssetInfoBase<String>,
        /// Borrowed asset to query
        borrowed_asset: AssetInfoBase<String>,
        contract_addr: String,
        money_market: MoneyMarketName,
    },
    #[returns(Decimal)]
    /// Current Loan-to-Value ratio
    /// Represents the borrow usage for a specific user
    /// Allows to know how much asset are currently borrowed
    RawCurrentLTV {
        /// User that has borrowed some funds
        user: String,
        /// Collateral asset to query
        collateral_asset: AssetInfoBase<String>,
        /// Borrowed asset to query
        borrowed_asset: AssetInfoBase<String>,
        contract_addr: String,
        money_market: MoneyMarketName,
    },
    #[returns(Decimal)]
    /// Maximum Loan to Value ratio for a user
    /// Allows to know how much assets can to be borrowed
    RawMaxLTV {
        /// User that has borrowed some funds
        user: String,
        /// Collateral asset to query
        collateral_asset: AssetInfoBase<String>,
        /// Borrowed asset to query
        borrowed_asset: AssetInfoBase<String>,
        contract_addr: String,
        money_market: MoneyMarketName,
    },
    #[returns(Decimal)]
    /// Price of an asset compared to another asset
    /// The returned decimal corresponds to
    /// How much quote assets can be bought with 1 base asset
    RawPrice {
        quote: AssetInfoBase<String>,
        base: AssetInfoBase<String>,
        money_market: MoneyMarketName,
    },

    #[returns(Uint128)]
    /// Query using ans assets
    /// Deposited funds for lending
    AnsUserDeposit {
        /// User that has deposited some funds
        user: String,
        /// Lended asset to query
        asset: AssetEntry,
        money_market: MoneyMarketName,
    },
    #[returns(Uint128)]
    /// Deposited Collateral funds
    AnsUserCollateral {
        /// User that has deposited some collateral
        user: String,
        /// Collateral asset to query
        collateral_asset: AssetEntry,
        /// Borrowed asset to query
        borrowed_asset: AssetEntry,
        money_market: MoneyMarketName,
    },
    #[returns(Uint128)]
    /// Borrowed funds
    AnsUserBorrow {
        /// User that has borrowed some funds
        user: String,
        /// Collateral asset to query
        collateral_asset: AssetEntry,
        /// Borrowed asset to query
        borrowed_asset: AssetEntry,
        money_market: MoneyMarketName,
    },
    #[returns(Decimal)]
    /// Current Loan-to-Value ratio
    /// Represents the borrow usage for a specific user
    /// Allows to know how much asset are currently borrowed
    AnsCurrentLTV {
        /// User that has borrowed some funds
        user: String,
        /// Collateral asset to query
        collateral_asset: AssetEntry,
        /// Borrowed asset to query
        borrowed_asset: AssetEntry,
        money_market: MoneyMarketName,
    },
    #[returns(Decimal)]
    /// Maximum Loan to Value ratio for a user
    /// Allows to know how much assets can to be borrowed
    AnsMaxLTV {
        /// User that has borrowed some funds
        user: String,
        /// Collateral asset to query
        collateral_asset: AssetEntry,
        /// Borrowed asset to query
        borrowed_asset: AssetEntry,
        money_market: MoneyMarketName,
    },
    #[returns(Decimal)]
    /// Price of an asset compared to another asset
    /// The returned decimal corresponds to
    /// How much quote assets can be bought with 1 base asset
    AnsPrice {
        quote: AssetEntry,
        base: AssetEntry,
        money_market: MoneyMarketName,
    },

    /// Fee info for using the different dex actions
    #[returns(MoneyMarketFeesResponse)]
    Fees {},
}

impl MoneyMarketQueryMsg {
    pub fn money_market(&self) -> StdResult<&str> {
        match self {
            MoneyMarketQueryMsg::GenerateMessages {
                message,
                addr_as_sender,
            } => Err(StdError::generic_err("Wrong query type")),
            MoneyMarketQueryMsg::RawUserDeposit {
                user,
                asset,
                contract_addr,
                money_market,
            } => Ok(money_market),
            MoneyMarketQueryMsg::RawUserCollateral {
                user,
                collateral_asset,
                borrowed_asset,
                contract_addr,
                money_market,
            } => Ok(money_market),
            MoneyMarketQueryMsg::RawUserBorrow {
                user,
                collateral_asset,
                borrowed_asset,
                contract_addr,
                money_market,
            } => Ok(money_market),
            MoneyMarketQueryMsg::RawCurrentLTV {
                user,
                collateral_asset,
                borrowed_asset,
                contract_addr,
                money_market,
            } => Ok(money_market),
            MoneyMarketQueryMsg::RawMaxLTV {
                user,
                collateral_asset,
                borrowed_asset,
                contract_addr,
                money_market,
            } => Ok(money_market),
            MoneyMarketQueryMsg::RawPrice {
                quote,
                base,
                money_market,
            } => Ok(money_market),
            MoneyMarketQueryMsg::AnsUserDeposit {
                user,
                asset,
                money_market,
            } => Ok(money_market),
            MoneyMarketQueryMsg::AnsUserCollateral {
                user,
                collateral_asset,
                borrowed_asset,
                money_market,
            } => Ok(money_market),
            MoneyMarketQueryMsg::AnsUserBorrow {
                user,
                collateral_asset,
                borrowed_asset,
                money_market,
            } => Ok(money_market),
            MoneyMarketQueryMsg::AnsCurrentLTV {
                user,
                collateral_asset,
                borrowed_asset,
                money_market,
            } => Ok(money_market),
            MoneyMarketQueryMsg::AnsMaxLTV {
                user,
                collateral_asset,
                borrowed_asset,
                money_market,
            } => Ok(money_market),
            MoneyMarketQueryMsg::AnsPrice {
                quote,
                base,
                money_market,
            } => Ok(money_market),
            MoneyMarketQueryMsg::Fees {} => Err(StdError::generic_err("Wrong query type")),
        }
    }
}
