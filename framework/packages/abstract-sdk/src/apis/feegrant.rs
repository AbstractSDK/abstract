//! # Fee Granter
//! This module provides functionality to interact with the feegrant module of Cosmos.
//! It allows for granting fee expenditure rights to other accounts.

use std::time::Duration;

use cosmos_sdk_proto::{
    cosmos::feegrant,
    traits::{Message, Name},
};
use cosmwasm_std::{Addr, AnyMsg, Binary, Coin, CosmosMsg, Timestamp};

use super::stargate::feegrant::{BasicOrPeriodicAllowance, MsgAllowance};
use crate::{
    apis::stargate::feegrant::{AllowedMsgAllowance, BasicAllowance, PeriodicAllowance},
    features::AccountExecutor,
    AbstractSdkResult,
};

/// An interface to the CosmosSDK FeeGrant module which allows for granting fee expenditure rights.
pub trait GrantInterface: AccountExecutor {
    /// API for accessing the Cosmos SDK FeeGrant module.
    /// The **granter** is the address of the user granting an allowance of their funds.
    /// By default, it is the proxy address of the Account.

    /// ```
    /// use abstract_sdk::prelude::*;
    /// # use cosmwasm_std::testing::mock_dependencies;
    /// # use abstract_sdk::{mock_module::MockModule, FeeGranter, GrantInterface, AbstractSdkResult};
    /// # use abstract_testing::prelude::*;
    /// # let deps = mock_dependencies();
    /// # let account = admin_account(deps.api);
    /// # let module = MockModule::new(deps.api, account);
    ///
    /// let grant: FeeGranter = module.fee_granter(deps.as_ref(), None)?;
    ///
    /// # AbstractSdkResult::Ok(())
    /// ```
    fn fee_granter<'a>(
        &'a self,
        deps: cosmwasm_std::Deps<'a>,
        granter: Option<Addr>,
    ) -> AbstractSdkResult<FeeGranter> {
        let granter = granter.unwrap_or(self.account(deps)?.addr().clone());
        Ok(FeeGranter { granter })
    }
}

impl<T> GrantInterface for T where T: AccountExecutor {}

/// This struct provides methods to grant fee allowances and interact with the feegrant module.
///
/// # Example
/// ```
/// use abstract_sdk::prelude::*;
/// # use cosmwasm_std::testing::mock_dependencies;
/// # use abstract_sdk::{mock_module::MockModule, FeeGranter, GrantInterface, AbstractSdkResult};
/// # use abstract_testing::prelude::*;
/// # let deps = mock_dependencies();
/// # let account = admin_account(deps.api);
/// # let module = MockModule::new(deps.api, account);
///
/// let grant: FeeGranter  = module.fee_granter(deps.as_ref(), None)?;
/// # AbstractSdkResult::Ok(())
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
            value: Binary::new(msg),
        }
    }

    /// Grants an allowance to a **grantee**.
    ///
    /// # Arguments
    ///
    /// * `grantee` - The address of the grantee.
    /// * `allowance` - The allowance to be granted.
    pub fn grant_allowance<A: MsgAllowance>(&self, grantee: &Addr, allowance: A) -> CosmosMsg {
        let msg = feegrant::v1beta1::MsgGrantAllowance {
            granter: self.granter().to_string(),
            grantee: grantee.to_string(),
            allowance: Some(allowance.to_any()),
        }
        .encode_to_vec();

        CosmosMsg::Stargate {
            type_url: feegrant::v1beta1::MsgGrantAllowance::type_url(),
            value: Binary::new(msg),
        }
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
    ) -> CosmosMsg {
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
    pub fn grant_allowed_msg_allowance<A: BasicOrPeriodicAllowance>(
        &self,
        grantee: &Addr,
        allowed_messages: Vec<String>,
        allowance: Option<A>,
    ) -> CosmosMsg {
        let allowed_msg_allowance = AllowedMsgAllowance::new(allowance, allowed_messages);
        self.grant_allowance(grantee, allowed_msg_allowance)
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::needless_borrows_for_generic_args)]
    use cosmwasm_std::coins;

    use super::*;
    use crate::{apis::stargate::StargateMessage, mock_module::*};

    fn grant_allowance_msg(
        granter: Addr,
        grantee: Addr,
        allowance: impl StargateMessage,
    ) -> CosmosMsg {
        CosmosMsg::Stargate {
            type_url: feegrant::v1beta1::MsgGrantAllowance::type_url(),
            value: Binary::new(
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
            let (deps, _, app) = mock_module_setup();

            let granter = deps.api.addr_make("granter");
            let grantee = deps.api.addr_make("grantee");

            let fee_granter = app
                .fee_granter(deps.as_ref(), Some(granter.clone()))
                .unwrap();

            let spend_limit = coins(100, "asset");
            let expiration = Some(Timestamp::from_seconds(10));

            let basic_allowance_msg =
                fee_granter.grant_basic_allowance(&grantee, spend_limit.clone(), expiration);

            let expected_msg = grant_allowance_msg(
                granter,
                grantee,
                BasicAllowance {
                    spend_limit,
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
            let (deps, _, app) = mock_module_setup();

            let granter = deps.api.addr_make("granter");
            let grantee = deps.api.addr_make("grantee");
            let fee_granter = app
                .fee_granter(deps.as_ref(), Some(granter.clone()))
                .unwrap();
            let spend_limit = coins(100, "asset");
            let period_spend_limit = vec![];
            let period_can_spend = vec![];
            let expiration = Some(Timestamp::from_seconds(10));

            let basic = Some(BasicAllowance {
                spend_limit,
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
            let (deps, _, app) = mock_module_setup();

            let granter = deps.api.addr_make("granter");
            let grantee = deps.api.addr_make("grantee");
            let fee_granter = app
                .fee_granter(deps.as_ref(), Some(granter.clone()))
                .unwrap();

            let revoke_msg = fee_granter.revoke_allowance(&grantee);

            let expected_msg = CosmosMsg::Stargate {
                type_url: feegrant::v1beta1::MsgRevokeAllowance::type_url(),
                value: Binary::new(
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
