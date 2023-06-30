//! # Bank
//! The Bank object handles asset transfers to and from the Account.

use crate::features::AccountIdentification;
use crate::AccountAction;
use crate::{ans_resolve::Resolve, features::AbstractNameService, AbstractSdkResult};
use core::objects::{AnsAsset, AssetEntry};
use cosmwasm_std::to_binary;
use cosmwasm_std::{Addr, Coin, CosmosMsg, Deps, Env};
use cw_asset::Asset;
use serde::Serialize;

/// Query and Transfer assets from and to the Abstract Account.
pub trait TransferInterface: AbstractNameService + AccountIdentification {
    /**
        API for transferring funds to and from the account.

        # Example
        ```
        use abstract_sdk::prelude::*;
        # use cosmwasm_std::testing::mock_dependencies;
        # use abstract_sdk::mock_module::MockModule;
        # let module = MockModule::new();
        # let deps = mock_dependencies();

        let bank: Bank<MockModule>  = module.bank(deps.as_ref());
        ```
    */
    fn bank<'a>(&'a self, deps: Deps<'a>) -> Bank<Self> {
        Bank { base: self, deps }
    }
}

impl<T> TransferInterface for T where T: AbstractNameService + AccountIdentification {}

/**
    API for transferring funds to and from the account.

    # Example
    ```
    use abstract_sdk::prelude::*;
    # use cosmwasm_std::testing::mock_dependencies;
    # use abstract_sdk::mock_module::MockModule;
    # let module = MockModule::new();
    # let deps = mock_dependencies();

    let bank: Bank<MockModule>  = module.bank(deps.as_ref());
    ```
*/
#[derive(Clone)]
pub struct Bank<'a, T: TransferInterface> {
    base: &'a T,
    deps: Deps<'a>,
}

impl<'a, T: TransferInterface> Bank<'a, T> {
    /// Get the balances of the provided assets.
    pub fn balances(&self, assets: &[AssetEntry]) -> AbstractSdkResult<Vec<Asset>> {
        assets
            .iter()
            .map(|asset| self.balance(asset))
            .collect::<AbstractSdkResult<Vec<Asset>>>()
    }
    /// Get the balance of the provided asset.
    pub fn balance(&self, asset: &AssetEntry) -> AbstractSdkResult<Asset> {
        let resolved_info = asset.resolve(&self.deps.querier, &self.base.ans_host(self.deps)?)?;
        let balance =
            resolved_info.query_balance(&self.deps.querier, self.base.proxy_address(self.deps)?)?;
        Ok(Asset::new(resolved_info, balance))
    }

    /// Transfer the provided funds from the Account to the recipient.
    /// ```
    /// # use cosmwasm_std::{Addr, Response, Deps, DepsMut, MessageInfo};
    /// # use abstract_core::objects::AnsAsset;
    /// # use abstract_core::objects::ans_host::AnsHost;
    /// # use abstract_sdk::{
    /// #    features::{AccountIdentification, AbstractNameService, ModuleIdentification},
    /// #    TransferInterface, AbstractSdkResult, Execution,
    /// # };
    /// # struct MockModule;
    /// # impl AccountIdentification for MockModule {
    /// #    fn proxy_address(&self, _deps: Deps) -> AbstractSdkResult<Addr> {
    /// #       unimplemented!("Not needed for this example")
    /// #   }
    /// # }
    /// #
    /// # impl ModuleIdentification for MockModule {
    /// #   fn module_id(&self) -> &'static str {
    /// #      "mock_module"
    /// #  }
    /// # }
    /// #
    /// # impl AbstractNameService for MockModule {
    /// #   fn ans_host(&self, _deps: Deps) -> AbstractSdkResult<AnsHost> {
    /// #     unimplemented!("Not needed for this example")
    /// #  }
    /// # }
    /// fn transfer_asset_to_sender(app: MockModule, deps: DepsMut, info: MessageInfo, requested_asset: AnsAsset) -> AbstractSdkResult<Response> {
    ///     let bank = app.bank(deps.as_ref());
    ///     let executor = app.executor(deps.as_ref());    
    ///     let transfer_action = bank.transfer(vec![requested_asset.clone()], &info.sender)?;
    ///
    ///     let transfer_msg = executor.execute(vec![transfer_action])?;
    ///
    ///     Ok(Response::new()
    ///         .add_message(transfer_msg)
    ///         .add_attribute("recipient", info.sender)
    ///         .add_attribute("asset_sent", requested_asset.to_string()))
    /// }
    /// ```
    pub fn transfer<R: Transferable>(
        &self,
        funds: Vec<R>,
        recipient: &Addr,
    ) -> AbstractSdkResult<AccountAction> {
        let transferable_funds = funds
            .into_iter()
            .map(|asset| asset.transferable_asset(self.base, self.deps))
            .collect::<AbstractSdkResult<Vec<Asset>>>()?;
        let msgs = transferable_funds
            .iter()
            .map(|asset| asset.transfer_msg(recipient.clone()))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(AccountAction::from_vec(msgs))
    }

