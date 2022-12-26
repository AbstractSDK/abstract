//! # Bank
//! The Bank object handles asset transfers to and from the OS.

use abstract_os::objects::AnsAsset;
use cosmwasm_std::{Addr, BankMsg, Coin, CosmosMsg, Deps, StdResult};

use super::{execution::Execution, AbstractNameService};
use crate::ans_resolve::Resolve;

/// Bank assets from and to the Abstract OS.
pub trait TransferInterface: AbstractNameService + Execution {
    fn bank<'a>(&'a self, deps: Deps<'a>) -> Bank<Self> {
        Bank { base: self, deps }
    }
}

impl<T> TransferInterface for T where T: AbstractNameService + Execution {}

#[derive(Clone)]
pub struct Bank<'a, T: TransferInterface> {
    base: &'a T,
    deps: Deps<'a>,
}

impl<'a, T: TransferInterface> Bank<'a, T> {
    /// Transfer the provided **funds** from the OS' vault to the **recipient**.
    /// The caller must be a whitelisted module or trader.
    ///
    /// ```rust
    /// use cosmwasm_std::{Addr, Response, StdResult, Deps, DepsMut, Env, MessageInfo};
    /// use abstract_os::objects::AnsAsset;
    /// # use abstract_os::objects::ans_host::AnsHost;
    /// use abstract_sdk::{
    ///     # base::features::{Identification, AbstractNameService},
    ///     TransferInterface
    /// };
    /// #
    /// # struct MockModule;
    /// # impl Identification for MockModule {
    /// #    fn proxy_address(&self, _deps: Deps) -> StdResult<Addr> {
    /// #       unimplemented!("Not needed for this example")
    /// #   }
    /// # }
    ///
    /// # impl AbstractNameService for MockModule {
    /// #   fn ans_host(&self, _deps: Deps) -> StdResult<AnsHost> {
    /// #     unimplemented!("Not needed for this example")
    /// #  }
    /// # }
    ///
    /// fn transfer_asset_to_sender(app: MockModule, deps: DepsMut, info: MessageInfo, requested_asset: AnsAsset) -> StdResult<Response> {
    ///     // check that the caller has the rights to the asset
    ///     // ...
    ///     let bank = app.bank(deps.as_ref());
    ///
    ///     let transfer_msg = bank.transfer(vec![requested_asset.clone()], &info.sender)?;
    ///
    ///     Ok(Response::new()
    ///         .add_message(transfer_msg)
    ///         .add_attribute("recipient", info.sender)
    ///         .add_attribute("asset_sent", requested_asset.to_string()))
    /// }
    /// ```
    pub fn transfer(&self, funds: Vec<AnsAsset>, recipient: &Addr) -> StdResult<CosmosMsg> {
        let resolved_funds = funds.resolve(&self.deps.querier, &self.base.ans_host(self.deps)?)?;
        let transfer_msgs = resolved_funds
            .iter()
            .map(|asset| asset.transfer_msg(recipient.clone()))
            .collect::<StdResult<Vec<CosmosMsg>>>();
        self.base.executor(self.deps).execute(transfer_msgs?)
    }

    /// Transfer **coins** from the OS' vault to the **recipient**.
    /// The caller must be a whitelisted module or trader.
    pub fn transfer_coins(&self, coins: Vec<Coin>, recipient: &Addr) -> StdResult<CosmosMsg> {
        let send_msg = CosmosMsg::Bank(BankMsg::Send {
            to_address: recipient.to_string(),
            amount: coins,
        });
        self.base.executor(self.deps).execute(vec![send_msg])
    }

    /// Transfer the **funds** (deposit) into the OS from the current contract.
    pub fn deposit(&self, funds: Vec<AnsAsset>) -> StdResult<Vec<CosmosMsg>> {
        let recipient = self.base.proxy_address(self.deps)?;
        let resolved_funds = funds.resolve(&self.deps.querier, &self.base.ans_host(self.deps)?)?;
        resolved_funds
            .iter()
            .map(|asset| asset.transfer_msg(recipient.clone()))
            .collect::<StdResult<Vec<CosmosMsg>>>()
    }

    /// Deposit coins into the OS
    pub fn deposit_coins(&self, coins: Vec<Coin>) -> StdResult<CosmosMsg> {
        let recipient = self.base.proxy_address(self.deps)?.into_string();
        Ok(CosmosMsg::Bank(BankMsg::Send {
            to_address: recipient,
            amount: coins,
        }))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::apis::test_common::*;

    mod transfer_coins {
        use super::*;
        use os::proxy::ExecuteMsg::ModuleAction;

        #[test]
        fn transfer_asset_to_sender() {
            let app = MockModule::new();
            let deps = mock_dependencies();
            let expected_amount = 100u128;
            let expected_recipient = Addr::unchecked("recipient");

            let bank = app.bank(deps.as_ref());
            let coins = coins(expected_amount, "asset");
            let actual_res = bank.transfer_coins(coins.clone(), &expected_recipient);

            assert_that!(actual_res).is_ok();

            let expected_msg: CosmosMsg = wasm_execute(
                TEST_PROXY,
                &ModuleAction {
                    // actual assertion
                    msgs: vec![CosmosMsg::Bank(BankMsg::Send {
                        to_address: expected_recipient.to_string(),
                        amount: coins,
                    })],
                },
                vec![],
            )
            .unwrap()
            .into();

            assert_that!(actual_res.unwrap()).is_equal_to(expected_msg);
        }
    }

    // transfer must be tested via integration test

    mod deposit_coins {
        use super::*;

        #[test]
        fn deposit_coins() {
            let app = MockModule::new();
            let deps = mock_dependencies();
            let expected_amount = 100u128;

            let bank = app.bank(deps.as_ref());
            let coins = coins(expected_amount, "asset");
            let actual_res = bank.deposit_coins(coins.clone());

            let expected_msg: CosmosMsg = CosmosMsg::Bank(BankMsg::Send {
                to_address: TEST_PROXY.to_string(),
                amount: coins,
            });

            assert_that!(actual_res).is_ok().is_equal_to(expected_msg);
        }
    }

    // deposit must be tested via integration test
}
