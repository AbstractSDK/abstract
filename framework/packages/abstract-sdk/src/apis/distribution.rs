//! # Distribution
//! Interacts with the distribution module of cosmos
//!

use cosmos_sdk_proto::{
    cosmos::{base, distribution},
    prost::Name,
    traits::Message,
};
use cosmwasm_std::{to_json_binary, Addr, AnyMsg, Coin, CosmosMsg};

use crate::{features::AccountExecutor, AbstractSdkResult, AccountAction};

/// Interact with the Cosmos SDK Distribution module.
/// Requires `Stargate` feature.
pub trait DistributionInterface: AccountExecutor {
    /**
        API for accessing the Cosmos SDK distribution module.

        # Example
        ```
        use abstract_sdk::prelude::*;
        # use cosmwasm_std::testing::mock_dependencies;
        # use abstract_sdk::mock_module::MockModule;
        # use abstract_testing::prelude::*;
        # let deps = mock_dependencies();
        # let account = admin_account(deps.api);
        # let module = MockModule::new(deps.api, account);

        let distr: Distribution  = module.distribution();
        ```
    */
    fn distribution(&self) -> Distribution {
        Distribution {}
    }
}

impl<T> DistributionInterface for T where T: AccountExecutor {}

/**
    API for accessing the Cosmos SDK distribution module.

    # Example
    ```
    use abstract_sdk::prelude::*;
    # use cosmwasm_std::testing::mock_dependencies;
    # use abstract_sdk::mock_module::MockModule;
    # use abstract_testing::prelude::*;
    # let deps = mock_dependencies();
    # let account = admin_account(deps.api);
    # let module = MockModule::new(deps.api, account);

    let distr: Distribution  = module.distribution();
    ```
*/
#[derive(Clone)]
pub struct Distribution {}

impl Distribution {
    /// sets the withdraw address for a delegator (or validator self-delegation).
    pub fn set_withdraw_address(
        &self,
        delegator: &Addr,
        withdraw: &Addr,
    ) -> AbstractSdkResult<AccountAction> {
        let msg = distribution::v1beta1::MsgSetWithdrawAddress {
            delegator_address: delegator.into(),
            withdraw_address: withdraw.into(),
        }
        .encode_to_vec();

        let msg = CosmosMsg::Stargate {
            type_url: distribution::v1beta1::MsgSetWithdrawAddress::type_url(),
            value: to_json_binary(&msg)?,
        };

        Ok(msg.into())
    }

    /// represents delegation withdrawal to a delegator from a single validator.
    pub fn withdraw_delegator_reward(
        &self,
        validator: &Addr,
        delegator: &Addr,
    ) -> AbstractSdkResult<AccountAction> {
        let msg = distribution::v1beta1::MsgWithdrawDelegatorReward {
            validator_address: validator.into(),
            delegator_address: delegator.into(),
        }
        .encode_to_vec();

        let msg = CosmosMsg::Stargate {
            type_url: distribution::v1beta1::MsgWithdrawDelegatorReward::type_url(),
            value: to_json_binary(&msg)?,
        };

        Ok(msg.into())
    }

    /// withdraws the full commission to the validator address.
    pub fn withdraw_delegator_commission(
        &self,
        validator: &Addr,
    ) -> AbstractSdkResult<AccountAction> {
        let msg = distribution::v1beta1::MsgWithdrawValidatorCommission {
            validator_address: validator.into(),
        }
        .encode_to_vec();

        let msg = CosmosMsg::Stargate {
            type_url: distribution::v1beta1::MsgWithdrawValidatorCommission::type_url(),
            value: to_json_binary(&msg)?,
        };

        Ok(msg.into())
    }

    /// allows an account to directly fund the community pool.
    pub fn fund_community_pool(
        &self,
        amount: &[Coin],
        depositor: &Addr,
    ) -> AbstractSdkResult<AccountAction> {
        let msg = distribution::v1beta1::MsgFundCommunityPool {
            amount: amount
                .iter()
                .map(|item| base::v1beta1::Coin {
                    denom: item.denom.to_owned(),
                    amount: item.amount.to_string(),
                })
                .collect(),
            depositor: depositor.into(),
        }
        .encode_to_vec();

        let msg = CosmosMsg::Stargate {
            type_url: distribution::v1beta1::MsgFundCommunityPool::type_url(),
            value: to_json_binary(&msg)?,
        };

        Ok(msg.into())
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::needless_borrows_for_generic_args)]
    use speculoos::prelude::*;

    use super::*;
    use crate::mock_module::*;
    use abstract_testing::prelude::*;
    use cosmwasm_std::testing::MockApi;

    mod set_withdraw_address {
        use super::*;

        #[test]
        fn set_withdraw_address() {
            let mock_api = MockApi::default();
            let app = MockModule::new(mock_api, test_account(mock_api));

            let distribution = app.distribution();

            let delegator = mock_api.addr_make("delegator");
            let withdraw = mock_api.addr_make("withdraw");

            let res = distribution.set_withdraw_address(&delegator, &withdraw);

            assert_that!(&res).is_ok();
        }
    }

    mod withdraw_delegator_reward {
        use super::*;

        #[test]
        fn withdraw_delegator_reward() {
            let mock_api = MockApi::default();
            let app = MockModule::new(mock_api, test_account(mock_api));

            let distribution = app.distribution();

            let validator = mock_api.addr_make("validator");
            let delegator = mock_api.addr_make("delegator");

            let res = distribution.withdraw_delegator_reward(&validator, &delegator);

            assert_that!(&res).is_ok();
        }
    }

    mod withdraw_delegator_comission {
        use super::*;

        #[test]
        fn withdraw_delegator_comission() {
            let mock_api = MockApi::default();
            let app = MockModule::new(mock_api, test_account(mock_api));

            let distribution = app.distribution();

            let validator = mock_api.addr_make("validator");

            let res = distribution.withdraw_delegator_commission(&validator);

            assert_that!(&res).is_ok();
        }
    }

    mod fund_community_pool {
        use cosmwasm_std::coins;

        use super::*;

        #[test]
        fn fund_community_pool() {
            let mock_api = MockApi::default();
            let app = MockModule::new(mock_api, test_account(mock_api));

            let distribution = app.distribution();

            let depositor = mock_api.addr_make("depositor");
            let amount = coins(1000, "coin");

            let res = distribution.fund_community_pool(&amount, &depositor);

            assert_that!(&res).is_ok();
        }
    }
}
