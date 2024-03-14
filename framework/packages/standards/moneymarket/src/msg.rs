#![warn(missing_docs)]
//! # MoneyMarket Adapter API
// re-export response types
use crate::query::{MoneymarketAnsQuery, MoneymarketQueryResponse, MoneymarketRawQuery};
use crate::{ans_action::MoneymarketAnsAction, raw_action::MoneymarketRawAction};
use abstract_core::{
    adapter,
    objects::fee::{Fee, UsageFee},
    AbstractError, AbstractResult,
};
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Addr, CosmosMsg, Decimal};

/// Max fee for the dex adapter actions
pub const MAX_FEE: Decimal = Decimal::percent(5);

/// The name of the dex to trade on.
pub type MoneymarketName = String;

/// The callback id for interacting with a dex over ibc
pub const IBC_DEX_PROVIDER_ID: &str = "IBC_DEX_ACTION";

/// Top-level Abstract Adapter execute message. This is the message that is passed to the `execute` entrypoint of the smart-contract.
pub type ExecuteMsg = adapter::ExecuteMsg<MoneymarketExecuteMsg>;
/// Top-level Abstract Adapter instantiate message. This is the message that is passed to the `instantiate` entrypoint of the smart-contract.
pub type InstantiateMsg = adapter::InstantiateMsg<MoneymarketInstantiateMsg>;
/// Top-level Abstract Adapter query message. This is the message that is passed to the `query` entrypoint of the smart-contract.
pub type QueryMsg = adapter::QueryMsg<MoneymarketQueryMsg>;

impl adapter::AdapterExecuteMsg for MoneymarketExecuteMsg {}
impl adapter::AdapterQueryMsg for MoneymarketQueryMsg {}

/// Response from GenerateMsgs
#[cosmwasm_schema::cw_serde]
pub struct GenerateMessagesResponse {
    /// Messages generated for dex action
    pub messages: Vec<CosmosMsg>,
}

/// Response for MoneyMarket Fees
#[cosmwasm_schema::cw_serde]
pub struct MoneymarketFeesResponse {
    /// Fee for using swap action
    pub moneymarket_fee: Fee,
    /// Address where all fees will go
    pub recipient: Addr,
}

/// Instantiation message for dex adapter
#[cosmwasm_schema::cw_serde]
pub struct MoneymarketInstantiateMsg {
    /// Fee charged on each swap.
    pub swap_fee: Decimal,
    /// Recipient account for fees.
    pub recipient_account: u32,
}

/// MoneyMarket Execute msg
#[cosmwasm_schema::cw_serde]
pub enum MoneymarketExecuteMsg {
    /// Update the fee
    UpdateFee {
        /// New fee to set
        moneymarket_fee: Option<Decimal>,
        /// New recipient account for fees
        recipient_account: Option<u32>,
    },
    /// Action to perform on the DEX with ans asset denomination
    AnsAction {
        /// The name of the dex to interact with
        moneymarket: MoneymarketName,
        /// The action to perform
        action: MoneymarketAnsAction,
    },
    /// Action to perform on the DEX with raw asset denominations
    RawAction {
        /// The name of the dex to interact with
        moneymarket: MoneymarketName,
        /// The action to perform
        action: MoneymarketRawAction,
    },
}

/// Query messages for the dex adapter
#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses)]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
#[cfg_attr(feature = "interface", impl_into(QueryMsg))]
pub enum MoneymarketQueryMsg {
    /// Endpoint can be used by front-end to easily interact with contracts.
    /// Returns [`GenerateMessagesResponse`]
    #[returns(GenerateMessagesResponse)]
    GenerateMessages {
        /// Execute message to generate messages for
        message: MoneymarketExecuteMsg,
        /// Sender Addr generate messages for
        addr_as_sender: String,
    },

    /// Query using raw asset denoms and addresses
    #[returns(MoneymarketQueryResponse)]
    MoneymarketRawQuery {
        /// Actual query
        query: MoneymarketRawQuery,
        /// The name of the dex to interact with
        moneymarket: MoneymarketName,
    },
    /// Query using ans assets
    #[returns(MoneymarketQueryResponse)]
    MoneymarketAnsQuery {
        /// Actual query
        query: MoneymarketAnsQuery,
        /// The name of the dex to interact with
        moneymarket: MoneymarketName,
    },

    /// Fee info for using the different dex actions
    #[returns(MoneymarketFeesResponse)]
    Fees {},
}

/// Fees for using the dex adapter
#[cosmwasm_schema::cw_serde]
pub struct MoneymarketFees {
    /// Fee for using swap action
    swap_fee: Fee,
    /// Address where all fees will go
    pub recipient: Addr,
}

impl MoneymarketFees {
    /// Create checked MoneymarketFees
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
