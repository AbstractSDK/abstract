use crate::{
    contract::{IbcClientResponse, IbcClientResult},
    error::IbcClientError,
};
use abstract_core::{
    ibc_client::{
        state::{POLYTONE_NOTE, REMOTE_HOST, REVERSE_POLYTONE_NOTE},
        IbcClientCallback,
    },
    ibc_host, manager,
    objects::{chain_name::ChainName, AccountId}, version_control::AccountBase,
};
use abstract_sdk::{
    core::{
        ibc_client::state::{ACCOUNTS, ADMIN, ANS_HOST, CONFIG},
        ibc_host::{HostAction, InternalAction},
        objects::{ans_host::AnsHost, ChannelEntry},
        ICS20,
    },
    feature_objects::VersionControlContract,
    features::AccountIdentification,
    AccountVerification, Resolve,
};
use cosmwasm_std::{
    to_binary, wasm_execute, Coin, CosmosMsg, Deps, DepsMut, Empty, Env, IbcMsg, MessageInfo,
    Storage,
};
use polytone::callbacks::CallbackRequest;

pub const PACKET_LIFETIME: u64 = 60 * 60;

pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    new_ans_host: Option<String>,
    new_version_control: Option<String>,
) -> IbcClientResult {
    // auth check
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;
    let mut cfg = CONFIG.load(deps.storage)?;

    if let Some(ans_host) = new_ans_host {
        ANS_HOST.save(
            deps.storage,
            &AnsHost {
                address: deps.api.addr_validate(&ans_host)?,
            },
        )?;
    }
    if let Some(version_control) = new_version_control {
        cfg.version_control_address = deps.api.addr_validate(&version_control)?;
        // New version control address implies new accounts.
        clear_accounts(deps.storage);
    }

    CONFIG.save(deps.storage, &cfg)?;

    Ok(IbcClientResponse::action("update_config"))
}

pub fn execute_allow_chain_host(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    chain: ChainName,
    host: String,
    note: String,
) -> IbcClientResult {
    // auth check
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    // We need to make sure we are not over-writing anything
    // TODO

    let note = deps.api.addr_validate(&note)?;
    POLYTONE_NOTE.save(deps.storage, &chain, &note)?;
    REVERSE_POLYTONE_NOTE.save(deps.storage, &note, &chain)?;
    REMOTE_HOST.save(deps.storage, &chain, &host)?;

    // When allowing a new chain host, we need to also get the proxy address of that host.
    // We do so by calling an empty message on the polytone note. This will come back in form of a execute by callback

    let note_proxy_msg = wasm_execute(
        note,
        &polytone_note::msg::ExecuteMsg::Execute {
            msgs: vec![],
            callback: Some(CallbackRequest {
                receiver: env.contract.address.to_string(),
                msg: to_binary(&IbcClientCallback::WhoAmI {})?,
            }),
            timeout_seconds: PACKET_LIFETIME.into(),
        },
        vec![],
    )?;

    Ok(IbcClientResponse::action("allow_chain_port").add_message(note_proxy_msg))
}

// allows admins to clear host if needed
pub fn execute_remove_host(
    deps: DepsMut,
    info: MessageInfo,
    host_chain: ChainName,
) -> IbcClientResult {
    // auth check
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    if let Some(note) = POLYTONE_NOTE.may_load(deps.storage, &host_chain)? {
        REVERSE_POLYTONE_NOTE.remove(deps.storage, &note);
    }
    POLYTONE_NOTE.remove(deps.storage, &host_chain);
    REMOTE_HOST.remove(deps.storage, &host_chain);

    Ok(IbcClientResponse::action("remove_host"))
}

fn send_remote_host_action(
    deps: Deps,
    env: Env,
    account_id: AccountId,
    account: AccountBase,
    host_chain: ChainName,
    action: HostAction,
    callback_request: Option<CallbackRequest>,
) -> IbcClientResult<CosmosMsg<Empty>> {
    // Send this message via the Polytone implementation
    let note_contract = POLYTONE_NOTE.load(deps.storage, &host_chain)?;
    let remote_ibc_host = REMOTE_HOST.load(deps.storage, &host_chain)?;

    let note_message = wasm_execute(
        note_contract.to_string(),
        &polytone_note::msg::ExecuteMsg::Execute {
            msgs: vec![wasm_execute(
                remote_ibc_host,
                &ibc_host::ExecuteMsg::Execute { proxy_address: account.proxy.to_string(), account_id, action },
                vec![],
            )?
            .into()],
            callback: callback_request,
            timeout_seconds: env
                .block
                .time
                .plus_seconds(PACKET_LIFETIME)
                .seconds()
                .into(),
        },
        vec![],
    )?;

    Ok(note_message.into())
}

