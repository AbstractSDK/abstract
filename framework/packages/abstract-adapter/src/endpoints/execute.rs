use abstract_sdk::{
    base::{ExecuteEndpoint, Handler, IbcCallbackEndpoint, ModuleIbcEndpoint},
    features::ModuleIdentification,
    AbstractResponse, AccountVerification,
};
use abstract_std::{
    account::state::ACCOUNT_MODULES,
    adapter::{AdapterBaseMsg, AdapterExecuteMsg, AdapterRequestMsg, BaseExecuteMsg, ExecuteMsg},
    objects::ownership::nested_admin::query_top_level_owner_addr,
};
use cosmwasm_std::{Addr, Deps, DepsMut, Env, MessageInfo, QuerierWrapper, Response, StdResult};
use schemars::JsonSchema;
use serde::Serialize;

use crate::{
    error::AdapterError,
    state::{AdapterContract, ContractError, MAXIMUM_AUTHORIZED_ADDRESSES},
    AdapterResult,
};

impl<
        Error: ContractError,
        CustomInitMsg,
        CustomExecMsg: Serialize + JsonSchema + AdapterExecuteMsg,
        CustomQueryMsg,
        SudoMsg,
    > ExecuteEndpoint
    for AdapterContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, SudoMsg>
{
    type ExecuteMsg = ExecuteMsg<CustomExecMsg>;

    fn execute(
        mut self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: Self::ExecuteMsg,
    ) -> Result<Response, Error> {
        match msg {
            ExecuteMsg::Module(request) => self.handle_app_msg(deps, env, info, request),
            ExecuteMsg::Base(exec_msg) => self
                .base_execute(deps, env, info, exec_msg)
                .map_err(From::from),
            ExecuteMsg::IbcCallback(msg) => self.ibc_callback(deps, env, info, msg),
            ExecuteMsg::ModuleIbc(msg) => self.module_ibc(deps, env, info, msg),
        }
    }
}

fn is_top_level_owner(querier: &QuerierWrapper, account: Addr, sender: &Addr) -> StdResult<bool> {
    let owner = query_top_level_owner_addr(querier, account)?;
    Ok(owner == sender)
}

