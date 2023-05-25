//! # Grant
//! Interacts with the feegrant module of cosmos
//!

use std::time::Duration;

use crate::Execution;

use cosmos_sdk_proto::{cosmos::base, cosmos::feegrant, traits::Message, Any};
use cosmwasm_std::{to_binary, Addr, Coin, CosmosMsg, Deps, Timestamp};

use crate::AbstractSdkResult;

/// An interface to the CosmosSDK FeeGrant module which allows for granting fee expenditure rights.
pub trait GrantInterface: Execution {
    /**
        API for accessing the Cosmos SDK FeeGrant module.

        # Example
        ```
        use abstract_sdk::prelude::*;
        # use cosmwasm_std::testing::mock_dependencies;
        # use abstract_sdk::mock_module::MockModule;
        # let module = MockModule::new();
        # let deps = mock_dependencies();

        let grant: Grant<MockModule> = module.grant(deps.as_ref());
        ```
    */
    fn grant<'a>(&'a self, deps: Deps<'a>) -> Grant<Self> {
        Grant { base: self, deps }
    }
}

impl<T> GrantInterface for T where T: Execution {}

/**
    API for accessing the Cosmos SDK FeeGrant module.

    # Example
    ```
    use abstract_sdk::prelude::*;
    # use cosmwasm_std::testing::mock_dependencies;
    # use abstract_sdk::mock_module::MockModule;
    # let module = MockModule::new();
    # let deps = mock_dependencies();

    let grant: Grant<MockModule>  = module.grant(deps.as_ref());
    ```
*/
#[derive(Clone)]
pub struct Grant<'a, T: GrantInterface> {
    base: &'a T,
    deps: Deps<'a>,
}

impl<'a, T: GrantInterface> Grant<'a, T> {
    /// Implements Allowance with a one-time grant of coins
    /// that optionally expires. The grantee can use up to SpendLimit to cover fees.
    pub fn basic(
        &self,
        granter: &Addr,
        grantee: &Addr,
        basic: BasicAllowance,
    ) -> AbstractSdkResult<CosmosMsg> {
        let msg = feegrant::v1beta1::MsgGrantAllowance {
            granter: granter.into(),
            grantee: grantee.into(),
            allowance: Some(build_any_basic(basic)),
        }
        .encode_to_vec();

        let msg = CosmosMsg::Stargate {
            type_url: "/cosmos.feegrant.v1beta1.MsgGrantAllowance".to_string(),
            value: to_binary(&msg)?,
        };

        self.base.executor(self.deps).execute(vec![msg])
    }

    /// Extends Allowance to allow for both a maximum cap,
    /// as well as a limit per time period.
    pub fn periodic(
        &self,
        granter: &Addr,
        grantee: &Addr,
        basic: Option<BasicAllowance>,
        periodic: PeriodicAllowance,
    ) -> AbstractSdkResult<CosmosMsg> {
        let msg = feegrant::v1beta1::MsgGrantAllowance {
            granter: granter.into(),
            grantee: grantee.into(),
            allowance: Some(build_any_periodic(periodic, basic)),
        }
        .encode_to_vec();

        let msg = CosmosMsg::Stargate {
            type_url: "/cosmos.feegrant.v1beta1.MsgGrantAllowance".to_string(),
            value: to_binary(&msg)?,
        };

        self.base.executor(self.deps).execute(vec![msg])
    }

    /// creates allowance only for BasicAllowance.
    pub fn allow_basic(&self, basic: BasicAllowance) -> AbstractSdkResult<CosmosMsg> {
        let msg = feegrant::v1beta1::AllowedMsgAllowance {
            allowance: Some(build_any_basic(basic)),
            allowed_messages: vec!["BasicAllowance".to_owned()],
        }
        .encode_to_vec();

        let msg = CosmosMsg::Stargate {
            type_url: "/cosmos.feegrant.v1beta1.AllowedMsgAllowance".to_string(),
            value: to_binary(&msg)?,
        };

        self.base.executor(self.deps).execute(vec![msg])
    }

    /// Creates allowance only for PeriodicAllowance.
    pub fn allow_periodic(&self, periodic: PeriodicAllowance) -> AbstractSdkResult<CosmosMsg> {
        let msg = feegrant::v1beta1::AllowedMsgAllowance {
            allowance: Some(build_any_periodic(periodic, None)),
            allowed_messages: vec!["PeriodicAllowance".to_owned()],
        }
        .encode_to_vec();

        let msg = CosmosMsg::Stargate {
            type_url: "/cosmos.feegrant.v1beta1.AllowedMsgAllowance".to_string(),
            value: to_binary(&msg)?,
        };

        self.base.executor(self.deps).execute(vec![msg])
    }

    /// Creates allowance only for BasicAllowance and PeriodicAllowance.
    pub fn allow_both(
        &self,
        basic: BasicAllowance,
        periodic: PeriodicAllowance,
    ) -> AbstractSdkResult<CosmosMsg> {
        let msg = feegrant::v1beta1::AllowedMsgAllowance {
            allowance: Some(build_any_periodic(periodic, Some(basic))),
            allowed_messages: vec!["BasicAllowance".to_owned(), "PeriodicAllowance".to_owned()],
        }
        .encode_to_vec();

        let msg = CosmosMsg::Stargate {
            type_url: "/cosmos.feegrant.v1beta1.AllowedMsgAllowance".to_string(),
            value: to_binary(&msg)?,
        };

        self.base.executor(self.deps).execute(vec![msg])
    }

