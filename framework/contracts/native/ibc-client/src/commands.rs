use crate::{
    contract::{IbcClientResponse, IbcClientResult, MAX_RETRIES},
    error::IbcClientError,
    ibc::PACKET_LIFETIME,
};
use abstract_core::{
    ibc_client::state::CHAIN_HOSTS, manager, objects::chain_name::ChainName,
    proto::ibc::ProtoMsgTransfer,
};
use abstract_sdk::AccountAction;
use abstract_sdk::{
    core::{
        ibc_client::{
            state::{ACCOUNTS, ADMIN, ANS_HOST, CHANNELS, CONFIG},
            CallbackInfo,
        },
        ibc_host::{HostAction, InternalAction, PacketMsg},
        objects::{ans_host::AnsHost, ChannelEntry},
        ICS20,
    },
    feature_objects::VersionControlContract,
    features::AccountIdentification,
    AccountVerification, Execution, Resolve,
};
use cosmos_sdk_proto::{
    cosmos::feegrant,
    ibc::{applications::transfer::v1::MsgTransfer, core::client::v1::Height},
    traits::{Message, TypeUrl},
    Any,
};
use cosmwasm_std::{
    to_binary, Binary, Coin, CosmosMsg, DepsMut, Env, IbcMsg, MessageInfo, Storage,
};

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
    info: MessageInfo,
    chain: String,
    host: String,
) -> IbcClientResult {
    // auth check
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;
    CHAIN_HOSTS.save(deps.storage, &ChainName::from(chain), &host)?;

    Ok(IbcClientResponse::action("allow_chain_port"))
}

// allows admins to clear host if needed
pub fn execute_remove_host(
    deps: DepsMut,
    info: MessageInfo,
    host_chain: ChainName,
) -> IbcClientResult {
    // auth check
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;
    CHANNELS.remove(deps.storage, &host_chain);

    Ok(IbcClientResponse::action("remove_host"))
}

pub fn execute_send_packet(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    host_chain: ChainName,
    action: HostAction,
    callback_info: Option<CallbackInfo>,
    mut retries: u8,
) -> IbcClientResult {
    let cfg = CONFIG.load(deps.storage)?;
    let version_control = VersionControlContract::new(cfg.version_control_address);

    // Verify that the sender is a proxy contract
    let account_base = version_control
        .account_registry(deps.as_ref())
        .assert_proxy(&info.sender)?;

    // Can only call non-internal actions
    if let HostAction::Internal(_) = action {
        return Err(IbcClientError::ForbiddenInternalCall {});
    }
    // Set max retries
    retries = retries.min(MAX_RETRIES);

    // get account_id
    let account_id = account_base.account_id(deps.as_ref())?;
    // ensure the channel exists and loads it.
    let channel = CHANNELS.load(deps.storage, &host_chain)?;
    let packet = PacketMsg {
        host_chain,
        retries,
        account_id,
        callback_info,
        action,
    };
    let msg = IbcMsg::SendPacket {
        channel_id: channel,
        data: to_binary(&packet)?,
        timeout: env.block.time.plus_seconds(PACKET_LIFETIME).into(),
    };

    Ok(IbcClientResponse::action("handle_send_msgs").add_message(msg))
}

pub fn execute_register_account(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    host_chain: ChainName,
) -> IbcClientResult {
    // auth check
    let cfg = CONFIG.load(deps.storage)?;
    // Verify that the sender is a proxy contract

    let version_control = VersionControlContract::new(cfg.version_control_address);

    let account_base = version_control
        .account_registry(deps.as_ref())
        .assert_proxy(&info.sender)?;

    // ensure the channel exists (not found if not registered)
    let channel_id = CHANNELS.load(deps.storage, &host_chain)?;
    let account_id = account_base.account_id(deps.as_ref())?;
    // get auxiliary information
    let account_info: manager::InfoResponse = deps
        .querier
        .query_wasm_smart(account_base.manager, &manager::QueryMsg::Info {})?;
    let account_info = account_info.info;
    // construct a packet to send
    let packet = PacketMsg {
        retries: 0u8,
        host_chain,
        account_id,
        callback_info: None,
        action: HostAction::Internal(InternalAction::Register {
            account_proxy_address: account_base.proxy.into_string(),
            description: account_info.description,
            link: account_info.link,
            name: account_info.name,
        }),
    };

    let msg = IbcMsg::SendPacket {
        channel_id,
        data: to_binary(&packet)?,
        timeout: env.block.time.plus_seconds(PACKET_LIFETIME).into(),
    };

    Ok(IbcClientResponse::action("handle_register").add_message(msg))
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
            chain: TEST_CHAIN.to_string(),
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

            CHANNELS.save(
                deps.as_mut().storage,
                &ChainName::from(TEST_CHAIN),
                &"test_channel".into(),
            )?;

            let msg = ExecuteMsg::RemoveHost {
                host_chain: TEST_CHAIN.into(),
            };

            let res = execute_as_admin(deps.as_mut(), msg)?;
            assert_that!(res.messages).is_empty();

            assert_that!(CHANNELS.is_empty(&deps.storage)).is_true();

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
