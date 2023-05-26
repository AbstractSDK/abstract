//! # Distribution
//! Interacts with the distribution module of cosmos
//!

use crate::features::AccountIdentification;
use cosmos_sdk_proto::{
    cosmos::{base, distribution},
    traits::Message,
};
use cosmwasm_std::{to_binary, Addr, Coin, CosmosMsg};

use crate::AbstractSdkResult;
use crate::AccountAction;

/// Interact with the Cosmos SDK Distribution module.
/// Requires `Stargate` feature.
pub trait DistributionInterface: AccountIdentification {
    /**
        API for accessing the Cosmos SDK distribution module.

        # Example
        ```
        use abstract_sdk::prelude::*;
        # use cosmwasm_std::testing::mock_dependencies;
        # use abstract_sdk::mock_module::MockModule;
        # let module = MockModule::new();

        let distr: Distribution  = module.distribution();
        ```
    */
    fn distribution(&self) -> Distribution {
        Distribution {}
    }
}

impl<T> DistributionInterface for T where T: AccountIdentification {}

/**
    API for accessing the Cosmos SDK distribution module.

    # Example
    ```
    use abstract_sdk::prelude::*;
    # use cosmwasm_std::testing::mock_dependencies;
    # use abstract_sdk::mock_module::MockModule;
    # let module = MockModule::new();

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
            type_url: "/cosmos.distribution.v1beta1.MsgSetWithdrawAddress".to_string(),
            value: to_binary(&msg)?,
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
            type_url: "/cosmos.distribution.v1beta1.MsgWithdrawDelegatorReward".to_string(),
            value: to_binary(&msg)?,
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
            type_url: "/cosmos.distribution.v1beta1.MsgWithdrawValidatorCommission".to_string(),
            value: to_binary(&msg)?,
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
            type_url: "/cosmos.distribution.v1beta1.MsgFundCommunityPool".to_string(),
            value: to_binary(&msg)?,
        };

        Ok(msg.into())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::mock_module::*;
    use speculoos::prelude::*;

    mod set_withdraw_address {
        use super::*;

        #[test]
        fn set_withdraw_address() {
            let app = MockModule::new();

            let distribution = app.distribution();

            let delegator = Addr::unchecked("delegator");
            let withdraw = Addr::unchecked("withdraw");

            let res = distribution.set_withdraw_address(&delegator, &withdraw);

            assert_that!(&res).is_ok();
        }
    }

    mod withdraw_delegator_reward {
        use super::*;

        #[test]
        fn withdraw_delegator_reward() {
            let app = MockModule::new();

            let distribution = app.distribution();

            let validator = Addr::unchecked("validator");
            let delegator = Addr::unchecked("delegator");

            let res = distribution.withdraw_delegator_reward(&validator, &delegator);

            assert_that!(&res).is_ok();
        }
    }

    mod withdraw_delegator_comission {
        use super::*;

        #[test]
        fn withdraw_delegator_comission() {
            let app = MockModule::new();

            let distribution = app.distribution();

            let validator = Addr::unchecked("validator");

            let res = distribution.withdraw_delegator_commission(&validator);

            assert_that!(&res).is_ok();
        }
    }

    mod fund_community_pool {
        use super::*;
        use cosmwasm_std::coins;

        #[test]
        fn fund_community_pool() {
            let app = MockModule::new();

            let distribution = app.distribution();

            let depositor = Addr::unchecked("depositor");
            let amount = coins(1000, "coin");

            let res = distribution.fund_community_pool(&amount, &depositor);

            assert_that!(&res).is_ok();
        }
    }
}