/// The api-contract base implementation.
impl<Error: ContractError, CustomInitMsg, CustomExecMsg, CustomQueryMsg, SudoMsg>
    AdapterContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, SudoMsg>
{
    fn base_execute(
        &mut self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        message: BaseExecuteMsg,
    ) -> AdapterResult {
        let BaseExecuteMsg {
            account_address,
            msg,
        } = message;
        let account_registry = self.account_registry(deps.as_ref())?;
        let account_base = match account_address {
            // If account address provided, check if the sender is a direct or nested owner for this account.
            Some(requested_account) => {
                let account_address = deps.api.addr_validate(&requested_account)?;
                let requested_core = account_registry.assert_account(&account_address)?;
                if requested_core.addr() == info.sender
                    || is_top_level_owner(
                        &deps.querier,
                        requested_core.addr().clone(),
                        &info.sender,
                    )
                    .unwrap_or(false)
                {
                    requested_core
                } else {
                    return Err(AdapterError::UnauthorizedAdapterRequest {
                        adapter: self.module_id().to_string(),
                        sender: info.sender.to_string(),
                    });
                }
            }
            // If not provided the sender must be the direct owner
            // In that case, because this is a admin call, we need to check that the ADMIN_CALL_TO on the account is indeed this contract
            None => account_registry
                .assert_account_admin(&env, &info.sender)
                .map_err(|_| AdapterError::UnauthorizedAdapterRequest {
                    adapter: self.module_id().to_string(),
                    sender: info.sender.to_string(),
                })?,
        };
        self.target_account = Some(account_base);
        match msg {
            AdapterBaseMsg::UpdateAuthorizedAddresses { to_add, to_remove } => {
                self.update_authorized_addresses(deps, info, to_add, to_remove)
            }
        }
    }

    /// Handle a custom execution message sent to this api.
    /// Two success scenarios are possible:
    /// 1. The sender is an authorized address of the given proxy address and has provided the proxy address in the message.
    /// 2. The sender is a account of the given proxy address.
    fn handle_app_msg(
        mut self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        request: AdapterRequestMsg<CustomExecMsg>,
    ) -> Result<Response, Error> {
        let sender = &info.sender;
        let unauthorized_sender = || AdapterError::UnauthorizedAddressAdapterRequest {
            adapter: self.module_id().to_string(),
            sender: sender.to_string(),
        };

        let account_registry = self.account_registry(deps.as_ref())?;

        let account_base = match request.account_address {
            // The sender must either be an authorized address or account.
            Some(requested_account) => {
                let account_address = deps.api.addr_validate(&requested_account)?;
                let requested_core = account_registry.assert_account(&account_address)?;

                if requested_core.addr() == sender {
                    // If the caller is the account of the indicated proxy_address, it's authorized to do the operation
                    // This covers the case where the proxy field of the request is indicated where it doesn't need to be
                    requested_core
                } else {
                    // If not, we load the authorized addresses for the given proxy address.
                    let authorized = self
                        .authorized_addresses
                        .load(deps.storage, account_address)
                        .unwrap_or_default();
                    if authorized.contains(sender)
                        || is_top_level_owner(&deps.querier, requested_core.addr().clone(), sender)
                            .unwrap_or(false)
                    {
                        // If the sender is an authorized address,
                        // or top level account return the account_base.
                        requested_core
                    } else {
                        // If not, we error, this call is not permitted
                        return Err(unauthorized_sender().into());
                    }
                }
            }
            None => account_registry
                .assert_account(sender)
                .map_err(|_| unauthorized_sender())?,
        };
        self.target_account = Some(account_base);
        self.execute_handler()?(deps, env, info, self, request.request)
    }

    /// Update authorized addresses from the adapter.
    fn update_authorized_addresses(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        to_add: Vec<String>,
        to_remove: Vec<String>,
    ) -> AdapterResult {
        let account_base = self.target_account.as_ref().unwrap();
        let account = account_base.addr().clone();

        let mut authorized_addrs = self
            .authorized_addresses
            .may_load(deps.storage, account.clone())?
            .unwrap_or_default();

        // Handle the addition of authorized addresses
        for authorized in to_add {
            // authorized here can either be a contract address or a module id
            let authorized_addr = get_addr_from_module_id_or_addr(
                deps.as_ref(),
                info.sender.clone(),
                authorized.clone(),
            )?;

            if authorized_addrs.contains(&authorized_addr) {
                return Err(AdapterError::AuthorizedAddressOrModuleIdAlreadyPresent {
                    addr_or_module_id: authorized,
                });
            } else {
                authorized_addrs.push(authorized_addr);
            }
        }

        // Handling the removal of authorized addresses
        for deauthorized in to_remove {
            let deauthorized_addr = get_addr_from_module_id_or_addr(
                deps.as_ref(),
                info.sender.clone(),
                deauthorized.clone(),
            )?;
            if !authorized_addrs.contains(&deauthorized_addr) {
                return Err(AdapterError::AuthorizedAddressOrModuleIdNotPresent {
                    addr_or_module_id: deauthorized,
                });
            } else {
                authorized_addrs.retain(|addr| deauthorized_addr.ne(addr));
            }
        }

        if authorized_addrs.len() > MAXIMUM_AUTHORIZED_ADDRESSES as usize {
            return Err(AdapterError::TooManyAuthorizedAddresses {
                max: MAXIMUM_AUTHORIZED_ADDRESSES,
            });
        }

        self.authorized_addresses
            .save(deps.storage, account.clone(), &authorized_addrs)?;
        Ok(self.custom_response(
            "update_authorized_addresses",
            vec![("account", account.as_str())],
        ))
    }
}

/// This function is a helper to get a contract address from a module ir or from an address.
/// This is a temporary fix until we change or get rid of the UpdateAuthorizedAddresses API
fn get_addr_from_module_id_or_addr(
    deps: Deps,
    account: Addr,
    addr_or_module_id: String,
) -> Result<Addr, AdapterError> {
    // authorized here can either be a contract address or a module id
    if let Ok(Some(addr)) = ACCOUNT_MODULES.query(&deps.querier, account, &addr_or_module_id) {
        // In case we receive a module id
        Ok(addr)
    } else if let Ok(addr) = deps.api.addr_validate(addr_or_module_id.as_str()) {
        // In case we receive an address
        Ok(addr)
    } else {
        Err(AdapterError::AuthorizedAddressOrModuleIdNotValid { addr_or_module_id })
    }
}

