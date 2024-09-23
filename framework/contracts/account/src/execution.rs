use abstract_sdk::std::{
    account::state::WHITELISTED_MODULES, ibc_client::ExecuteMsg as IbcClientMsg, IBC_CLIENT,
};
use abstract_std::{account::state::ACCOUNT_MODULES, objects::ownership, ICA_CLIENT};
use cosmwasm_std::{
    wasm_execute, Addr, Binary, CosmosMsg, Deps, DepsMut, Empty, MessageInfo, StdError, SubMsg,
    WasmQuery,
};

use crate::{
    contract::{AccountResponse, AccountResult, RESPONSE_REPLY_ID},
    error::AccountError,
};

/// Check that sender either whitelisted or governance
pub(crate) fn assert_whitelisted_or_owner(deps: Deps, sender: &Addr) -> AccountResult<()> {
    let whitelisted_modules = WHITELISTED_MODULES.load(deps.storage)?;
    if whitelisted_modules.0.contains(sender)
        || ownership::assert_nested_owner(deps.storage, &deps.querier, sender).is_ok()
    {
        Ok(())
    } else {
        Err(AccountError::SenderNotWhitelistedOrOwner {})
    }
}

/// Executes `Vec<CosmosMsg>` on the proxy.
/// Permission: Module
pub fn execute_module_action(
    deps: DepsMut,
    msg_info: MessageInfo,
    msgs: Vec<CosmosMsg<Empty>>,
) -> AccountResult {
    assert_whitelisted_or_owner(deps.as_ref(), &msg_info.sender)?;

    Ok(AccountResponse::action("execute_module_action").add_messages(msgs))
}

/// Executes `CosmosMsg` on the proxy and forwards its response.
/// Permission: Module
pub fn execute_module_action_response(
    deps: DepsMut,
    msg_info: MessageInfo,
    msg: CosmosMsg<Empty>,
) -> AccountResult {
    assert_whitelisted_or_owner(deps.as_ref(), &msg_info.sender)?;

    let submsg = SubMsg::reply_on_success(msg, RESPONSE_REPLY_ID);

    Ok(AccountResponse::action("execute_module_action_response").add_submessage(submsg))
}

/// Executes IBC actions on the IBC client.
/// Permission: Module
pub fn execute_ibc_action(
    deps: DepsMut,
    msg_info: MessageInfo,
    msg: IbcClientMsg,
) -> AccountResult {
    assert_whitelisted_or_owner(deps.as_ref(), &msg_info.sender)?;

    let ibc_client_address = ACCOUNT_MODULES
        .may_load(deps.storage, IBC_CLIENT)?
        .ok_or_else(|| {
            StdError::generic_err(format!(
                "ibc_client not found on account. Add it under the {IBC_CLIENT} name."
            ))
        })?;

    let funds_to_send = if let IbcClientMsg::SendFunds { funds, .. } = &msg {
        funds.clone()
    } else {
        vec![]
    };
    let client_msg = wasm_execute(ibc_client_address, &msg, funds_to_send)?;

    Ok(AccountResponse::action("execute_ibc_action").add_message(client_msg))
}

