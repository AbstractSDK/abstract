//! # Fee Granter
//! This module provides functionality to interact with the feegrant module of Cosmos.
//! It allows for granting fee expenditure rights to other accounts.

use std::time::Duration;

use cosmos_sdk_proto::{Any, cosmos::base, cosmos::feegrant, traits::Message};
use cosmwasm_std::{Addr, Binary, Coin, CosmosMsg, Timestamp};
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;
use cosmos_sdk_proto::traits::TypeUrl;
use std::cell::RefCell;

use crate::AbstractSdkResult;
use crate::features::AccountIdentification;

const BASIC_ALLOWANCE_TYPE_URL: &str = feegrant::v1beta1::BasicAllowance::TYPE_URL;
const PERIODIC_ALLOWANCE_TYPE_URL: &str = feegrant::v1beta1::PeriodicAllowance::TYPE_URL;
const ALLOWED_MSG_ALLOWANCE_TYPE_URL: &str = feegrant::v1beta1::AllowedMsgAllowance::TYPE_URL;


/// An interface to the CosmosSDK FeeGrant module which allows for granting fee expenditure rights.
pub trait GrantInterface: AccountIdentification {
    /// API for accessing the Cosmos SDK FeeGrant module.
    /// The **granter** is the address of the user granting an allowance of their funds.
    /// By default, it is the proxy address of the Account.

/// ```
/// use abstract_sdk::prelude::*;
/// # use cosmwasm_std::testing::mock_dependencies;
/// # use abstract_sdk::mock_module::MockModule;
/// # let module = MockModule::new();
/// # let deps = mock_dependencies();

/// let grant: FeeGranter = module.fee_granter(deps.as_ref(), None);
/// ```
    fn fee_granter<'a>(&'a self, deps: cosmwasm_std::Deps<'a>, granter: Option<Addr>) -> FeeGranter<Self> {
        FeeGranter { base: self, deps, granter: RefCell::new(granter) }
    }
}

impl<T> GrantInterface for T where T: AccountIdentification {}

/// This struct provides methods to grant fee allowances and interact with the feegrant module.
///
/// # Example
/// ```
/// use abstract_sdk::prelude::*;
/// # use cosmwasm_std::testing::mock_dependencies;
/// # use abstract_sdk::mock_module::MockModule;
/// # let module = MockModule::new();
///
/// let grant: FeeGranter  = module.fee_granter();
/// ```
/// */
#[derive(Clone)]
pub struct FeeGranter<'a, T: GrantInterface> {
    base: &'a T,
    deps: cosmwasm_std::Deps<'a>,
    // We use a RefCell here to allow for the granter to be set after the FeeGranter is created
    granter: RefCell<Option<Addr>>,
}

impl<'a, T: GrantInterface> FeeGranter<'a, T> {
    /// Retrieve the granter's address.
    /// By default, this is the proxy address of the Account.
    /// If the granter was already set or overridden, it returns that value.
    fn granter(&self) -> AbstractSdkResult<Addr> {
        // If the sender was already set or overridden, return it
        if let Some(granter) = &*self.granter.borrow() {
            return Ok(granter.clone());
        }

        let granter = self.base.proxy_address(self.deps)?;
        *self.granter.borrow_mut() = Some(granter.clone());
        Ok(granter)
    }

    /// Removes any existing allowances from the granter to the **grantee**.
    ///
    /// # Arguments
    ///
    /// * `grantee` - The address of the grantee.
    pub fn revoke_allowance(&self, grantee: &Addr) -> AbstractSdkResult<CosmosMsg> {
        let msg = feegrant::v1beta1::MsgRevokeAllowance {
            granter: self.granter()?.to_string(),
            grantee: grantee.to_string(),
        }
            .encode_to_vec();

        let msg = CosmosMsg::Stargate {
            type_url: feegrant::v1beta1::MsgRevokeAllowance::TYPE_URL.to_string(),
            value: Binary(msg),
        };

        Ok(msg)
    }