    /// Move funds from the contract into the Account.
    pub fn deposit<R: Transferable>(&self, funds: Vec<R>) -> AbstractSdkResult<Vec<CosmosMsg>> {
        let recipient = self.base.proxy_address(self.deps)?;
        let transferable_funds = funds
            .into_iter()
            .map(|asset| asset.transferable_asset(self.base, self.deps))
            .collect::<AbstractSdkResult<Vec<Asset>>>()?;
        transferable_funds
            .iter()
            .map(|asset| asset.transfer_msg(recipient.clone()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(Into::into)
            .map(Into::into)
    }

    /// Withdraw funds from the Account to this contract.
    pub fn withdraw<R: Transferable>(
        &self,
        env: &Env,
        funds: Vec<R>,
    ) -> AbstractSdkResult<AccountAction> {
        let recipient = &env.contract.address;
        self.transfer(funds, recipient)
    }

    /// Move cw20 assets from the Account to a recipient with the possibility using the cw20 send/receive hook
    ///
    /// Note:  **Native coins are NOT and will NEVER be supported by this method**.
    ///
    /// In order to send funds with your message, you need to construct the message yourself
    pub fn send<R: Transferable, M: Serialize>(
        &self,
        funds: R,
        recipient: &Addr,
        message: &M,
    ) -> AbstractSdkResult<AccountAction> {
        let transferable_funds = funds.transferable_asset(self.base, self.deps)?;

        let msgs = transferable_funds.send_msg(recipient, to_binary(message)?)?;

        Ok(AccountAction::from_vec(vec![msgs]))
    }
}

/// Turn an object that represents an asset into the blockchain representation of an asset, i.e. [`Asset`].
pub trait Transferable {
    /// Turn an object that represents an asset into the blockchain representation of an asset, i.e. [`Asset`].
    fn transferable_asset<T: AbstractNameService>(
        self,
        base: &T,
        deps: Deps,
    ) -> AbstractSdkResult<Asset>;
}

impl Transferable for &AnsAsset {
    fn transferable_asset<T: AbstractNameService>(
        self,
        base: &T,
        deps: Deps,
    ) -> AbstractSdkResult<Asset> {
        self.resolve(&deps.querier, &base.ans_host(deps)?)
    }
}

impl Transferable for AnsAsset {
    fn transferable_asset<T: AbstractNameService>(
        self,
        base: &T,
        deps: Deps,
    ) -> AbstractSdkResult<Asset> {
        self.resolve(&deps.querier, &base.ans_host(deps)?)
    }
}

impl Transferable for Asset {
    fn transferable_asset<T: AbstractNameService>(
        self,
        _base: &T,
        _deps: Deps,
    ) -> AbstractSdkResult<Asset> {
        Ok(self)
    }
}

impl Transferable for Coin {
    fn transferable_asset<T: AbstractNameService>(
        self,
        _base: &T,
        _deps: Deps,
    ) -> AbstractSdkResult<Asset> {
        Ok(Asset::from(self))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::mock_module::*;
    use abstract_testing::prelude::*;
    use cosmwasm_std::{testing::*, *};
    use speculoos::prelude::*;

    mod transfer_coins {
        use abstract_core::proxy::ExecuteMsg;

        use crate::{Execution, Executor};

        use super::*;

        #[test]
        fn transfer_asset_to_sender() {
            let app = MockModule::new();
            let deps = mock_dependencies();

            // ANCHOR: transfer
            let recipient: Addr = Addr::unchecked("recipient");
            let bank: Bank<'_, MockModule> = app.bank(deps.as_ref());
            let coins: Vec<Coin> = coins(100u128, "asset");
            let bank_transfer: AccountAction = bank.transfer(coins.clone(), &recipient).unwrap();

            let executor: Executor<'_, MockModule> = app.executor(deps.as_ref());
            let account_message: CosmosMsg = executor.execute(vec![bank_transfer]).unwrap();
            let response: Response = Response::new().add_message(account_message);
            // ANCHOR_END: transfer

            let expected_msg = CosmosMsg::Bank(BankMsg::Send {
                to_address: recipient.to_string(),
                amount: coins,
            });

            assert_that!(response.messages[0].msg).is_equal_to(
                &wasm_execute(
                    TEST_PROXY,
                    &ExecuteMsg::ModuleAction {
                        msgs: vec![expected_msg],
                    },
                    vec![],
                )
                .unwrap()
                .into(),
            );
        }
    }

    // transfer must be tested via integration test

    mod deposit {
        use super::*;

        #[test]
        fn deposit() {
            let app = MockModule::new();
            let deps = mock_dependencies();

            // ANCHOR: deposit
            // Get bank API struct from the app
            let bank: Bank<'_, MockModule> = app.bank(deps.as_ref());
            // Create coins to deposit
            let coins: Vec<Coin> = coins(100u128, "asset");
            // Construct messages for deposit (transfer from this contract to the account)
            let deposit_msgs: Vec<CosmosMsg> = bank.deposit(coins.clone()).unwrap();
            // Add to response
            let response: Response = Response::new().add_messages(deposit_msgs);
            // ANCHOR_END: deposit

            let expected_msg: CosmosMsg = CosmosMsg::Bank(BankMsg::Send {
                to_address: TEST_PROXY.to_string(),
                amount: coins,
            });

            assert_that!(response.messages[0].msg).is_equal_to::<CosmosMsg>(expected_msg);
        }
    }

    mod withdraw_coins {
        use super::*;

        #[test]
        fn withdraw_coins() {
            let app = MockModule::new();
            let deps = mock_dependencies();
            let expected_amount = 100u128;
            let env = mock_env();

            let bank = app.bank(deps.as_ref());
            let coins = coins(expected_amount, "asset");
            let actual_res = bank.withdraw(&env, coins.clone());

            let expected_msg: CosmosMsg = CosmosMsg::Bank(BankMsg::Send {
                to_address: env.contract.address.to_string(),
                amount: coins,
            });

            assert_that!(actual_res.unwrap().messages()[0]).is_equal_to::<CosmosMsg>(expected_msg);
        }
    }

    mod send_coins {
        use cw20::Cw20ExecuteMsg;
        use cw_asset::AssetError;

        use crate::AbstractSdkError;

        use super::*;

        #[test]
        fn send_cw20() {
            let app = MockModule::new();
            let deps = mock_dependencies();
            let expected_amount = 100u128;
            let expected_recipient = Addr::unchecked("recipient");

            let bank = app.bank(deps.as_ref());
            let hook_msg = Empty {};
            let asset = Addr::unchecked("asset");
            let coin = Asset::cw20(asset.clone(), expected_amount);
            let actual_res = bank.send(coin, &expected_recipient, &hook_msg);

            let expected_msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: asset.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Send {
                    contract: expected_recipient.to_string(),
                    amount: expected_amount.into(),
                    msg: to_binary(&hook_msg).unwrap(),
                })
                .unwrap(),
                funds: vec![],
            });

            assert_that!(actual_res.unwrap().messages()[0]).is_equal_to::<CosmosMsg>(expected_msg);
        }

        #[test]
        fn send_coins() {
            let app = MockModule::new();
            let deps = mock_dependencies();
            let expected_amount = 100u128;
            let expected_recipient = Addr::unchecked("recipient");

            let bank = app.bank(deps.as_ref());
            let coin = coin(expected_amount, "asset");
            let hook_msg = Empty {};
            let actual_res = bank.send(coin, &expected_recipient, &hook_msg);

            assert_that!(actual_res.unwrap_err()).is_equal_to::<AbstractSdkError>(
                AbstractSdkError::Asset(AssetError::UnavailableMethodForNative {
                    method: "send".into(),
                }),
            );
        }
    }
}
