//! # Bank
//! The Bank object handles asset transfers to and from the Account.

use abstract_std::objects::{ans_host::AnsHostError, AnsAsset, AssetEntry};
use cosmwasm_std::{to_json_binary, Addr, Coin, CosmosMsg, Deps, Env};
use cw_asset::Asset;
use serde::Serialize;

use super::AbstractApi;
use crate::{
    ans_resolve::Resolve,
    cw_helpers::ApiQuery,
    features::{AbstractNameService, AccountExecutor, AccountIdentification, ModuleIdentification},
    AbstractSdkError, AbstractSdkResult, AccountAction,
};

/// Query and Transfer assets from and to the Abstract Account.
pub trait TransferInterface:
    AbstractNameService + AccountIdentification + ModuleIdentification
{
    /**
        API for transferring funds to and from the account.

        # Example
        ```
        use abstract_sdk::prelude::*;
        # use cosmwasm_std::testing::mock_dependencies;
        # use abstract_sdk::mock_module::MockModule;
        # use abstract_testing::prelude::*;
        # let deps = mock_dependencies();
        # let env = mock_env_validated(deps.api);
        # let account = admin_account(deps.api);
        # let module = MockModule::new(deps.api, account);

        let bank: Bank<MockModule>  = module.bank(deps.as_ref(), &env);
        ```
    */
    fn bank<'a>(&'a self, deps: Deps<'a>, env: &'a Env) -> Bank<Self> {
        Bank {
            base: self,
            deps,
            env,
        }
    }
}

impl<T> TransferInterface for T where
    T: AbstractNameService + AccountIdentification + ModuleIdentification
{
}

impl<'a, T: TransferInterface> AbstractApi<T> for Bank<'a, T> {
    const API_ID: &'static str = "Bank";

    fn base(&self) -> &T {
        self.base
    }
    fn deps(&self) -> Deps {
        self.deps
    }
}

/**
    API for transferring funds to and from the account.

    # Example
    ```
    use abstract_sdk::prelude::*;
    # use cosmwasm_std::testing::mock_dependencies;
    # use abstract_sdk::mock_module::MockModule;
    # use abstract_testing::prelude::*;
    # let deps = mock_dependencies();
    # let env = mock_env_validated(deps.api);
    # let account = admin_account(deps.api);
    # let module = MockModule::new(deps.api, account);

    let bank: Bank<MockModule>  = module.bank(deps.as_ref(), &env);
    ```
*/
#[derive(Clone)]
pub struct Bank<'a, T: TransferInterface> {
    base: &'a T,
    deps: Deps<'a>,
    env: &'a Env,
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
        let resolved_info = asset
            .resolve(
                &self.deps.querier,
                &self.base.ans_host(self.deps, self.env)?,
            )
            .map_err(|error| self.wrap_query_error(error))?;
        let balance = resolved_info.query_balance(
            &self.deps.querier,
            self.base.account(self.deps)?.into_addr(),
        )?;
        Ok(Asset::new(resolved_info, balance))
    }

    /// Move funds from the contract into the Account.
    pub fn deposit<R: Transferable>(&self, funds: Vec<R>) -> AbstractSdkResult<Vec<CosmosMsg>> {
        let recipient = self.base.account(self.deps)?.into_addr();
        let transferable_funds = funds
            .into_iter()
            .map(|asset| asset.transferable_asset(self.base, self.deps, self.env))
            .collect::<AbstractSdkResult<Vec<Asset>>>()?;
        transferable_funds
            .iter()
            .map(|asset| asset.transfer_msg(recipient.clone()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(Into::into)
    }
}

impl<'a, T: TransferInterface + AccountExecutor> Bank<'a, T> {
    /// Transfer the provided funds from the Account to the recipient.
    /// ```
    /// # use cosmwasm_std::{Addr, Response, Deps, DepsMut, MessageInfo, Env};
    /// # use abstract_std::registry::Account;
    /// # use abstract_std::objects::AnsAsset;
    /// # use abstract_std::objects::ans_host::AnsHost;
    /// # use abstract_sdk::{
    /// #    features::{AccountIdentification, AbstractNameService, ModuleIdentification},
    /// #    TransferInterface, AbstractSdkResult, Execution,
    /// # };
    /// # struct MockModule;
    /// # impl AccountIdentification for MockModule {
    /// #    fn account(&self, _deps: Deps) -> AbstractSdkResult<Account> {
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
    /// #   fn ans_host(&self, _deps: Deps, env: &Env) -> AbstractSdkResult<AnsHost> {
    /// #     unimplemented!("Not needed for this example")
    /// #  }
    /// # }
    /// fn transfer_asset_to_sender(app: MockModule, deps: DepsMut, info: MessageInfo, env: &Env, requested_asset: AnsAsset) -> AbstractSdkResult<Response> {
    ///     let bank = app.bank(deps.as_ref(), env);
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
            .map(|asset| asset.transferable_asset(self.base, self.deps, self.env))
            .collect::<AbstractSdkResult<Vec<Asset>>>()?;
        let msgs = transferable_funds
            .iter()
            .map(|asset| asset.transfer_msg(recipient.clone()))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(AccountAction::from_vec(msgs))
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
        let transferable_funds = funds.transferable_asset(self.base, self.deps, self.env)?;

        let msgs = transferable_funds.send_msg(recipient, to_json_binary(message)?)?;

        Ok(AccountAction::from_vec(vec![msgs]))
    }
}

