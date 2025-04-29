use abstract_sdk::std::account::state::WHITELISTED_MODULES;
use abstract_std::{
    account::state::{ACCOUNT_MODULES, CALLING_TO_AS_ADMIN},
    objects::ownership,
    ICA_CLIENT,
};
use cosmwasm_std::{
    Addr, Binary, Coin, CosmosMsg, DepsMut, Empty, Env, MessageInfo, StdError, SubMsg, WasmMsg,
    WasmQuery,
};

use crate::{
    contract::{AccountResponse, AccountResult, ADMIN_ACTION_REPLY_ID, FORWARD_RESPONSE_REPLY_ID},
    error::AccountError,
    modules::load_module_addr,
};

/// Check that sender either whitelisted or governance
pub(crate) fn assert_whitelisted_owner_or_self(
    deps: &mut DepsMut,
    env: &Env,
    sender: &Addr,
) -> AccountResult<()> {
    let whitelisted_modules = WHITELISTED_MODULES.load(deps.storage)?;
    if whitelisted_modules.0.contains(sender)
        || ownership::assert_nested_owner(deps.storage, &deps.querier, sender).is_ok()
        || sender == env.contract.address
    {
        Ok(())
    } else {
        Err(AccountError::SenderNotWhitelistedOrOwner {})
    }
}

/// Executes `Vec<CosmosMsg>` on the account.
/// Permission: Module
pub fn execute_msgs(
    mut deps: DepsMut,
    env: Env,
    msg_sender: &Addr,
    msgs: Vec<CosmosMsg<Empty>>,
) -> AccountResult {
    assert_whitelisted_owner_or_self(&mut deps, &env, msg_sender)?;

    Ok(AccountResponse::action("execute_module_action").add_messages(msgs))
}

/// Executes `CosmosMsg` on the account and forwards its response.
/// Permission: Module
pub fn execute_msgs_with_data(
    mut deps: DepsMut,
    env: Env,
    msg_sender: &Addr,
    msg: CosmosMsg<Empty>,
) -> AccountResult {
    assert_whitelisted_owner_or_self(&mut deps, &env, msg_sender)?;

    let submsg = SubMsg::reply_on_success(msg, FORWARD_RESPONSE_REPLY_ID);

    Ok(AccountResponse::action("execute_module_action_response").add_submessage(submsg))
}

/// Execute the [`exec_msg`] on the provided [`module_id`],
/// This is a simple wrapper around [`ExecuteMsg::Execute`](abstract_std::account::ExecuteMsg::Execute).
pub fn execute_on_module(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    module_id: String,
    exec_msg: Binary,
    funds: Vec<Coin>,
) -> AccountResult {
    let module_addr = load_module_addr(deps.storage, &module_id)?;
    execute_msgs(
        deps,
        env,
        &info.sender,
        vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: module_addr.into(),
            msg: exec_msg,
            funds,
        })],
    )
}

pub fn admin_execute(
    deps: DepsMut,
    info: MessageInfo,
    addr: Addr,
    exec_msg: Binary,
) -> AccountResult {
    ownership::assert_nested_owner(deps.storage, &deps.querier, &info.sender)?;

    if CALLING_TO_AS_ADMIN.exists(deps.storage) {
        return Err(AccountError::CantChainAdminCalls {});
    }
    CALLING_TO_AS_ADMIN.save(deps.storage, &addr)?;

    let msg = SubMsg::reply_on_success(
        WasmMsg::Execute {
            contract_addr: addr.to_string(),
            msg: exec_msg,
            funds: info.funds,
        },
        ADMIN_ACTION_REPLY_ID,
    );

    Ok(AccountResponse::action("admin_execute").add_submessage(msg))
}

pub fn admin_execute_on_module(
    deps: DepsMut,
    info: MessageInfo,
    module_id: String,
    exec_msg: Binary,
) -> AccountResult {
    let module_addr = load_module_addr(deps.storage, &module_id)?;
    admin_execute(deps, info, module_addr, exec_msg)
}

