#![allow(unused)]
use abstract_std::objects::AnsAsset;
use cosmwasm_std::{Addr, CosmosMsg, Deps, Env, StdResult, Uint128};

use super::AbstractApi;
use crate::{
    features::{AccountExecutor, ModuleIdentification},
    AbstractSdkResult, AccountAction, TransferInterface,
};
// ANCHOR: splitter
/// This trait allows to retrieve the Splitter object to split funds amongst multiple receivers
pub trait SplitterInterface: TransferInterface + AccountExecutor + ModuleIdentification {
    fn splitter<'a>(&'a self, deps: Deps<'a>, env: &'a Env) -> Splitter<Self> {
        Splitter {
            base: self,
            deps,
            env,
        }
    }
}

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
    use abstract_std::objects::{AnsAsset, AssetEntry};
    use abstract_unit_test_utils::{abstract_mock_querier_builder, prelude::*};
    use cosmwasm_std::{
        coins,
        testing::{mock_dependencies, mock_env},
        Addr, BankMsg, CosmosMsg, Empty, Response, StdError, SubMsg, Uint128, WasmMsg,
    };
    use cw_asset::AssetInfo;

    use crate::{
        apis::{splitter::SplitterInterface, traits::test::abstract_api_test},
        mock_module::MockModule,
        AbstractSdkError, Execution, ExecutorMsg,
    };

    #[coverage_helper::test]
    fn split() -> Result<(), AbstractSdkError> {
        let mut deps = mock_dependencies();
        let env = mock_env_validated(deps.api);
        let account = test_account(deps.api);
        let abstr = AbstractMockAddrs::new(deps.api);
        deps.querier = abstract_mock_querier_builder(deps.api)
            .account(&account, TEST_ACCOUNT_ID)
            .assets(vec![(&AssetEntry::new("usd"), AssetInfo::native("usd"))])
            .build();
        let module = MockModule::new(deps.api, account.clone());
        let receiver1 = deps.api.addr_make("receiver1");
        let receiver2 = deps.api.addr_make("receiver2");
        let receiver3 = deps.api.addr_make("receiver3");
        let usage_anchor: Result<Response, AbstractSdkError> = {
            // ANCHOR: usage
            let asset = AnsAsset {
                amount: Uint128::from(100u128),
                name: "usd".into(),
            };

            let receivers = vec![receiver1, receiver2, receiver3];

            let split_funds = module
                .splitter(deps.as_ref(), &env)
                .split(asset, &receivers)?;
            assert_eq!(split_funds.messages().len(), 3);

            let msg: ExecutorMsg = module.executor(deps.as_ref()).execute(vec![split_funds])?;

            Ok(Response::new().add_message(msg))
            // ANCHOR_END: usage
        };
        let response = usage_anchor.unwrap();
        assert_eq!(
            response.messages,
            vec![SubMsg::new(WasmMsg::Execute {
                contract_addr: account.addr().to_string(),
                msg: to_json_binary(&abstract_std::account::ExecuteMsg::Execute::<Empty> {
                    msgs: vec![
                        BankMsg::Send {
                            to_address: deps.api.addr_make("receiver1").to_string(),
                            amount: coins(33, "usd")
                        }
                        .into(),
                        BankMsg::Send {
                            to_address: deps.api.addr_make("receiver2").to_string(),
                            amount: coins(33, "usd")
                        }
                        .into(),
                        BankMsg::Send {
                            to_address: deps.api.addr_make("receiver3").to_string(),
                            amount: coins(33, "usd")
                        }
                        .into()
                    ]
                })
                .unwrap(),
                funds: vec![]
            })]
        );

        Ok(())
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