/// Turn an object that represents an asset into the blockchain representation of an asset, i.e. [`Asset`].
pub trait Transferable {
    /// Turn an object that represents an asset into the blockchain representation of an asset, i.e. [`Asset`].
    fn transferable_asset<T: AbstractNameService + ModuleIdentification>(
        self,
        base: &T,
        deps: Deps,
        env: &Env,
    ) -> AbstractSdkResult<Asset>;
}

// Helper to wrap errors
fn transferable_api_error(
    base: &impl ModuleIdentification,
    error: AnsHostError,
) -> AbstractSdkError {
    AbstractSdkError::ApiQuery {
        api: "Transferable".to_owned(),
        module_id: base.module_id().to_owned(),
        error: Box::new(error.into()),
    }
}

impl Transferable for &AnsAsset {
    fn transferable_asset<T: AbstractNameService + ModuleIdentification>(
        self,
        base: &T,
        deps: Deps,
        env: &Env,
    ) -> AbstractSdkResult<Asset> {
        self.resolve(&deps.querier, &base.ans_host(deps, env)?)
            .map_err(|error| transferable_api_error(base, error))
    }
}

impl Transferable for AnsAsset {
    fn transferable_asset<T: AbstractNameService + ModuleIdentification>(
        self,
        base: &T,
        deps: Deps,
        env: &Env,
    ) -> AbstractSdkResult<Asset> {
        self.resolve(&deps.querier, &base.ans_host(deps, env)?)
            .map_err(|error| transferable_api_error(base, error))
    }
}

impl Transferable for Asset {
    fn transferable_asset<T: AbstractNameService>(
        self,
        _base: &T,
        _deps: Deps,
        _env: &Env,
    ) -> AbstractSdkResult<Asset> {
        Ok(self)
    }
}