#[cfg(test)]
mod tests {
    use abstract_std::adapter;
    use abstract_testing::prelude::*;
    use cosmwasm_std::{testing::*, Addr, Storage};
    use speculoos::prelude::*;

    use super::*;
    use crate::mock::{mock_init, AdapterMockResult, MockError, MockExecMsg, MOCK_ADAPTER};

    fn execute_as(
        deps: DepsMut,
        sender: &Addr,
        msg: ExecuteMsg<MockExecMsg>,
    ) -> Result<Response, MockError> {
        MOCK_ADAPTER.execute(deps, mock_env(), message_info(&sender, &[]), msg)
    }

    fn base_execute_as(
        deps: DepsMut,
        sender: &Addr,
        msg: BaseExecuteMsg,
    ) -> Result<Response, MockError> {
        execute_as(deps, sender, adapter::ExecuteMsg::Base(msg))
    }

    mod update_authorized_addresses {
        use super::*;
        use crate::mock::TEST_AUTHORIZED_ADDR;

        fn load_test_proxy_authorized_addresses(
            storage: &dyn Storage,
            proxy_addr: &Addr,
        ) -> Vec<Addr> {
            MOCK_ADAPTER
                .authorized_addresses
                .load(storage, proxy_addr.clone())
                .unwrap()
        }

        #[test]
        fn authorize_address() -> AdapterMockResult {
            let mut deps = mock_dependencies();
            deps.querier = mock_querier(deps.api);
            let base = test_account_base(deps.api);

            mock_init(&mut deps)?;

            let msg = BaseExecuteMsg {
                msg: AdapterBaseMsg::UpdateAuthorizedAddresses {
                    to_add: vec![deps.api.addr_make(TEST_AUTHORIZED_ADDR).to_string()],
                    to_remove: vec![],
                },
                account_address: None,
            };

            base_execute_as(deps.as_mut(), &base.account, msg)?;

            let api = MOCK_ADAPTER;
            assert_that!(api.authorized_addresses.is_empty(&deps.storage)).is_false();

            let test_proxy_authorized_addrs =
                load_test_proxy_authorized_addresses(&deps.storage, &base.proxy);

            assert_that!(test_proxy_authorized_addrs.len()).is_equal_to(1);
            assert_that!(test_proxy_authorized_addrs)
                .contains(deps.api.addr_make(TEST_AUTHORIZED_ADDR));
            Ok(())
        }

        #[test]
        fn revoke_address_authorization() -> AdapterMockResult {
            let mut deps = mock_dependencies();
            deps.querier = mock_querier(deps.api);
            let base = test_account_base(deps.api);

            mock_init(&mut deps)?;

            let _api = MOCK_ADAPTER;
            let msg = BaseExecuteMsg {
                account_address: None,
                msg: AdapterBaseMsg::UpdateAuthorizedAddresses {
                    to_add: vec![deps.api.addr_make(TEST_AUTHORIZED_ADDR).to_string()],
                    to_remove: vec![],
                },
            };

            base_execute_as(deps.as_mut(), &base.account, msg)?;

            let authorized_addrs = load_test_proxy_authorized_addresses(&deps.storage, &base.proxy);
            assert_that!(authorized_addrs.len()).is_equal_to(1);

            let msg = BaseExecuteMsg {
                account_address: None,
                msg: AdapterBaseMsg::UpdateAuthorizedAddresses {
                    to_add: vec![],
                    to_remove: vec![deps.api.addr_make(TEST_AUTHORIZED_ADDR).to_string()],
                },
            };

            base_execute_as(deps.as_mut(), &base.account, msg)?;
            let authorized_addrs = load_test_proxy_authorized_addresses(&deps.storage, &base.proxy);
            assert_that!(authorized_addrs.len()).is_equal_to(0);
            Ok(())
        }