pub fn add_auth_method(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    #[allow(unused_mut)] mut _auth: crate::msg::Authenticator,
) -> AccountResult {
    #[cfg(feature = "xion")]
    {
        ownership::assert_nested_owner(_deps.storage, &_deps.querier, &_info.sender)?;
        abstract_xion::execute::add_auth_method(_deps, &_env, &mut _auth).map_err(Into::into)
    }
    #[cfg(not(feature = "xion"))]
    {
        Err(AccountError::NoAuthFeature {})
    }
}

pub fn remove_auth_method(_deps: DepsMut, _env: Env, _info: MessageInfo, _id: u8) -> AccountResult {
    #[cfg(feature = "xion")]
    {
        ownership::assert_nested_owner(_deps.storage, &_deps.querier, &_info.sender)?;
        abstract_xion::execute::remove_auth_method(_deps, _env, _id).map_err(Into::into)
    }
    #[cfg(not(feature = "xion"))]
    {
        Err(AccountError::NoAuthFeature {})
    }
}

/// Execute an action on an ICA.
/// Permission: Module
///
/// This function queries the `abstract:ica-client` contract from the account.
/// It then fires a smart-query on that address of type [`QueryMsg::IcaAction`](abstract_ica::msg::QueryMsg).
///
/// The resulting `Vec<CosmosMsg>` are then executed on the account contract.
pub fn ica_action(
    mut deps: DepsMut,
    env: Env,
    msg_info: MessageInfo,
    action_query: Binary,
) -> AccountResult {
    assert_whitelisted_owner_or_self(&mut deps, &env, &msg_info.sender)?;

    let ica_client_address = ACCOUNT_MODULES
        .may_load(deps.storage, ICA_CLIENT)?
        .ok_or_else(|| {
            StdError::generic_err(format!(
                "ica_client not found on account. Add it under the {ICA_CLIENT} name."
            ))
        })?;

    let res: abstract_std::ica_client::IcaActionResult = deps.querier.query(
        &WasmQuery::Smart {
            contract_addr: ica_client_address.into(),
            msg: action_query,
        }
        .into(),
    )?;

    Ok(AccountResponse::action("ica_action").add_messages(res.msgs))
}

#[cfg(test)]
mod test {
    use crate::contract::execute;
    use crate::error::AccountError;
    use crate::test_common::mock_init;
    use abstract_sdk::namespaces::OWNERSHIP_STORAGE_KEY;
    use abstract_std::account::{state::*, *};
    use abstract_std::objects::gov_type::GovernanceDetails;
    use abstract_std::objects::ownership::Ownership;
    use abstract_std::{account, IBC_CLIENT};
    use abstract_testing::prelude::*;
    use cosmwasm_std::{coins, CosmosMsg, SubMsg};
    use cosmwasm_std::{testing::*, Addr};
    use cw_storage_plus::Item;

    #[coverage_helper::test]
    fn abstract_account_can_execute_on_itself() -> anyhow::Result<()> {
        let mut deps = mock_dependencies();
        deps.querier = abstract_mock_querier(deps.api);
        mock_init(&mut deps)?;

        let env = mock_env_validated(deps.api);
        // We set the contract as owner.
        // We can't make it through execute msgs, because of XION signatures are too messy to reproduce in tests
        let ownership = Ownership {
            owner: GovernanceDetails::AbstractAccount {
                address: env.contract.address.clone(),
            }
            .verify(deps.as_ref())?,
            pending_owner: None,
            pending_expiry: None,
        };
        const OWNERSHIP: Item<Ownership<Addr>> = Item::new(OWNERSHIP_STORAGE_KEY);
        OWNERSHIP.save(deps.as_mut().storage, &ownership)?;

        // Module calls nested admin calls on account, making it admin
        let info = message_info(&env.contract.address, &[]);

        let msg = ExecuteMsg::Execute { msgs: vec![] };

        execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        Ok(())
    }