    /// Grants an allowance to a **grantee**.
    ///
    /// # Arguments
    ///
    /// * `grantee` - The address of the grantee.
    /// * `allowance` - The allowance to be granted.
    pub fn grant_allowance<A: FeeGranterAllowance>(
        &self,
        grantee: &Addr,
        allowance: A,
    ) -> AbstractSdkResult<CosmosMsg> {
        let msg = feegrant::v1beta1::MsgGrantAllowance {
            granter: self.granter()?.to_string(),
            grantee: grantee.to_string(),
            allowance: Some(allowance.to_any()),
        }
            .encode_to_vec();

        let msg = CosmosMsg::Stargate {
            type_url: feegrant::v1beta1::MsgGrantAllowance::TYPE_URL.to_string(),
            value: Binary(msg),
        };

        Ok(msg)
    }

    /// Grants a basic allowance.
    ///
    /// # Arguments
    ///
    /// * `grantee` - The address of the grantee.
    /// * `spend_limit` - The maximum amount the grantee can spend.
    /// * `expiration` - The expiration timestamp of the grant.
    pub fn grant_basic_allowance(
        &self,
        grantee: &Addr,
        spend_limit: Vec<Coin>,
        expiration: Option<Timestamp>,
    ) -> AbstractSdkResult<CosmosMsg> {
        let basic_allowance = BasicAllowance::new(spend_limit, expiration);
        self.grant_allowance(grantee, basic_allowance)
    }

    /// Grants a periodic allowance to a grantee.
    ///
    /// # Arguments
    ///
    /// * `grantee` - The address of the grantee.
    /// * `basic` - The basic allowance details.
    /// * `period` - The period of the allowance.
    /// * `period_spend_limit` - The maximum amount the grantee can spend in the period.
    /// * `period_can_spend` - The amount left for the grantee to spend before the period reset.
    /// * `period_reset` - The time at which the period resets.
    pub fn grant_periodic_allowance(
        &self,
        grantee: &Addr,
        basic: Option<BasicAllowance>,
        period: Option<Duration>,
        period_spend_limit: Vec<Coin>,
        period_can_spend: Vec<Coin>,
        period_reset: Option<Timestamp>,
    ) -> AbstractSdkResult<CosmosMsg> {
        let periodic_allowance = PeriodicAllowance::new(
            basic,
            period,
            period_spend_limit,
            period_can_spend,
            period_reset,
        );
        self.grant_allowance(grantee, periodic_allowance)
    }

    /// Grants an allowed message allowance to a grantee.
    ///
    /// # Arguments
    /// * `grantee` - The address of the grantee.
    /// * `allowed_messages` - The list of allowed messages for the grantee.
    /// * `allowance` - The allowance details.
    pub fn grant_allowed_msg_allowance<A: AllowedMsgAllowanceAllowance + 'static>(&self, grantee: &Addr, allowed_messages: Vec<String>, allowance: Option<A>) -> AbstractSdkResult<CosmosMsg> {
        let allowed_msg_allowance = AllowedMsgAllowance {
            allowance: allowance.map(|a| Box::new(a) as Box<dyn AllowedMsgAllowanceAllowance>),
            allowed_messages,
        };
        self.grant_allowance(grantee, allowed_msg_allowance)
    }
}

fn convert_coins(coins: Vec<Coin>) -> Vec<base::v1beta1::Coin> {
    coins
        .into_iter()
        .map(|item| base::v1beta1::Coin {
            denom: item.denom,
            amount: item.amount.to_string(),
        })
        .collect()
}

fn convert_stamp(stamp: Timestamp) -> prost_types::Timestamp {
    prost_types::Timestamp {
        seconds: stamp.seconds() as i64,
        nanos: stamp.nanos() as i32,
    }
}

/// Trait for types that can be used as allowances in the FeeGranter.
pub trait AllowedMsgAllowanceAllowance: FeeGranterAllowance {}
impl AllowedMsgAllowanceAllowance for BasicAllowance {}
impl AllowedMsgAllowanceAllowance for PeriodicAllowance {}

/// Represents a basic fee allowance grant.
#[derive(Serialize, Deserialize, Clone, Default, PartialEq, Eq, JsonSchema)]
pub struct BasicAllowance {
    /// Maximum amount of tokens that can be spent
    pub spend_limits: Vec<Coin>,
    /// When the grant expires
    pub expiration: Option<Timestamp>,
}

impl BasicAllowance {
    pub fn new(spend_limits: Vec<Coin>, expiration: Option<Timestamp>) -> Self {
        Self {
            spend_limits,
            expiration,
        }
    }