impl Transferable for Coin {
    fn transferable_asset<T: AbstractNameService>(
        self,
        _base: &T,
        _deps: Deps,
        _env: &Env,
    ) -> AbstractSdkResult<Asset> {
        Ok(Asset::from(self))
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::needless_borrows_for_generic_args)]
    use abstract_testing::mock_env_validated;
    use abstract_testing::prelude::*;
    use cosmwasm_std::*;

    use super::*;
    use crate::apis::traits::test::abstract_api_test;
    use crate::mock_module::*;

    mod balance {
        use super::*;

        #[coverage_helper::test]
        fn balance() {
            let (mut deps, account, app) = mock_module_setup();
            let env = mock_env_validated(deps.api);

            // API Query Error
            {
                let bank: Bank<'_, MockModule> = app.bank(deps.as_ref(), &env);
                let res = bank
                    .balances(&[AssetEntry::new("asset_entry")])
                    .unwrap_err();
                let AbstractSdkError::ApiQuery {
                    api,
                    module_id,
                    error: _,
                } = res
                else {
                    panic!("expected api error");
                };
                assert_eq!(api, "Bank");
                assert_eq!(module_id, app.module_id());
            }

            let abstr = abstract_testing::prelude::AbstractMockAddrs::new(deps.api);
            // update querier and balances
            deps.querier = abstract_testing::abstract_mock_querier_builder(deps.api)
                .with_contract_map_entry(
                    &abstr.ans_host,
                    abstract_std::ans_host::state::ASSET_ADDRESSES,
                    (
                        &AssetEntry::new("asset_entry"),
                        cw_asset::AssetInfo::native("asset"),
                    ),
                )
                .build();
            let recipient: Addr = account.into_addr();
            let coins: Vec<Coin> = coins(100u128, "asset");
            deps.querier.bank.update_balance(recipient, coins.clone());

            let bank: Bank<'_, MockModule> = app.bank(deps.as_ref(), &env);
            let res = bank.balances(&[AssetEntry::new("asset_entry")]).unwrap();
            assert_eq!(res, vec![Asset::native("asset", 100u128)]);
        }
    }

    mod transfer_coins {
        use abstract_std::account::ExecuteMsg;

        use super::*;
        use crate::{Execution, Executor, ExecutorMsg};

        #[coverage_helper::test]
        fn transfer_asset_to_sender() {
            let (deps, account, app) = mock_module_setup();
            let env = mock_env_validated(deps.api);

            // ANCHOR: transfer
            let recipient: Addr = Addr::unchecked("recipient");
            let bank: Bank<'_, MockModule> = app.bank(deps.as_ref(), &env);
            let coins: Vec<Coin> = coins(100u128, "asset");
            let bank_transfer: AccountAction = bank.transfer(coins.clone(), &recipient).unwrap();

            let executor: Executor<'_, MockModule> = app.executor(deps.as_ref());
            let account_message: ExecutorMsg = executor.execute(vec![bank_transfer]).unwrap();
            let response: Response = Response::new().add_message(account_message);
            // ANCHOR_END: transfer

            let expected_msg = CosmosMsg::Bank(BankMsg::Send {
                to_address: recipient.to_string(),
                amount: coins,
            });

            assert_eq!(
                response.messages[0].msg,
                wasm_execute(
                    account.addr(),
                    &ExecuteMsg::<Empty>::Execute {
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
        use crate::apis::respond::AbstractResponse;

        #[coverage_helper::test]
        fn deposit() {
            let (deps, account, app) = mock_module_setup();
            let env = mock_env_validated(deps.api);

            // ANCHOR: deposit
            // Get bank API struct from the app
            let bank: Bank<'_, MockModule> = app.bank(deps.as_ref(), &env);
            // Define coins to send
            let coins: Vec<Coin> = coins(100u128, "denom");
            // Construct messages for deposit (transfer from this contract to the account)
            let deposit_msgs: Vec<CosmosMsg> = bank.deposit(coins.clone()).unwrap();
            // Create response and add deposit msgs
            let response: Response = app.response("deposit").add_messages(deposit_msgs);
            // ANCHOR_END: deposit

            let bank_msg: CosmosMsg = CosmosMsg::Bank(BankMsg::Send {
                to_address: account.addr().to_string(),
                amount: coins,
            });

            assert_eq!(response.messages[0].msg, bank_msg);
        }
    }

    mod withdraw_coins {
        use super::*;

        #[coverage_helper::test]
        fn withdraw_coins() {
            let (deps, _, app) = mock_module_setup();

            let expected_amount = 100u128;
            let env = mock_env_validated(deps.api);

            let bank = app.bank(deps.as_ref(), &env);
            let coins = coins(expected_amount, "asset");
            let actual_res = bank.withdraw(&env, coins.clone());

            let expected_msg: CosmosMsg = CosmosMsg::Bank(BankMsg::Send {
                to_address: env.contract.address.to_string(),
                amount: coins,
            });

            assert_eq!(actual_res.unwrap().messages()[0], expected_msg);
        }
    }

    mod send_coins {
        use super::*;

        use cw20::Cw20ExecuteMsg;
        use cw_asset::AssetError;

        #[coverage_helper::test]
        fn send_cw20() {
            let (deps, _, app) = mock_module_setup();
            let env = mock_env_validated(deps.api);

            let expected_amount = 100u128;
            let expected_recipient = deps.api.addr_make("recipient");

            let bank = app.bank(deps.as_ref(), &env);
            let hook_msg = Empty {};
            let asset = deps.api.addr_make("asset");
            let coin = Asset::cw20(asset.clone(), expected_amount);
            let actual_res = bank.send(coin, &expected_recipient, &hook_msg);

            let expected_msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: asset.to_string(),
                msg: to_json_binary(&Cw20ExecuteMsg::Send {
                    contract: expected_recipient.to_string(),
                    amount: expected_amount.into(),
                    msg: to_json_binary(&hook_msg).unwrap(),
                })
                .unwrap(),
                funds: vec![],
            });

            assert_eq!(actual_res.unwrap().messages()[0], expected_msg);
        }

        #[coverage_helper::test]
        fn send_coins() {
            let (deps, _, app) = mock_module_setup();
            let env = mock_env_validated(deps.api);

            let expected_amount = 100u128;
            let expected_recipient = deps.api.addr_make("recipient");

            let bank = app.bank(deps.as_ref(), &env);
            let coin = coin(expected_amount, "asset");
            let hook_msg = Empty {};
            let actual_res = bank.send(coin, &expected_recipient, &hook_msg);

            assert_eq!(
                actual_res,
                Err(AbstractSdkError::Asset(
                    AssetError::UnavailableMethodForNative {
                        method: "send".into(),
                    }
                )),
            );
        }
    }

    #[coverage_helper::test]
    fn abstract_api() {
        let (deps, _, app) = mock_module_setup();
        let env = mock_env_validated(deps.api);
        let bank = app.bank(deps.as_ref(), &env);

        abstract_api_test(bank);
    }
}