    mod execute_action {

        use cosmwasm_std::{testing::MOCK_CONTRACT_ADDR, wasm_execute, CosmosMsg};

        use super::*;

        #[cfg(feature = "xion")]
        #[coverage_helper::test]
        fn admin_actions_not_chained() -> anyhow::Result<()> {
            use crate::contract::AccountResult;
            use abstract_sdk::namespaces::OWNERSHIP_STORAGE_KEY;
            use abstract_std::objects::{gov_type::GovernanceDetails, ownership::Ownership};
            use cosmwasm_std::{Addr, Binary, DepsMut, Empty, Env, Response, WasmMsg};
            use cw_storage_plus::Item;

            fn execute_from_res(deps: DepsMut, env: Env, res: Response) -> AccountResult<Response> {
                // Execute all messages
                let info = message_info(&env.contract.address, &[]);
                if let CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: _,
                    msg,
                    funds: _,
                }) = res.messages[0].msg.clone()
                {
                    execute(deps, env.clone(), info, from_json(&msg)?).map_err(Into::into)
                } else {
                    panic!("Wrong message received");
                }
            }
            let mut deps = mock_dependencies();
            deps.querier = abstract_mock_querier(deps.api);
            mock_init(&mut deps)?;
            let env = mock_env_validated(deps.api);

            // We set the contract as owner.
            // We can't make it through execute msgs, because signatures are too messy to reproduce in tests
            let ownership = Ownership {
                owner: GovernanceDetails::AbstractAccount {
                    address: env.contract.address.clone(),
                }
                .verify(deps.as_ref())?,
                pending_owner: None,
                pending_expiry: None,
            };

            const OWNERSHIP: Item<Ownership<Addr>> = Item::new(OWNERSHIP_STORAGE_KEY);
            OWNERSHIP.save(deps.as_mut().storage, &ownership)?;

            let msg = ExecuteMsg::AdminExecute {
                addr: env.contract.address.to_string(),
                msg: to_json_binary(&ExecuteMsg::<Empty>::AdminExecute {
                    addr: env.contract.address.to_string(),
                    msg: Binary::new(vec![]),
                })?,
            };

            let info = message_info(&env.contract.address, &[]);
            let env = mock_env_validated(deps.api);

