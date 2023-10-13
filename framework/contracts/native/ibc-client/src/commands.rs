use std::str::FromStr;

use crate::{
    contract::{IbcClientResponse, IbcClientResult},
    error::IbcClientError,
};
use abstract_core::{
    ibc::CallbackInfo,
    ibc_client::{
        state::{IbcInfrastructure, IBC_INFRA, REVERSE_POLYTONE_NOTE},
        IbcClientCallback,
    },
    ibc_host, manager,
    objects::{chain_name::ChainName, AccountId},
    version_control::AccountBase,
};
use abstract_sdk::{
    core::{
        ibc_client::state::{ACCOUNTS, ADMIN, CONFIG},
        ibc_host::{HostAction, InternalAction},
        objects::{ans_host::AnsHost, version_control::VersionControlContract, ChannelEntry},
        ICS20,
    },
    features::AccountIdentification,
    AccountVerification, Resolve,
};
use cosmwasm_std::{
    to_binary, wasm_execute, Coin, CosmosMsg, Deps, DepsMut, Empty, Env, IbcMsg, MessageInfo,
    QueryRequest, Storage,
};
use polytone::callbacks::CallbackRequest;

/// Packet lifetime in seconds
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
        cfg.ans_host = AnsHost {
            address: deps.api.addr_validate(&ans_host)?,
        };
    }
    if let Some(version_control) = new_version_control {
        cfg.version_control =
            VersionControlContract::new(deps.api.addr_validate(&version_control)?);
        // New version control address implies new accounts.
        clear_accounts(deps.storage);
    }

    CONFIG.save(deps.storage, &cfg)?;

    Ok(IbcClientResponse::action("update_config"))
}

/// Registers a chain to the client.
/// This registration includes the counterparty information (note and proxy address)
pub fn execute_register_infrastructure(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    host_chain: String,
    host: String,
    note: String,
) -> IbcClientResult {
    let host_chain = ChainName::from_str(&host_chain)?;
    // auth check
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    let note = deps.api.addr_validate(&note)?;
    // Can't allow if it already exists
    if IBC_INFRA.has(deps.storage, &host_chain) || REVERSE_POLYTONE_NOTE.has(deps.storage, &note) {
        return Err(IbcClientError::HostAddressExists {});
    }

    IBC_INFRA.save(
        deps.storage,
        &host_chain,
        &IbcInfrastructure {
            polytone_note: note.clone(),
            remote_abstract_host: host,
            remote_proxy: None,
        },
    )?;
    REVERSE_POLYTONE_NOTE.save(deps.storage, &note, &host_chain)?;

    // When registering a new chain host, we need to get the remote proxy address of the local note.
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
    host_chain: String,
) -> IbcClientResult {
    let host_chain = ChainName::from_str(&host_chain)?;
    // auth check
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    if let Some(ibc_infra) = IBC_INFRA.may_load(deps.storage, &host_chain)? {
        REVERSE_POLYTONE_NOTE.remove(deps.storage, &ibc_infra.polytone_note);
    }
    IBC_INFRA.remove(deps.storage, &host_chain);

    Ok(IbcClientResponse::action("remove_host"))
}

/// Send a message to a remote abstract-ibc-host. This message will be proxied through polytone.
fn send_remote_host_action(
    deps: Deps,
    account_id: AccountId,
    account: AccountBase,
    host_chain: ChainName,
    action: HostAction,
    callback_request: Option<CallbackRequest>,
) -> IbcClientResult<CosmosMsg<Empty>> {
    // Send this message via the Polytone implementation
    let ibc_infra = IBC_INFRA.load(deps.storage, &host_chain)?;
    let note_contract = ibc_infra.polytone_note;
    let remote_ibc_host = ibc_infra.remote_abstract_host;

    // message that will be called on the local note contract
    let note_message = wasm_execute(
        note_contract.to_string(),
        &polytone_note::msg::ExecuteMsg::Execute {
            msgs: vec![wasm_execute(
                // The note's remote proxy will call the ibc host
                remote_ibc_host,
                &ibc_host::ExecuteMsg::Execute {
                    /// And the ibc host will call the
                    proxy_address: account.proxy.to_string(),
                    account_id,
                    action,
                },
                vec![],
            )?
            .into()],
            callback: callback_request,
            timeout_seconds: PACKET_LIFETIME.into(),
        },
        vec![],
    )?;

    Ok(note_message.into())
}

