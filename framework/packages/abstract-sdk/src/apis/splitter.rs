#![allow(unused)]
use abstract_std::objects::AnsAsset;
use cosmwasm_std::{Addr, CosmosMsg, Deps, Env, StdResult, Uint128};

use super::AbstractApi;
use crate::{
    features::{AccountExecutor, ModuleIdentification},
    AbstractSdkResult, AccountAction, TransferInterface,
};
// ANCHOR: splitter
// Trait to retrieve the Splitter object
// Depends on the ability to transfer funds
pub trait SplitterInterface: TransferInterface + AccountExecutor + ModuleIdentification {
    fn splitter<'a>(&'a self, deps: Deps<'a>, env: &'a Env) -> Splitter<Self> {
        Splitter {
            base: self,
            deps,
            env,
        }
    }
}

// Implement for every object that can transfer funds
impl<T> SplitterInterface for T where T: TransferInterface + AccountExecutor + ModuleIdentification {}

impl<'a, T: SplitterInterface> AbstractApi<T> for Splitter<'a, T> {
    const API_ID: &'static str = "Splitter";

    fn base(&self) -> &T {
        self.base
    }
    fn deps(&self) -> Deps {
        self.deps
    }
}

#[derive(Clone)]
pub struct Splitter<'a, T: SplitterInterface> {
    base: &'a T,
    deps: Deps<'a>,
    env: &'a Env,
}

impl<'a, T: SplitterInterface> Splitter<'a, T> {
    /// Split an asset to multiple users
    pub fn split(&self, asset: AnsAsset, receivers: &[Addr]) -> AbstractSdkResult<AccountAction> {
        // split the asset between all receivers
        let receives_each = AnsAsset {
            amount: asset
                .amount
                .multiply_ratio(Uint128::one(), Uint128::from(receivers.len() as u128)),
            ..asset
        };

        // Retrieve the bank API
        let bank = self.base.bank(self.deps, self.env);
        receivers
            .iter()
            .map(|receiver| {
                // Construct the transfer message
                bank.transfer(vec![&receives_each], receiver)
            })
            .try_fold(AccountAction::default(), |mut acc, v| match v {
                Ok(action) => {
                    // Merge two AccountAction objects
                    acc.merge(action);
                    Ok(acc)
                }
                Err(e) => Err(e),
            })
    }
}
// ANCHOR_END: splitter

#[cfg(test)]
mod test {
    #![allow(clippy::needless_borrows_for_generic_args)]
    use abstract_std::objects::AnsAsset;
    use abstract_testing::{abstract_mock_querier_builder, prelude::*};
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env},
        Addr, CosmosMsg, Response, StdError, Uint128,
    };

    use crate::{
        apis::{splitter::SplitterInterface, traits::test::abstract_api_test},
        mock_module::MockModule,
        AbstractSdkError, Execution, ExecutorMsg,
    };

    fn split() -> Result<Response, AbstractSdkError> {
        let mut deps = mock_dependencies();
        let account = test_account(deps.api);
        deps.querier = abstract_mock_querier_builder(deps.api)
            .account(&account, TEST_ACCOUNT_ID)
            .build();
        let module = MockModule::new(deps.api, account.clone());
        // ANCHOR: usage
        let asset = AnsAsset {
            amount: Uint128::from(100u128),
            name: "usd".into(),
        };

        let receivers = vec![
            deps.api.addr_make("receiver1"),
            deps.api.addr_make("receiver2"),
            deps.api.addr_make("receiver3"),
        ];

        let split_funds = module
            .splitter(deps.as_ref(), &mock_env_validated(deps.api))
            .split(asset, &receivers)?;
        assert_eq!(split_funds.messages().len(), 3);

        let msg: ExecutorMsg = module.executor(deps.as_ref()).execute(vec![split_funds])?;

        Ok(Response::new().add_message(msg))
        // ANCHOR_END: usage
    }

    #[coverage_helper::test]
    fn abstract_api() {
        let mut deps = mock_dependencies();
        let account = test_account(deps.api);
        let module = MockModule::new(deps.api, account.clone());
        let env = mock_env_validated(deps.api);
        let splitter = module.splitter(deps.as_ref(), &env);

        abstract_api_test(splitter);
    }
}