/// Execute an action on an ICA.
/// Permission: Module
///
/// This function queries the `abstract:ica-client` contract from the account's manager.
/// It then fires a smart-query on that address of type [`QueryMsg::IcaAction`](abstract_ica::msg::QueryMsg).
///
/// The resulting `Vec<CosmosMsg>` are then executed on the proxy contract.
pub fn ica_action(deps: DepsMut, msg_info: MessageInfo, action_query: Binary) -> AccountResult {
    assert_whitelisted_or_owner(deps.as_ref(), &msg_info.sender)?;

    let ica_client_address = ACCOUNT_MODULES
        .may_load(deps.storage, ICA_CLIENT)?
        .ok_or_else(|| {
            StdError::generic_err(format!(
                "ica_client not found on account. Add it under the {ICA_CLIENT} name."
            ))
        })?;

    let res: abstract_ica::msg::IcaActionResult = deps.querier.query(
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
    use abstract_std::account::{state::*, *};
    use abstract_std::{account, IBC_CLIENT};
    use abstract_testing::prelude::*;
    use cosmwasm_std::testing::*;
    use cosmwasm_std::{coins, CosmosMsg, SubMsg};
    use speculoos::prelude::*;

    mod execute_action {

        use cosmwasm_std::{testing::MOCK_CONTRACT_ADDR, wasm_execute, CosmosMsg};

        use super::*;

        #[test]
        fn only_whitelisted_can_execute() -> anyhow::Result<()> {
            let mut deps = mock_dependencies();
            mock_init(&mut deps)?;

            let msg = ExecuteMsg::ModuleAction { msgs: vec![] };

            let info = message_info(&deps.api.addr_make("not_whitelisted"), &[]);

            let res = execute(deps.as_mut(), mock_env(), info, msg);
            assert_that(&res)
                .is_err()
                .is_equal_to(AccountError::SenderNotWhitelistedOrOwner {});
            Ok(())
        }

        #[test]
        fn forwards_action() -> anyhow::Result<()> {
            let mut deps = mock_dependencies();
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
                &ExecuteMsg::UpdateOwnership(
                    abstract_std::objects::gov_type::GovAction::RenounceOwnership,
                ),
                vec![],
            )?
            .into();

            let msg = ExecuteMsg::ModuleAction {
                msgs: vec![action.clone()],
            };

            // execute it AS the module
            let res = execute(
                deps.as_mut(),
                mock_env(),
                message_info(&module_addr, &[]),
                msg,
            );
            assert_that(&res).is_ok();

            let msgs = res?.messages;
            assert_that(&msgs).has_length(1);

            let msg = &msgs[0];
            assert_that(&msg.msg).is_equal_to(&action);

            Ok(())
        }
    }

    mod execute_ibc {
        use crate::modules::update_module_addresses;

        use super::*;

        #[test]
        fn add_module() -> anyhow::Result<()> {
            let mut deps = mock_dependencies();
            mock_init(&mut deps)?;
            let abstr = AbstractMockAddrs::new(deps.api);
            // whitelist creator
            account::state::WHITELISTED_MODULES.save(
                &mut deps.storage,
                &WhitelistedModules(vec![abstr.account.addr().clone()]),
            )?;

            let msg = ExecuteMsg::IbcAction {
                msg: abstract_std::ibc_client::ExecuteMsg::Register {
                    host_chain: "juno".parse()?,
                    namespace: None,
                    install_modules: vec![],
                },
            };

            let not_whitelisted_info = message_info(&deps.api.addr_make("not_whitelisted"), &[]);
            execute(deps.as_mut(), mock_env(), not_whitelisted_info, msg.clone()).unwrap_err();

            let manager_info = message_info(abstr.account.addr(), &[]);
            // ibc not enabled
            execute(deps.as_mut(), mock_env(), manager_info.clone(), msg.clone()).unwrap_err();
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

            let res = execute(deps.as_mut(), mock_env(), manager_info, msg)?;
            assert_that(&res.messages).has_length(1);
            assert_that!(res.messages[0]).is_equal_to(SubMsg::new(CosmosMsg::Wasm(
                cosmwasm_std::WasmMsg::Execute {
                    contract_addr: ibc_client_addr.to_string(),
                    msg: to_json_binary(&abstract_std::ibc_client::ExecuteMsg::Register {
                        host_chain: "juno".parse()?,
                        namespace: None,
                        install_modules: vec![],
                    })?,
                    funds: vec![],
                },
            )));
            Ok(())
        }

        #[test]
        fn send_funds() -> anyhow::Result<()> {
            let mut deps = mock_dependencies();
            mock_init(&mut deps)?;
            let abstr = AbstractMockAddrs::new(deps.api);
            // whitelist creator
            account::state::WHITELISTED_MODULES.save(
                &mut deps.storage,
                &WhitelistedModules(vec![abstr.account.addr().clone()]),
            )?;

            let funds = coins(10, "denom");
            let msg = ExecuteMsg::IbcAction {
                msg: abstract_std::ibc_client::ExecuteMsg::SendFunds {
                    host_chain: "juno".parse()?,
                    funds: funds.clone(),
                    memo: None,
                },
            };

            let not_whitelisted_info = message_info(&deps.api.addr_make("not_whitelisted"), &[]);
            execute(deps.as_mut(), mock_env(), not_whitelisted_info, msg.clone()).unwrap_err();

            let manager_info = message_info(abstr.account.addr(), &[]);
            // ibc not enabled
            execute(deps.as_mut(), mock_env(), manager_info.clone(), msg.clone()).unwrap_err();
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

            let res = execute(deps.as_mut(), mock_env(), manager_info, msg)?;
            assert_that(&res.messages).has_length(1);
            assert_that!(res.messages[0]).is_equal_to(SubMsg::new(CosmosMsg::Wasm(
                cosmwasm_std::WasmMsg::Execute {
                    contract_addr: ibc_client_addr.into(),
                    msg: to_json_binary(&abstract_std::ibc_client::ExecuteMsg::SendFunds {
                        host_chain: "juno".parse()?,
                        funds: funds.clone(),
                        memo: None,
                    })?,
                    funds,
                },
            )));
            Ok(())
        }
    }

    mod ica_action {
        use abstract_ica::msg::IcaActionResult;
        use abstract_std::ICA_CLIENT;
        use cosmwasm_std::{Binary, Empty};

        use crate::modules::update_module_addresses;

        use super::*;

        #[test]
        fn ica_action() -> anyhow::Result<()> {
            let mut deps = mock_dependencies();
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
            execute(deps.as_mut(), mock_env(), not_whitelisted_info, msg.clone()).unwrap_err();

            let manager_info = message_info(abstr.account.addr(), &[]);
            // ibc not enabled
            execute(deps.as_mut(), mock_env(), manager_info.clone(), msg.clone()).unwrap_err();
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

            let res = execute(deps.as_mut(), mock_env(), manager_info, msg)?;
            assert_that(&res.messages).has_length(1);
            assert_that!(res.messages[0]).is_equal_to(SubMsg::new(CosmosMsg::Custom(Empty {})));
            Ok(())
        }
    }
}