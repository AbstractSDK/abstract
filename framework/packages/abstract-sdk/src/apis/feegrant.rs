//! # Fee Granter
//! This module provides functionality to interact with the feegrant module of Cosmos.
//! It allows for granting fee expenditure rights to other accounts.

use std::time::Duration;

use cosmos_sdk_proto::traits::Name;
use cosmos_sdk_proto::{cosmos::base, cosmos::feegrant, traits::Message, Any};
use cosmwasm_std::{Addr, Binary, Coin, CosmosMsg, Timestamp};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::features::AccountIdentification;
use crate::AbstractSdkResult;

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

    /// let grant: FeeGranter = module.fee_granter(deps.as_ref(), None)?;
    /// ```
    fn fee_granter<'a>(
        &'a self,
        deps: cosmwasm_std::Deps<'a>,
        granter: Option<Addr>,
    ) -> AbstractSdkResult<FeeGranter> {
        let granter = granter.unwrap_or(self.proxy_address(deps)?);
        Ok(FeeGranter { granter })
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
/// let grant: FeeGranter  = module.fee_granter(deps.as_ref(), None)?;
/// ```
/// */
#[derive(Clone)]
pub struct FeeGranter {
    granter: Addr,
}

impl FeeGranter {
    /// Retrieve the granter's address.
    /// By default, this is the proxy address of the Account.
    fn granter(&self) -> Addr {
        self.granter.clone()
    }

    /// Removes any existing allowances from the granter to the **grantee**.
    ///
    /// # Arguments
    ///
    /// * `grantee` - The address of the grantee.
    pub fn revoke_allowance(&self, grantee: &Addr) -> CosmosMsg {
        let msg = feegrant::v1beta1::MsgRevokeAllowance {
            granter: self.granter().to_string(),
            grantee: grantee.to_string(),
        }
        .encode_to_vec();

        CosmosMsg::Stargate {
            type_url: feegrant::v1beta1::MsgRevokeAllowance::type_url(),
            value: Binary(msg),
        }
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
    ) -> CosmosMsg {
        let msg = feegrant::v1beta1::MsgGrantAllowance {
            granter: self.granter().to_string(),
            grantee: grantee.to_string(),
            allowance: Some(allowance.to_any()),
        }
        .encode_to_vec();

        let msg = CosmosMsg::Stargate {
            type_url: feegrant::v1beta1::MsgGrantAllowance::type_url(),
            value: Binary(msg),
        };

        msg
    }

    /// Grants a basic allowance.
    ///
    /// # Arguments
    ///
    /// * `grantee` - The address of the grantee.
    /// * `spend_limits` - The maximum amount the grantee can spend.
    /// * `expiration` - The expiration timestamp of the grant.
    pub fn grant_basic_allowance(
        &self,
        grantee: &Addr,
        spend_limits: Vec<Coin>,
        expiration: Option<Timestamp>,
    ) -> CosmosMsg {
        let basic_allowance = BasicAllowance::new(spend_limits, expiration);
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
    ) -> CosmosMsg {
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
    pub fn grant_allowed_msg_allowance<A: AllowedMsgAllowanceAllowance + 'static>(
        &self,
        grantee: &Addr,
        allowed_messages: Vec<String>,
        allowance: Option<A>,
    ) -> CosmosMsg {
        let allowed_msg_allowance = AllowedMsgAllowance {
            allowance,
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
    /// Create new basic allowance
    pub fn new(spend_limits: Vec<Coin>, expiration: Option<Timestamp>) -> Self {
        Self {
            spend_limits,
            expiration,
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
    /// Create new periodic allowance
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
}

/// Allowance and list of allowed messages
pub struct AllowedMsgAllowance<A> {
    /// allowance can be any of basic and periodic fee allowance.
    pub allowance: Option<A>,
    /// allowed_messages are the messages for which the grantee has the access.
    pub allowed_messages: Vec<String>,
}

/// This trait allows generate `Any` and proto message from FeeGranter message
pub trait FeeGranterAllowance {
    /// Returned proto type
    type ProtoType: Message + Name + Sized;

    /// Get `Any`
    fn to_any(&self) -> Any {
        Any {
            type_url: Self::ProtoType::type_url(),
            value: self.to_proto().encode_to_vec(),
        }
    }

    /// Get `Self::ProtoType`
    fn to_proto(&self) -> Self::ProtoType;
}

impl FeeGranterAllowance for BasicAllowance {
    type ProtoType = feegrant::v1beta1::BasicAllowance;

    fn to_proto(&self) -> feegrant::v1beta1::BasicAllowance {
        feegrant::v1beta1::BasicAllowance {
            spend_limit: self
                .spend_limits
                .iter()
                .map(|item| base::v1beta1::Coin {
                    denom: item.denom.clone(),
                    amount: item.amount.to_string(),
                })
                .collect(),
            expiration: self.expiration.map(convert_stamp),
        }
    }
}

impl FeeGranterAllowance for PeriodicAllowance {
    type ProtoType = feegrant::v1beta1::PeriodicAllowance;

    fn to_proto(&self) -> feegrant::v1beta1::PeriodicAllowance {
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

impl<A: FeeGranterAllowance> FeeGranterAllowance for AllowedMsgAllowance<A> {
    type ProtoType = feegrant::v1beta1::AllowedMsgAllowance;

    fn to_proto(&self) -> feegrant::v1beta1::AllowedMsgAllowance {
        feegrant::v1beta1::AllowedMsgAllowance {
            allowance: self.allowance.as_ref().map(|a| a.to_any()),
            allowed_messages: self.allowed_messages.clone(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::mock_module::*;

    use cosmwasm_std::coins;
    use cosmwasm_std::testing::mock_dependencies;

    fn grant_allowance_msg(
        granter: Addr,
        grantee: Addr,
        allowance: impl FeeGranterAllowance,
    ) -> CosmosMsg {
        CosmosMsg::Stargate {
            type_url: feegrant::v1beta1::MsgGrantAllowance::type_url(),
            value: Binary(
                feegrant::v1beta1::MsgGrantAllowance {
                    granter: granter.to_string(),
                    grantee: grantee.to_string(),
                    allowance: Some(allowance.to_any()),
                }
                .encode_to_vec(),
            ),
        }
    }

    mod basic_allowance {

        use super::*;

        #[test]
        fn basic_allowance() {
            let app = MockModule::new();
            let deps = mock_dependencies();

            let granter = Addr::unchecked("granter");
            let grantee = Addr::unchecked("grantee");

            let fee_granter = app
                .fee_granter(deps.as_ref(), Some(granter.clone()))
                .unwrap();

            let spend_limits = coins(100, "asset");
            let expiration = Some(Timestamp::from_seconds(10));

            let basic_allowance_msg =
                fee_granter.grant_basic_allowance(&grantee, spend_limits.clone(), expiration);

            let expected_msg = grant_allowance_msg(
                granter,
                grantee,
                BasicAllowance {
                    spend_limits,
                    expiration,
                },
            );
            assert_eq!(basic_allowance_msg, expected_msg);
        }
    }

    mod periodic_allowance {
        use super::*;

        #[test]
        fn periodic_allowance() {
            let app = MockModule::new();
            let deps = mock_dependencies();

            let granter = Addr::unchecked("granter");
            let grantee = Addr::unchecked("grantee");
            let fee_granter = app
                .fee_granter(deps.as_ref(), Some(granter.clone()))
                .unwrap();
            let spend_limits = coins(100, "asset");
            let period_spend_limit = vec![];
            let period_can_spend = vec![];
            let expiration = Some(Timestamp::from_seconds(10));

            let basic = Some(BasicAllowance {
                spend_limits,
                expiration,
            });

            let periodic_msg = fee_granter.grant_periodic_allowance(
                &grantee,
                basic.clone(),
                None,
                period_spend_limit.clone(),
                period_can_spend.clone(),
                None,
            );

            let periodic = PeriodicAllowance {
                basic,
                period: None,
                period_spend_limit,
                period_can_spend,
                period_reset: None,
            };
            let expected_msg = grant_allowance_msg(granter, grantee, periodic);
            assert_eq!(periodic_msg, expected_msg);
        }
    }

    mod revoke_all {
        use super::*;

        #[test]
        fn revoke_all() {
            let app = MockModule::new();
            let deps = mock_dependencies();

            let granter = Addr::unchecked("granter");
            let grantee = Addr::unchecked("grantee");
            let fee_granter = app
                .fee_granter(deps.as_ref(), Some(granter.clone()))
                .unwrap();

            let revoke_msg = fee_granter.revoke_allowance(&grantee);

            let expected_msg = CosmosMsg::Stargate {
                type_url: feegrant::v1beta1::MsgRevokeAllowance::type_url(),
                value: Binary(
                    feegrant::v1beta1::MsgRevokeAllowance {
                        granter: granter.to_string(),
                        grantee: grantee.to_string(),
                    }
                    .encode_to_vec(),
                ),
            };
            assert_eq!(revoke_msg, expected_msg);
        }
    }
}