        #[test]
        fn add_existing_authorized_address() -> AdapterMockResult {
            let mut deps = mock_dependencies();
            deps.querier = mock_querier(deps.api);
            let base = test_account_base(deps.api);

            mock_init(&mut deps)?;

            let _api = MOCK_ADAPTER;
            let msg = BaseExecuteMsg {
                account_address: None,
                msg: AdapterBaseMsg::UpdateAuthorizedAddresses {
                    to_add: vec![deps.api.addr_make(TEST_AUTHORIZED_ADDR).to_string()],
                    to_remove: vec![],
                },
            };

            base_execute_as(deps.as_mut(), &base.account, msg)?;

            let msg = BaseExecuteMsg {
                account_address: None,
                msg: AdapterBaseMsg::UpdateAuthorizedAddresses {
                    to_add: vec![deps.api.addr_make(TEST_AUTHORIZED_ADDR).to_string()],
                    to_remove: vec![],
                },
            };

            let res = base_execute_as(deps.as_mut(), &base.account, msg);

            assert_that!(res).is_err().matches(|e| {
                matches!(
                    e,
                    MockError::Adapter(AdapterError::AuthorizedAddressOrModuleIdAlreadyPresent {
                        addr_or_module_id: _test_authorized_address_string
                    })
                )
            });

            Ok(())
        }

        #[test]
        fn add_module_id_authorized_address() -> AdapterMockResult {
            let mut deps = mock_dependencies();
            deps.querier = mock_querier(deps.api);
            let abstr = AbstractMockAddrs::new(deps.api);

            mock_init(&mut deps)?;

            let _api = MOCK_ADAPTER;
            let msg = BaseExecuteMsg {
                account_address: None,
                msg: AdapterBaseMsg::UpdateAuthorizedAddresses {
                    to_add: vec![TEST_MODULE_ID.into()],
                    to_remove: vec![],
                },
            };

            base_execute_as(deps.as_mut(), &abstr.account.account, msg)?;

            let authorized_addrs =
                load_test_proxy_authorized_addresses(&deps.storage, &abstr.account.proxy);
            assert_that!(authorized_addrs.len()).is_equal_to(1);
            assert_that!(authorized_addrs[0].to_string())
                .is_equal_to(abstr.module_address.to_string());

            Ok(())
        }

        #[test]
        fn remove_authorized_address_dne() -> AdapterMockResult {
            let mut deps = mock_dependencies();
            deps.querier = mock_querier(deps.api);
            let base = test_account_base(deps.api);

            mock_init(&mut deps)?;

            let _api = MOCK_ADAPTER;
            let msg = BaseExecuteMsg {
                account_address: None,
                msg: AdapterBaseMsg::UpdateAuthorizedAddresses {
                    to_add: vec![],
                    to_remove: vec![deps.api.addr_make(TEST_AUTHORIZED_ADDR).into()],
                },
            };

            let res = base_execute_as(deps.as_mut(), &base.account, msg);

            assert_that!(res).is_err().matches(|e| {
                matches!(
                    e,
                    MockError::Adapter(AdapterError::AuthorizedAddressOrModuleIdNotPresent {
                        addr_or_module_id: _test_authorized_address_string
                    })
                )
            });

            Ok(())
        }
    }

    mod execute_app {
        use super::*;

        use crate::mock::TEST_AUTHORIZED_ADDR;
        use abstract_std::{
            objects::{account::AccountTrace, AccountId},
            version_control::Account,
        };
        use cosmwasm_std::OwnedDeps;

        /// This sets up the test with the following:
        /// TEST_PROXY has a single authorized address, test_authorized_address
        /// TEST_MANAGER and TEST_PROXY are the Account base
        ///
        /// Note that the querier needs to mock the Account base, as the proxy will
        /// query the Account base to get the list of authorized addresses.
        fn setup_with_authorized_addresses(
            deps: &mut OwnedDeps<MockStorage, MockApi, MockQuerier>,
            authorized: Vec<&str>,
        ) {
            mock_init(deps).unwrap();

            let msg = BaseExecuteMsg {
                account_address: None,
                msg: AdapterBaseMsg::UpdateAuthorizedAddresses {
                    to_add: authorized
                        .into_iter()
                        .map(|addr| deps.api.addr_make(addr).to_string())
                        .collect(),
                    to_remove: vec![],
                },
            };

            let base = test_account_base(deps.api);
            base_execute_as(deps.as_mut(), &base.account, msg).unwrap();
        }