    /// Removes any existing Allowance from Granter to Grantee.
    pub fn revoke_all(&self, granter: &Addr, grantee: &Addr) -> AbstractSdkResult<CosmosMsg> {
        let msg = feegrant::v1beta1::MsgRevokeAllowance {
            granter: granter.into(),
            grantee: grantee.into(),
        }
        .encode_to_vec();

        let msg = CosmosMsg::Stargate {
            type_url: "/cosmos.feegrant.v1beta1.MsgRevokeAllowance".to_string(),
            value: to_binary(&msg)?,
        };

        self.base.executor(self.deps).execute(vec![msg])
    }
}

/// Details for a basic fee allowance grant
pub struct BasicAllowance {
    /// Maximum amount of tokens that can be spent
    pub spend_limit: Vec<Coin>,
    /// When the grant expires
    pub expiration: Option<Timestamp>,
}

/// Details for a periodic fee allowance grant
pub struct PeriodicAllowance {
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

fn build_any_basic(basic: BasicAllowance) -> Any {
    Any {
        type_url: "/cosmos.feegrant.v1beta1.BasicAllowance".to_string(),
        value: build_basic_allowance(basic).encode_to_vec(),
    }
}

fn build_any_periodic(periodic: PeriodicAllowance, basic: Option<BasicAllowance>) -> Any {
    Any {
        type_url: "/cosmos.feegrant.v1beta1.PeriodicAllowance".to_string(),
        value: feegrant::v1beta1::PeriodicAllowance {
            basic: basic.map(build_basic_allowance),
            period: periodic.period.map(|p| prost_types::Duration {
                seconds: p.as_secs() as i64,
                nanos: 0,
            }),
            period_spend_limit: convert_coins(periodic.period_spend_limit),
            period_can_spend: convert_coins(periodic.period_can_spend),
            period_reset: periodic.period_reset.map(convert_stamp),
        }
        .encode_to_vec(),
    }
}

fn build_basic_allowance(basic: BasicAllowance) -> feegrant::v1beta1::BasicAllowance {
    feegrant::v1beta1::BasicAllowance {
        spend_limit: convert_coins(basic.spend_limit),
        expiration: basic.expiration.map(convert_stamp),
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::mock_module::*;

    use cosmwasm_std::{coins, testing::*};

    use speculoos::prelude::*;

    mod basic_allowance {
        use super::*;

        #[test]
        fn basic_allowance() {
            let app = MockModule::new();
            let deps = mock_dependencies();
            let grant = app.grant(deps.as_ref());

            let granter = Addr::unchecked("granter");
            let grantee = Addr::unchecked("grantee");
            let spend_limit = coins(100, "asset");
            let expiration = Some(Timestamp::from_seconds(10));

            let res = grant.basic(
                &granter,
                &grantee,
                BasicAllowance {
                    spend_limit,
                    expiration,
                },
            );

            assert_that!(&res).is_ok();
        }
    }

    mod periodic_allowance {
        use super::*;

        #[test]
        fn periodic_allowance() {
            let app = MockModule::new();
            let deps = mock_dependencies();
            let grant = app.grant(deps.as_ref());

            let granter = Addr::unchecked("granter");
            let grantee = Addr::unchecked("grantee");
            let spend_limit = coins(100, "asset");
            let period_spend_limit = vec![];
            let period_can_spend = vec![];
            let expiration = Some(Timestamp::from_seconds(10));

            let basic = Some(BasicAllowance {
                spend_limit,
                expiration,
            });

            let periodic = PeriodicAllowance {
                period: None,
                period_spend_limit,
                period_can_spend,
                period_reset: None,
            };

            let res = grant.periodic(&granter, &grantee, basic, periodic);

            assert_that!(&res).is_ok();
        }
    }

    mod allow_basic {
        use super::*;

        #[test]
        fn allow_basic() {
            let app = MockModule::new();
            let deps = mock_dependencies();
            let grant = app.grant(deps.as_ref());

            let spend_limit = coins(100, "asset");
            let expiration = Some(Timestamp::from_seconds(10));

            let res = grant.allow_basic(BasicAllowance {
                spend_limit,
                expiration,
            });

            assert_that!(&res).is_ok();
        }
    }

    mod allow_periodic {
        use super::*;

        #[test]
        fn allow_periodic() {
            let app = MockModule::new();
            let deps = mock_dependencies();
            let grant = app.grant(deps.as_ref());

            let period_spend_limit = vec![];
            let period_can_spend = vec![];

            let res = grant.allow_periodic(PeriodicAllowance {
                period: None,
                period_spend_limit,
                period_can_spend,
                period_reset: None,
            });

            assert_that!(&res).is_ok();
        }
    }

    mod allow_both {
        use super::*;

        #[test]
        fn allow_both() {
            let app = MockModule::new();
            let deps = mock_dependencies();
            let grant = app.grant(deps.as_ref());

            let spend_limit = coins(100, "asset");
            let expiration = Some(Timestamp::from_seconds(10));
            let period_spend_limit = vec![];
            let period_can_spend = vec![];

            let res = grant.allow_both(
                BasicAllowance {
                    spend_limit,
                    expiration,
                },
                PeriodicAllowance {
                    period: None,
                    period_spend_limit,
                    period_can_spend,
                    period_reset: None,
                },
            );

            assert_that!(&res).is_ok();
        }
    }

    mod revoke_all {
        use super::*;

        #[test]
        fn revoke_all() {
            let app = MockModule::new();
            let deps = mock_dependencies();
            let grant = app.grant(deps.as_ref());

            let granter = Addr::unchecked("granter");
            let grantee = Addr::unchecked("grantee");

            let res = grant.revoke_all(&granter, &grantee);

            assert_that!(&res).is_ok();
        }
    }
}
