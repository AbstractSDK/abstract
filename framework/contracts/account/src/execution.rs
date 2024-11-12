use abstract_sdk::{
    feature_objects::AnsHost, std::account::state::WHITELISTED_MODULES, HookMemoBuilder, Resolve,
};
use abstract_std::{
    account::state::{ACCOUNT_ID, ACCOUNT_MODULES, CALLING_TO_AS_ADMIN},
    ibc::PACKET_LIFETIME,
    ibc_client::state::IBC_INFRA,
    ibc_host,
    objects::{ownership, ChannelEntry, TruncatedChainId},
    IBC_CLIENT, ICA_CLIENT, ICS20,
};
use cosmwasm_std::{
    to_json_binary, Addr, Binary, Coin, CosmosMsg, DepsMut, Empty, Env, IbcMsg, IbcTimeout,
    MessageInfo, Response, StdError, SubMsg, WasmMsg, WasmQuery,
};

use crate::{
    contract::{
        AccountResponse, AccountResult, ADMIN_ACTION_REPLY_ID, FORWARD_RESPONSE_REPLY_ID,
        IBC_TOKEN_FLOW,
    },
    error::AccountError,
    modules::load_module_addr,
    msg::ExecuteMsg,
    reply::TokenFlowPayload,
};

/// Check that sender either whitelisted or governance
pub(crate) fn assert_whitelisted_or_owner(
    deps: &mut DepsMut,
    env: &Env,
    sender: &Addr,
) -> AccountResult<()> {
    #[cfg(feature = "xion")]
    {
        if let Some(is_admin) = crate::state::AUTH_ADMIN.may_load(deps.storage)? {
            // Clear auth if it was set
            crate::state::AUTH_ADMIN.remove(deps.storage);
            if is_admin {
                return Ok(());
            }
        }
    }
    // Account can execute on itself
    if env.contract.address == sender {
        return Ok(());
    }
    let whitelisted_modules = WHITELISTED_MODULES.load(deps.storage)?;
    if whitelisted_modules.0.contains(sender)
        || ownership::assert_nested_owner(deps.storage, &deps.querier, sender).is_ok()
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
    assert_whitelisted_or_owner(&mut deps, &env, msg_sender)?;

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
    assert_whitelisted_or_owner(&mut deps, &env, msg_sender)?;

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
    #[allow(unused_mut)] mut _auth: crate::msg::Authenticator,
) -> AccountResult {
    #[cfg(feature = "xion")]
    {
        abstract_xion::auth::execute::add_auth_method(_deps, &_env, &mut _auth).map_err(Into::into)
    }
    #[cfg(not(feature = "xion"))]
    {
        Ok(AccountResponse::action("add_auth"))
    }
}

pub fn remove_auth_method(_deps: DepsMut, _env: Env, _id: u8) -> AccountResult {
    #[cfg(feature = "xion")]
    {
        abstract_xion::auth::execute::remove_auth_method(_deps, _env, _id).map_err(Into::into)
    }
    #[cfg(not(feature = "xion"))]
    {
        Ok(AccountResponse::action("remove_auth"))
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
    assert_whitelisted_or_owner(&mut deps, &env, &msg_info.sender)?;

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

pub fn send_funds_with_actions(
    mut deps: DepsMut,
    msg_info: MessageInfo,
    env: Env,
    host_chain: TruncatedChainId,
    funds: Coin,
    actions: Vec<ExecuteMsg>,
) -> AccountResult {
    assert_whitelisted_or_owner(&mut deps, &env, &msg_info.sender)?;
    // We send a funds message to the host saying we want to deposit on a remote account
    host_chain.verify()?;

    let ans = AnsHost::new(deps.api, &env)?;

    let ics20_channel_entry = ChannelEntry {
        connected_chain: host_chain.clone(),
        protocol: ICS20.to_string(),
    };
    let ics20_channel_id = ics20_channel_entry.resolve(&deps.querier, &ans)?;

    let ibc_client_module = load_module_addr(deps.storage, IBC_CLIENT)?;
    let remote_host = IBC_INFRA
        .query(&deps.querier, ibc_client_module, &host_chain)?
        .ok_or(AccountError::ChainNotRegistered(host_chain))?;

    // Hook for sending the funds correctly to the sender
    let action_memo = HookMemoBuilder::new(
        remote_host.remote_abstract_host.clone(),
        &ibc_host::ExecuteMsg::Fund {
            src_account: ACCOUNT_ID.load(deps.storage)?,
            src_chain: TruncatedChainId::from_chain_id(&env.block.chain_id),
        },
    )
    .callback(&env)
    .build()?;

    let transfer_msg = IbcMsg::Transfer {
        channel_id: ics20_channel_id.clone(),
        to_address: remote_host.remote_abstract_host,
        amount: funds,
        timeout: IbcTimeout::with_timestamp(env.block.time.plus_seconds(PACKET_LIFETIME)),
        memo: Some(action_memo),
    };

    Ok(Response::new().add_submessage(
        SubMsg::reply_on_success(transfer_msg, IBC_TOKEN_FLOW).with_payload(to_json_binary(
            &TokenFlowPayload {
                channel_id: ics20_channel_id,
                msgs: actions,
            },
        )?),
    ))
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

    mod execute_action {

        use cosmwasm_std::{testing::MOCK_CONTRACT_ADDR, wasm_execute, CosmosMsg};

        use super::*;

        #[coverage_helper::test]
        fn only_whitelisted_can_execute() -> anyhow::Result<()> {
            let mut deps = mock_dependencies();
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
        use abstract_ica::msg::IcaActionResult;
        use abstract_std::ICA_CLIENT;
        use cosmwasm_std::{Binary, Empty};

        use crate::modules::update_module_addresses;

        use super::*;

        #[coverage_helper::test]
        fn ica_action() -> anyhow::Result<()> {
            let mut deps = mock_dependencies();
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