    pub fn to_proto(&self) -> feegrant::v1beta1::BasicAllowance {
        feegrant::v1beta1::BasicAllowance {
            spend_limit: self.spend_limits.iter().map(|item| base::v1beta1::Coin {
                denom: item.denom.clone(),
                amount: item.amount.to_string(),
            }).collect(),
            expiration: self.expiration.map(convert_stamp),
        }
    }
}

/// Details for a periodic fee allowance grant
/// @see [cosmos_sdk_proto::cosmos::feegrant::v1beta1::PeriodicAllowance]
#[derive(Serialize, Deserialize, Clone, Default, PartialEq, Eq, JsonSchema)]
pub struct PeriodicAllowance {
    /// basic is the instance of [BasicAllowance] which is optional for periodic fee allowance. If empty, the grant will have no expiration and no spend_limit.
    pub basic: Option<BasicAllowance>,
    /// period specifies the time duration in which period_spend_limit coins can
    /// be spent before that allowance is reset
    pub period: Option<Duration>,
    /// Maximum amount of tokens that can be spent per period
    pub period_spend_limit: Vec<Coin>,
    /// period_can_spend is the number of coins left to be spent before the period_reset time
    pub period_can_spend: Vec<Coin>,
    /// period_reset is the time at which this period resets and a new one begins,
    /// it is calculated from the start time of the first transaction after the
    /// last period ended
    pub period_reset: Option<Timestamp>,
}

impl PeriodicAllowance {
    pub fn new(
        basic: Option<BasicAllowance>,
        period: Option<Duration>,
        period_spend_limit: Vec<Coin>,
        period_can_spend: Vec<Coin>,
        period_reset: Option<Timestamp>,
    ) -> Self {
        Self {
            basic,
            period,
            period_spend_limit,
            period_can_spend,
            period_reset,
        }
    }

    pub fn to_proto(&self) -> feegrant::v1beta1::PeriodicAllowance {
        feegrant::v1beta1::PeriodicAllowance {
            basic: self.basic.clone().map(|b| b.to_proto()),
            period: self.period.map(|p| prost_types::Duration {
                seconds: p.as_secs() as i64,
                nanos: 0,
            }),
            period_spend_limit: convert_coins(self.period_spend_limit.clone()),
            period_can_spend: convert_coins(self.period_can_spend.clone()),
            period_reset: self.period_reset.map(convert_stamp),
        }
    }
}

pub struct AllowedMsgAllowance {
    /// allowance can be any of basic and periodic fee allowance.
    pub allowance: Option<Box<dyn AllowedMsgAllowanceAllowance>>,
    /// allowed_messages are the messages for which the grantee has the access.
    pub allowed_messages: Vec<String>,
}

pub trait FeeGranterAllowance {
    fn to_any(&self) -> Any;
}

impl FeeGranterAllowance for BasicAllowance {
    fn to_any(&self) -> Any {
        Any {
            type_url: feegrant::v1beta1::BasicAllowance::TYPE_URL.to_string(),
            value: self.clone().to_proto().encode_to_vec(),
        }
    }
}

impl FeeGranterAllowance for PeriodicAllowance {
    fn to_any(&self) -> Any {
        Any {
            type_url: feegrant::v1beta1::PeriodicAllowance::TYPE_URL.to_string(),
            value: feegrant::v1beta1::PeriodicAllowance {
                basic: self.basic.clone().map(|b| b.to_proto()),
                period: self.period.map(|p| prost_types::Duration {
                    seconds: p.as_secs() as i64,
                    nanos: 0,
                }),
                period_spend_limit: convert_coins(self.period_spend_limit.clone()),
                period_can_spend: convert_coins(self.period_can_spend.clone()),
                period_reset: self.period_reset.map(convert_stamp),
            }
                .encode_to_vec(),
        }
    }
}

impl FeeGranterAllowance for AllowedMsgAllowance {
    fn to_any(&self) -> Any {
        Any {
            type_url: feegrant::v1beta1::AllowedMsgAllowance::TYPE_URL.to_string(),
            value: feegrant::v1beta1::AllowedMsgAllowance {
                allowance: self.allowance.as_ref().map(|a| a.to_any()),
                allowed_messages: self.allowed_messages.clone(),
            }
                .encode_to_vec(),
        }
    }
}


// TODO: tests using test-tube