fn send_remote_host_query(
    deps: Deps,
    _env: Env,
    host_chain: ChainName,
    queries: Vec<QueryRequest<Empty>>,
    callback_request: CallbackRequest,
) -> IbcClientResult<CosmosMsg<Empty>> {
    // Send this message via the Polytone implementation
    let note_contract = IBC_INFRA.load(deps.storage, &host_chain)?.polytone_note;

    let note_message = wasm_execute(
        note_contract.to_string(),
        &polytone_note::msg::ExecuteMsg::Query {
            msgs: queries,
            callback: callback_request,
            timeout_seconds: PACKET_LIFETIME.into(),
        },
        vec![],
    )?;

    Ok(note_message.into())
}

pub fn execute_send_packet(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    host_chain: String,
    action: HostAction,
    callback_info: Option<CallbackInfo>,
) -> IbcClientResult {
    let host_chain = ChainName::from_str(&host_chain)?;

    let cfg = CONFIG.load(deps.storage)?;

    // Verify that the sender is a proxy contract
    let account_base = cfg
        .version_control
        .account_registry(deps.as_ref())
        .assert_proxy(&info.sender)?;

    // get account_id
    let account_id = account_base.account_id(deps.as_ref())?;

    // Can only call non-internal actions
    if let HostAction::Internal(_) = action {
        return Err(IbcClientError::ForbiddenInternalCall {});
    }

    let callback_request = callback_info.map(|c| CallbackRequest {
        receiver: env.contract.address.to_string(),
        msg: to_binary(&IbcClientCallback::UserRemoteAction(c)).unwrap(),
    });

    let note_message = send_remote_host_action(
        deps.as_ref(),
        account_id,
        account_base,
        host_chain,
        action,
        callback_request,
    )?;

    Ok(IbcClientResponse::action("handle_send_msgs").add_message(note_message))
}

pub fn execute_send_query(
    deps: DepsMut,
    env: Env,
    host_chain: String,
    queries: Vec<QueryRequest<Empty>>,
    callback_info: CallbackInfo,
) -> IbcClientResult {
    let host_chain = ChainName::from_str(&host_chain)?;

    let callback_request = CallbackRequest {
        receiver: env.contract.address.to_string(),
        msg: to_binary(&IbcClientCallback::UserRemoteAction(callback_info)).unwrap(),
    };

    let note_message =
        send_remote_host_query(deps.as_ref(), env, host_chain, queries, callback_request)?;

    Ok(IbcClientResponse::action("handle_send_msgs").add_message(note_message))
}

pub fn execute_register_account(
    deps: DepsMut,
    info: MessageInfo,
    host_chain: String,
) -> IbcClientResult {
    let host_chain = ChainName::from_str(&host_chain)?;
    let cfg = CONFIG.load(deps.storage)?;

    // Verify that the sender is a proxy contract
    let account_base = cfg
        .version_control
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
    host_chain: String,
    funds: Vec<Coin>,
) -> IbcClientResult {
    let host_chain = ChainName::from_str(&host_chain)?;
    let cfg = CONFIG.load(deps.storage)?;
    let mem = cfg.ans_host;
    // Verify that the sender is a proxy contract

    let account_base = cfg
        .version_control
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
                version_control: VersionControlContract::new(Addr::unchecked(TEST_VERSION_CONTROL)),
                ans_host: AnsHost::new(Addr::unchecked(TEST_ANS_HOST)),
            };
            CONFIG.save(deps.as_mut().storage, &cfg)?;

            let new_ans_host = "new_ans_host".to_string();

            let msg = ExecuteMsg::UpdateConfig {
                ans_host: Some(new_ans_host.clone()),
                version_control: None,
            };

            let res = execute_as_admin(deps.as_mut(), msg)?;
            assert_that!(res.messages).is_empty();

            let actual = CONFIG.load(deps.as_ref().storage)?;
            assert_that!(actual.ans_host.address).is_equal_to(Addr::unchecked(new_ans_host));

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
            assert_that!(cfg.version_control.address)
                .is_equal_to(Addr::unchecked(new_version_control));

            Ok(())
        }

        #[test]
        fn update_version_control_should_clear_accounts() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            ACCOUNTS.save(
                deps.as_mut().storage,
                (&TEST_ACCOUNT_ID, &ChainName::from_str("channel")?),
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
                host_chain: "host-chain".into(),
            })
        }

        #[test]
        fn remove_existing_host() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            IBC_INFRA.save(
                deps.as_mut().storage,
                &ChainName::from_str(TEST_CHAIN)?,
                &IbcInfrastructure {
                    polytone_note: Addr::unchecked("note"),
                    remote_abstract_host: "test_remote_host".into(),
                    remote_proxy: None,
                },
            )?;

            let msg = ExecuteMsg::RemoveHost {
                host_chain: TEST_CHAIN.into(),
            };

            let res = execute_as_admin(deps.as_mut(), msg)?;
            assert_that!(res.messages).is_empty();

            assert_that!(IBC_INFRA.is_empty(&deps.storage)).is_true();

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