pub fn execute_send_packet(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    host_chain: ChainName,
    action: HostAction,
    callback_request: Option<CallbackRequest>,
) -> IbcClientResult {
    let cfg = CONFIG.load(deps.storage)?;
    let version_control = VersionControlContract::new(cfg.version_control_address);

    // Verify that the sender is a proxy contract
    let account_base = version_control
        .account_registry(deps.as_ref())
        .assert_proxy(&info.sender)?;

    // get account_id
    let account_id = account_base.account_id(deps.as_ref())?;

    // Can only call non-internal actions
    if let HostAction::Internal(_) = action {
        return Err(IbcClientError::ForbiddenInternalCall {});
    }

    let note_message = send_remote_host_action(
        deps.as_ref(),
        env,
        account_id,
        account_base,
        host_chain,
        action,
        callback_request,
    )?;

    Ok(IbcClientResponse::action("handle_send_msgs").add_message(note_message))
}

pub fn execute_register_account(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    host_chain: ChainName,
) -> IbcClientResult {
    let cfg = CONFIG.load(deps.storage)?;
    let version_control = VersionControlContract::new(cfg.version_control_address);

    // Verify that the sender is a proxy contract
    let account_base = version_control
        .account_registry(deps.as_ref())
        .assert_proxy(&info.sender)?;

    // get account_id
    let account_id = account_base.account_id(deps.as_ref())?;
    // get auxiliary information

    let account_info: manager::InfoResponse = deps
        .querier
        .query_wasm_smart(account_base.manager.clone(), &manager::QueryMsg::Info {})?;
    let account_info = account_info.info;

    let note_message = send_remote_host_action(
        deps.as_ref(),
        env,
        account_id,
        account_base,
        host_chain,
        HostAction::Internal(InternalAction::Register {
            description: account_info.description,
            link: account_info.link,
            name: account_info.name,
        }),
        None,
    )?;

    Ok(IbcClientResponse::action("handle_register").add_message(note_message))
}

pub fn execute_send_funds(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    host_chain: ChainName,
    funds: Vec<Coin>,
) -> IbcClientResult {
    let cfg = CONFIG.load(deps.storage)?;
    let mem = ANS_HOST.load(deps.storage)?;
    // Verify that the sender is a proxy contract
    let version_control = VersionControlContract::new(cfg.version_control_address);

    let account_base = version_control
        .account_registry(deps.as_ref())
        .assert_proxy(&info.sender)?;

    // get account_id of Account
    let account_id = account_base.account_id(deps.as_ref())?;
    // load remote account
    let remote_addr = ACCOUNTS.load(deps.storage, (&account_id, &host_chain))?;

    let ics20_channel_entry = ChannelEntry {
        connected_chain: host_chain,
        protocol: ICS20.to_string(),
    };
    let ics20_channel_id = ics20_channel_entry.resolve(&deps.querier, &mem)?;

    let mut transfers: Vec<CosmosMsg> = vec![];
    for amount in funds {
        // construct a packet to send

        transfers.push(
            IbcMsg::Transfer {
                channel_id: ics20_channel_id.clone(),
                to_address: remote_addr.clone(),
                amount,
                timeout: env.block.time.plus_seconds(PACKET_LIFETIME).into(),
            }
            .into(),
        );

        // TODO, do a better job and add a callback

        // use cosmos_sdk_proto::{
        //     cosmos::feegrant,
        //     ibc::{applications::transfer::v1::MsgTransfer, core::client::v1::Height},
        //     traits::{Message, TypeUrl},
        //     Any,
        // };

        // In case we need to work with the memo, we need to serialize the messages ourselves

        // let msg = //ProtoMsgTransfer{
        //     MsgTransfer{
        //     source_port: "transfer".to_string(),
        //     source_channel: ics20_channel_id.clone(),
        //     token: Some(cosmos_sdk_proto::cosmos::base::v1beta1::Coin{
        //         amount: amount.to_string(),
        //         denom: amount.denom.clone(),
        //     }),
        //     sender: env.contract.address.to_string(),
        //     receiver: remote_addr.clone(),
        //     timeout_height: Some(Height{
        //         revision_height: 0,
        //         revision_number: 0,
        //     }),
        //     timeout_timestamp: env.block.time.plus_seconds(PACKET_LIFETIME).nanos(),
        //     //memo: None,
        // };

        // let msg = CosmosMsg::Stargate {
        //     type_url: MsgTransfer::TYPE_URL.to_string(),
        //     value: msg.encode_to_vec().into(),
        // };

        // transfers.push(msg);
    }

    // // let these messages be executed by proxy
    // let proxy_msg = account_base
    //     .executor(deps.as_ref())
    //     .execute(vec![AccountAction::from_vec(transfers)])?;

    Ok(IbcClientResponse::action("handle_send_funds")
        //.add_message(proxy_msg)
        .add_messages(transfers))
}