            // We simulate it's an admin call
            AUTH_ADMIN.save(deps.as_mut().storage, &true)?;
            let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
            // We simulate it's still an admin call
            AUTH_ADMIN.save(deps.as_mut().storage, &true)?;
            let res = execute_from_res(deps.as_mut(), env, res);
            assert_eq!(res, Err(AccountError::CantChainAdminCalls {}));
            Ok(())
        }

        #[coverage_helper::test]
        fn only_whitelisted_can_execute() -> anyhow::Result<()> {
            let mut deps = mock_dependencies();
            deps.querier = abstract_mock_querier(deps.api);
            mock_init(&mut deps)?;

            let msg = ExecuteMsg::Execute { msgs: vec![] };

            let info = message_info(&deps.api.addr_make("not_whitelisted"), &[]);
            let env = mock_env_validated(deps.api);

            let res = execute(deps.as_mut(), env, info, msg);
            assert_eq!(res, Err(AccountError::SenderNotWhitelistedOrOwner {}));
            Ok(())
        }

        #[coverage_helper::test]
        fn forwards_action() -> anyhow::Result<()> {
            let mut deps = mock_dependencies();
            deps.querier = abstract_mock_querier(deps.api);
            let env = mock_env_validated(deps.api);
            mock_init(&mut deps)?;

            // stub a module
            let module_addr = deps.api.addr_make(TEST_MODULE_ID);
            WHITELISTED_MODULES.save(
                &mut deps.storage,
                &WhitelistedModules(vec![module_addr.clone()]),
            )?;

            let action: CosmosMsg = wasm_execute(
                MOCK_CONTRACT_ADDR.to_string(),
                // example garbage
                &<ExecuteMsg>::UpdateOwnership(
                    abstract_std::objects::gov_type::GovAction::RenounceOwnership,
                ),
                vec![],
            )?
            .into();

            let msg = ExecuteMsg::Execute {
                msgs: vec![action.clone()],
            };

            // execute it AS the module
            let res = execute(deps.as_mut(), env, message_info(&module_addr, &[]), msg);
            assert!(res.is_ok());

            let msgs = res?.messages;
            assert_eq!(msgs.len(), 1);

            let msg = &msgs[0];
            assert_eq!(msg.msg, action);

            Ok(())
        }
    }

    mod execute_ibc {
        use crate::modules::update_module_addresses;

        use super::*;

        #[coverage_helper::test]
        fn add_module() -> anyhow::Result<()> {
            let mut deps = mock_dependencies();
            deps.querier = abstract_mock_querier(deps.api);
            let env = mock_env_validated(deps.api);
            mock_init(&mut deps)?;
            let abstr = AbstractMockAddrs::new(deps.api);
            // whitelist creator
            account::state::WHITELISTED_MODULES.save(
                &mut deps.storage,
                &WhitelistedModules(vec![abstr.account.addr().clone()]),
            )?;

            let msg = ExecuteMsg::ExecuteOnModule {
                module_id: IBC_CLIENT.to_owned(),
                exec_msg: to_json_binary(&abstract_std::ibc_client::ExecuteMsg::Register {
                    host_chain: "juno".parse()?,
                    namespace: None,
                    install_modules: vec![],
                })?,
                funds: vec![],
            };

            let not_whitelisted_info = message_info(&deps.api.addr_make("not_whitelisted"), &[]);
            execute(
                deps.as_mut(),
                env.clone(),
                not_whitelisted_info,
                msg.clone(),
            )
            .unwrap_err();

            let account_info = message_info(abstr.account.addr(), &[]);
            // ibc not enabled
            execute(
                deps.as_mut(),
                env.clone(),
                account_info.clone(),
                msg.clone(),
            )
            .unwrap_err();
            // mock enabling ibc
            let ibc_client_addr = deps.api.addr_make("ibc_client_addr");
            deps.querier = MockQuerierBuilder::default()
                .with_contract_map_entry(
                    abstr.account.addr(),
                    account::state::ACCOUNT_MODULES,
                    (IBC_CLIENT, ibc_client_addr.clone()),
                )
                .build();
            // mock enabling ibc
            update_module_addresses(
                deps.as_mut(),
                vec![(IBC_CLIENT.into(), ibc_client_addr.clone())],
                vec![],
            )?;

            let res = execute(deps.as_mut(), env, account_info, msg)?;
            assert_eq!(res.messages.len(), 1);
            assert_eq!(
                res.messages[0],
                SubMsg::new(CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
                    contract_addr: ibc_client_addr.to_string(),
                    msg: to_json_binary(&abstract_std::ibc_client::ExecuteMsg::Register {
                        host_chain: "juno".parse()?,
                        namespace: None,
                        install_modules: vec![],
                    })?,
                    funds: vec![],
                },))
            );
            Ok(())
        }

        #[coverage_helper::test]
        fn send_funds() -> anyhow::Result<()> {
            let mut deps = mock_dependencies();
            deps.querier = abstract_mock_querier(deps.api);
            let env = mock_env_validated(deps.api);
            mock_init(&mut deps)?;
            let abstr = AbstractMockAddrs::new(deps.api);
            // whitelist creator
            account::state::WHITELISTED_MODULES.save(
                &mut deps.storage,
                &WhitelistedModules(vec![abstr.account.addr().clone()]),
            )?;

            let funds = coins(10, "denom");
            let msg = ExecuteMsg::ExecuteOnModule {
                module_id: IBC_CLIENT.to_owned(),
                exec_msg: to_json_binary(&abstract_std::ibc_client::ExecuteMsg::SendFunds {
                    host_chain: "juno".parse()?,
                    memo: None,
                    receiver: None,
                })?,
                funds: funds.clone(),
            };

            let not_whitelisted_info = message_info(&deps.api.addr_make("not_whitelisted"), &[]);
            execute(
                deps.as_mut(),
                env.clone(),
                not_whitelisted_info,
                msg.clone(),
            )
            .unwrap_err();

            let account_info = message_info(abstr.account.addr(), &[]);
            // ibc not enabled
            execute(
                deps.as_mut(),
                env.clone(),
                account_info.clone(),
                msg.clone(),
            )
            .unwrap_err();
            // mock enabling ibc
            let ibc_client_addr = deps.api.addr_make("ibc_client_addr");
            deps.querier = MockQuerierBuilder::default()
                .with_contract_map_entry(
                    &abstr.account.addr().clone(),
                    account::state::ACCOUNT_MODULES,
                    (IBC_CLIENT, ibc_client_addr.clone()),
                )
                .build();
            update_module_addresses(
                deps.as_mut(),
                vec![(IBC_CLIENT.into(), ibc_client_addr.clone())],
                vec![],
            )?;

            let res = execute(deps.as_mut(), env, account_info, msg)?;
            assert_eq!(res.messages.len(), 1);
            assert_eq!(
                res.messages[0],
                SubMsg::new(CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
                    contract_addr: ibc_client_addr.into(),
                    msg: to_json_binary(&abstract_std::ibc_client::ExecuteMsg::SendFunds {
                        host_chain: "juno".parse()?,
                        memo: None,
                        receiver: None,
                    })?,
                    funds,
                },))
            );
            Ok(())
        }
    }

    mod ica_action {
        use abstract_std::ica_client::IcaActionResult;
        use abstract_std::ICA_CLIENT;
        use cosmwasm_std::{Binary, Empty};

        use crate::modules::update_module_addresses;

        use super::*;

        #[coverage_helper::test]
        fn ica_action() -> anyhow::Result<()> {
            let mut deps = mock_dependencies();
            deps.querier = abstract_mock_querier(deps.api);
            let env = mock_env_validated(deps.api);
            let abstr = AbstractMockAddrs::new(deps.api);
            let ica_client_addr = deps.api.addr_make("ica_client_addr");
            mock_init(&mut deps)?;
            // whitelist creator
            account::state::WHITELISTED_MODULES.save(
                &mut deps.storage,
                &WhitelistedModules(vec![abstr.account.addr().clone()]),
            )?;

            let action = Binary::from(b"some_action");
            let msg = ExecuteMsg::IcaAction {
                action_query_msg: action.clone(),
            };

            let not_whitelisted_info = message_info(&deps.api.addr_make("not_whitelisted"), &[]);
            execute(
                deps.as_mut(),
                env.clone(),
                not_whitelisted_info,
                msg.clone(),
            )
            .unwrap_err();

            let account_info = message_info(abstr.account.addr(), &[]);
            // ibc not enabled
            execute(
                deps.as_mut(),
                env.clone(),
                account_info.clone(),
                msg.clone(),
            )
            .unwrap_err();
            // mock enabling ibc
            update_module_addresses(
                deps.as_mut(),
                vec![(ICA_CLIENT.into(), ica_client_addr.clone())],
                vec![],
            )?;

            deps.querier = MockQuerierBuilder::default()
                .with_smart_handler(&ica_client_addr, move |bin| {
                    if bin.eq(&action) {
                        Ok(to_json_binary(&IcaActionResult {
                            msgs: vec![CosmosMsg::Custom(Empty {})],
                        })
                        .unwrap())
                    } else {
                        Err("Unexpected action query".to_owned())
                    }
                })
                .build();

            let res = execute(deps.as_mut(), env, account_info, msg)?;
            assert_eq!(res.messages.len(), 1);
            assert_eq!(res.messages[0], SubMsg::new(CosmosMsg::Custom(Empty {})));
            Ok(())
        }
    }
}