        #[test]
        fn unauthorized_addresses_are_unauthorized() {
            let mut deps = mock_dependencies();
            deps.querier = AbstractMockQuerierBuilder::new(deps.api)
                .account(&test_account_base(deps.api), TEST_ACCOUNT_ID)
                .build();

            setup_with_authorized_addresses(&mut deps, vec![]);

            let msg = ExecuteMsg::Module(AdapterRequestMsg {
                account_address: None,
                request: MockExecMsg {},
            });

            let unauthorized = deps.api.addr_make("someoone");
            let res = execute_as(deps.as_mut(), &unauthorized, msg);

            assert_unauthorized(res);
        }

        fn assert_unauthorized(res: Result<Response, MockError>) {
            assert_that!(res).is_err().matches(|e| {
                matches!(
                    e,
                    MockError::Adapter(AdapterError::UnauthorizedAddressAdapterRequest {
                        sender: _unauthorized,
                        ..
                    })
                )
            });
        }

        #[test]
        fn executing_as_account_manager_is_allowed() {
            let mut deps = mock_dependencies();
            let base = test_account_base(deps.api);
            deps.querier = AbstractMockQuerierBuilder::new(deps.api)
                .account(&base, TEST_ACCOUNT_ID)
                .build();

            setup_with_authorized_addresses(&mut deps, vec![]);

            let msg = ExecuteMsg::Module(AdapterRequestMsg {
                account_address: None,
                request: MockExecMsg {},
            });

            let res = execute_as(deps.as_mut(), &base.account, msg);

            assert_that!(res).is_ok();
        }

        #[test]
        fn executing_as_authorized_address_not_allowed_without_proxy() {
            let mut deps = mock_dependencies();
            deps.querier = AbstractMockQuerierBuilder::new(deps.api)
                .account(&test_account_base(deps.api), TEST_ACCOUNT_ID)
                .build();

            setup_with_authorized_addresses(&mut deps, vec![TEST_AUTHORIZED_ADDR]);

            let msg = ExecuteMsg::Module(AdapterRequestMsg {
                account_address: None,
                request: MockExecMsg {},
            });

            let authorized = deps.api.addr_make(TEST_AUTHORIZED_ADDR);
            let res = execute_as(deps.as_mut(), &authorized, msg);

            assert_unauthorized(res);
        }

        #[test]
        fn executing_as_authorized_address_is_allowed_via_proxy() {
            let mut deps = mock_dependencies();
            let base = test_account_base(deps.api);
            deps.querier = AbstractMockQuerierBuilder::new(deps.api)
                .account(&base, TEST_ACCOUNT_ID)
                .build();

            setup_with_authorized_addresses(&mut deps, vec![TEST_AUTHORIZED_ADDR]);

            let msg = ExecuteMsg::Module(AdapterRequestMsg {
                account_address: Some(base.proxy.to_string()),
                request: MockExecMsg {},
            });

            let authorized = deps.api.addr_make(TEST_AUTHORIZED_ADDR);
            let res = execute_as(deps.as_mut(), &authorized, msg);

            assert_that!(res).is_ok();
        }

        #[test]
        fn executing_as_authorized_address_on_diff_proxy_should_err() {
            let mut deps = mock_dependencies();
            let base = test_account_base(deps.api);
            let another_base = Account::new(deps.api.addr_make("some_other_manager"));
            deps.querier = AbstractMockQuerierBuilder::new(deps.api)
                .account(&base, TEST_ACCOUNT_ID)
                .account(
                    &another_base,
                    AccountId::new(69420u32, AccountTrace::Local).unwrap(),
                )
                .build();

            setup_with_authorized_addresses(&mut deps, vec![TEST_AUTHORIZED_ADDR]);

            let msg = ExecuteMsg::Module(AdapterRequestMsg {
                account_address: Some(another_base.proxy.to_string()),
                request: MockExecMsg {},
            });

            let authorized = deps.api.addr_make(TEST_AUTHORIZED_ADDR);
            let res = execute_as(deps.as_mut(), &authorized, msg);

            assert_unauthorized(res);
        }
    }
}