fn clear_accounts(store: &mut dyn Storage) {
    ACCOUNTS.clear(store);
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::contract;
    use abstract_core::ibc_client::*;
    use abstract_testing::prelude::{TEST_ADMIN, TEST_ANS_HOST, TEST_VERSION_CONTROL};
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        Addr,
    };
    use speculoos::prelude::*;

    const TEST_CHAIN: &str = "test-chain";

    type IbcClientTestResult = Result<(), IbcClientError>;

    fn execute_as(deps: DepsMut, sender: &str, msg: ExecuteMsg) -> IbcClientResult {
        contract::execute(deps, mock_env(), mock_info(sender, &[]), msg)
    }

    fn execute_as_admin(deps: DepsMut, msg: ExecuteMsg) -> IbcClientResult {
        execute_as(deps, TEST_ADMIN, msg)
    }

    fn mock_init(deps: DepsMut) -> IbcClientResult {
        let msg = InstantiateMsg {
            ans_host_address: TEST_ANS_HOST.to_string(),
            version_control_address: TEST_VERSION_CONTROL.to_string(),
        };
        contract::instantiate(deps, mock_env(), mock_info(TEST_ADMIN, &[]), msg)
    }

    fn test_only_admin(msg: ExecuteMsg) -> IbcClientTestResult {
        let mut deps = mock_dependencies();
        mock_init(deps.as_mut())?;

        let res = execute_as(deps.as_mut(), "not_admin", msg);
        assert_that!(&res)
            .is_err()
            .matches(|e| matches!(e, IbcClientError::Admin { .. }));

        Ok(())
    }

    mod update_config {
        use super::*;
        use abstract_core::{ibc_client::state::Config, objects::account::TEST_ACCOUNT_ID};
        use abstract_testing::prelude::TEST_VERSION_CONTROL;

        #[test]
        fn only_admin() -> IbcClientTestResult {
            test_only_admin(ExecuteMsg::UpdateConfig {
                version_control: None,
                ans_host: None,
            })
        }

        #[test]
        fn update_ans_host() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;
            let cfg = Config {
                version_control_address: Addr::unchecked(TEST_VERSION_CONTROL),
            };
            CONFIG.save(deps.as_mut().storage, &cfg)?;

            let new_ans_host = "new_ans_host".to_string();

            let msg = ExecuteMsg::UpdateConfig {
                ans_host: Some(new_ans_host.clone()),
                version_control: None,
            };

            let res = execute_as_admin(deps.as_mut(), msg)?;
            assert_that!(res.messages).is_empty();

            let actual = ANS_HOST.load(deps.as_ref().storage)?;
            assert_that!(actual.address).is_equal_to(Addr::unchecked(new_ans_host));

            Ok(())
        }

        #[test]
        pub fn update_version_control() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let new_version_control = "new_version_control".to_string();

            let msg = ExecuteMsg::UpdateConfig {
                ans_host: None,
                version_control: Some(new_version_control.clone()),
            };

            let res = execute_as_admin(deps.as_mut(), msg)?;
            assert_that!(res.messages).is_empty();

            let cfg = CONFIG.load(deps.as_ref().storage)?;
            assert_that!(cfg.version_control_address)
                .is_equal_to(Addr::unchecked(new_version_control));

            Ok(())
        }

        #[test]
        fn update_version_control_should_clear_accounts() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            ACCOUNTS.save(
                deps.as_mut().storage,
                (&TEST_ACCOUNT_ID, &ChainName::from("channel")),
                &"Some-remote-account".to_string(),
            )?;

            let new_version_control = "new_version_control".to_string();

            let msg = ExecuteMsg::UpdateConfig {
                ans_host: None,
                version_control: Some(new_version_control),
            };

            let res = execute_as_admin(deps.as_mut(), msg)?;
            assert_that!(res.messages).is_empty();

            assert_that!(ACCOUNTS.is_empty(&deps.storage)).is_true();

            Ok(())
        }
    }

    mod remove_host {
        use super::*;

        #[test]
        fn only_admin() -> IbcClientTestResult {
            test_only_admin(ExecuteMsg::RemoveHost {
                host_chain: ChainName::from("host_chain"),
            })
        }

        #[test]
        fn remove_existing_host() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            REMOTE_HOST.save(
                deps.as_mut().storage,
                &ChainName::from(TEST_CHAIN),
                &"test_remote_host".into(),
            )?;

            let msg = ExecuteMsg::RemoveHost {
                host_chain: TEST_CHAIN.into(),
            };

            let res = execute_as_admin(deps.as_mut(), msg)?;
            assert_that!(res.messages).is_empty();

            assert_that!(REMOTE_HOST.is_empty(&deps.storage)).is_true();

            Ok(())
        }

        #[test]
        fn remove_host_nonexistent_should_not_throw() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let msg = ExecuteMsg::RemoveHost {
                host_chain: TEST_CHAIN.into(),
            };

            let res = execute_as_admin(deps.as_mut(), msg)?;
            assert_that!(res.messages).is_empty();

            Ok(())
        }
    }
}
