use abstract_macros::abstract_response;
use abstract_sdk::feature_objects::VersionControlContract;
use abstract_std::{
    ibc_client::{state::*, *},
    objects::{
        ans_host::AnsHost,
        module_version::{assert_cw_contract_upgrade, migrate_module_data, set_module_data},
    },
    IBC_CLIENT,
};
use cosmwasm_std::{to_json_binary, Deps, DepsMut, Env, MessageInfo, QueryResponse, Response};
use semver::Version;

use crate::{commands, error::IbcClientError, ibc, queries};

pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub(crate) type IbcClientResult<T = Response> = Result<T, IbcClientError>;

#[abstract_response(IBC_CLIENT)]
pub(crate) struct IbcClientResponse;

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> IbcClientResult {
    cw2::set_contract_version(deps.storage, IBC_CLIENT, CONTRACT_VERSION)?;
    set_module_data(
        deps.storage,
        IBC_CLIENT,
        CONTRACT_VERSION,
        &[],
        None::<String>,
    )?;
    let cfg = Config {
        version_control: VersionControlContract::new(
            deps.api.addr_validate(&msg.version_control_address)?,
        ),
        ans_host: AnsHost {
            address: deps.api.addr_validate(&msg.ans_host_address)?,
        },
    };
    CONFIG.save(deps.storage, &cfg)?;

    cw_ownable::initialize_owner(deps.storage, deps.api, Some(info.sender.as_str()))?;
    Ok(IbcClientResponse::action("instantiate"))
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> IbcClientResult {
    match msg {
        ExecuteMsg::UpdateOwnership(action) => {
            cw_ownable::update_ownership(deps, &env.block, &info.sender, action)?;
            Ok(IbcClientResponse::action("update_ownership"))
        }
        ExecuteMsg::UpdateConfig {
            ans_host,
            version_control,
        } => commands::execute_update_config(deps, info, ans_host, version_control)
            .map_err(Into::into),
        ExecuteMsg::RemoteAction { host_chain, action } => {
            commands::execute_send_packet(deps, env, info, host_chain, action)
        }
        ExecuteMsg::RegisterInfrastructure { chain, note, host } => {
            commands::execute_register_infrastructure(deps, env, info, chain, host, note)
        }
        ExecuteMsg::SendFunds {
            host_chain,
            funds,
            memo,
        } => commands::execute_send_funds(deps, env, info, host_chain, funds, memo)
            .map_err(Into::into),
        ExecuteMsg::Register {
            host_chain,
            namespace,
            install_modules,
        } => commands::execute_register_account(
            deps,
            info,
            env,
            host_chain,
            namespace,
            install_modules,
        ),
        ExecuteMsg::RemoveHost { host_chain } => {
            commands::execute_remove_host(deps, info, host_chain).map_err(Into::into)
        }
        ExecuteMsg::Callback(c) => {
            ibc::receive_action_callback(deps, env, info, c).map_err(Into::into)
        }
        ExecuteMsg::ModuleIbcAction {
            host_chain,
            target_module,
            msg,
            callback,
        } => commands::execute_send_module_to_module_packet(
            deps,
            env,
            info,
            host_chain,
            target_module,
            msg,
            callback,
        ),
        ExecuteMsg::IbcQuery {
            host_chain,
            queries,
            callback,
        } => commands::execute_send_query(deps, env, info, host_chain, queries, callback),
    }
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> IbcClientResult<QueryResponse> {
    match msg {
        QueryMsg::Ownership {} => to_json_binary(&cw_ownable::get_ownership(deps.storage)?),
        QueryMsg::Config {} => to_json_binary(&queries::config(deps)?),
        QueryMsg::Host { chain_name } => to_json_binary(&queries::host(deps, chain_name)?),
        QueryMsg::Account {
            chain_name,
            account_id,
        } => to_json_binary(&queries::account(deps, chain_name, account_id)?),
        QueryMsg::ListAccounts { start, limit } => {
            to_json_binary(&queries::list_accounts(deps, start, limit)?)
        }
        QueryMsg::ListRemoteHosts {} => to_json_binary(&queries::list_remote_hosts(deps)?),
        QueryMsg::ListRemoteProxies {} => to_json_binary(&queries::list_remote_proxies(deps)?),
        QueryMsg::ListIbcInfrastructures {} => {
            to_json_binary(&queries::list_ibc_counterparts(deps)?)
        }
        QueryMsg::ListRemoteProxiesByAccountId { account_id } => {
            to_json_binary(&queries::list_proxies_by_account_id(deps, account_id)?)
        }
    }
    .map_err(Into::into)
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> IbcClientResult {
    let to_version: Version = CONTRACT_VERSION.parse().unwrap();

    assert_cw_contract_upgrade(deps.storage, IBC_CLIENT, to_version)?;
    cw2::set_contract_version(deps.storage, IBC_CLIENT, CONTRACT_VERSION)?;
    migrate_module_data(deps.storage, IBC_CLIENT, CONTRACT_VERSION, None::<String>)?;
    Ok(IbcClientResponse::action("migrate"))
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test_common::mock_init;
    use abstract_testing::prelude::*;
    use cosmwasm_std::{
        from_json,
        testing::{message_info, mock_dependencies, mock_env},
        Addr,
    };
    use cw2::CONTRACT;
    use cw_ownable::{Ownership, OwnershipError};
    use speculoos::prelude::*;

    type IbcClientTestResult = Result<(), IbcClientError>;

    fn execute_as(deps: DepsMut, sender: &Addr, msg: ExecuteMsg) -> IbcClientResult {
        execute(deps, mock_env(), message_info(sender, &[]), msg)
    }

    fn test_only_admin(msg: ExecuteMsg) -> IbcClientTestResult {
        let mut deps = mock_dependencies();
        mock_init(&mut deps)?;
        let not_admin = deps.api.addr_make("not_admin");

        let res = execute_as(deps.as_mut(), &not_admin, msg);
        assert_that!(&res)
            .is_err()
            .matches(|e| matches!(e, IbcClientError::Ownership(OwnershipError::NotOwner)));

        Ok(())
    }

    #[test]
    fn instantiate_works() -> IbcClientResult<()> {
        let mut deps = mock_dependencies();
        let abstr = AbstractMockAddrs::new(deps.api);
        let owner = abstr.owner;
        let msg = InstantiateMsg {
            ans_host_address: abstr.ans_host.to_string(),
            version_control_address: abstr.version_control.to_string(),
        };
        let info = message_info(&owner, &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_that!(res.messages).is_empty();

        // config
        let expected_config = Config {
            version_control: VersionControlContract::new(abstr.version_control),
            ans_host: AnsHost::new(abstr.ans_host),
        };

        let ownership_resp: Ownership<Addr> =
            from_json(query(deps.as_ref(), mock_env(), QueryMsg::Ownership {})?)?;

        assert_eq!(ownership_resp.owner, Some(owner));

        let actual_config = CONFIG.load(deps.as_ref().storage).unwrap();
        assert_that!(actual_config).is_equal_to(expected_config);

        // CW2
        let cw2_info = CONTRACT.load(&deps.storage).unwrap();
        assert_that!(cw2_info.version).is_equal_to(CONTRACT_VERSION.to_string());
        assert_that!(cw2_info.contract).is_equal_to(IBC_CLIENT.to_string());

        Ok(())
    }

    mod migrate {
        use super::*;

        use crate::contract;
        use abstract_std::AbstractError;

        #[test]
        fn disallow_same_version() -> IbcClientResult<()> {
            let mut deps = mock_dependencies();
            mock_init(&mut deps)?;

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let res = contract::migrate(deps.as_mut(), mock_env(), MigrateMsg {});

            assert_that!(res)
                .is_err()
                .is_equal_to(IbcClientError::Abstract(
                    AbstractError::CannotDowngradeContract {
                        contract: IBC_CLIENT.to_string(),
                        from: version.to_string().parse().unwrap(),
                        to: version.to_string().parse().unwrap(),
                    },
                ));

            Ok(())
        }

        #[test]
        fn disallow_downgrade() -> IbcClientResult<()> {
            let mut deps = mock_dependencies();
            mock_init(&mut deps)?;

            let big_version = "999.999.999";
            cw2::set_contract_version(deps.as_mut().storage, IBC_CLIENT, big_version)?;

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let res = contract::migrate(deps.as_mut(), mock_env(), MigrateMsg {});

            assert_that!(res)
                .is_err()
                .is_equal_to(IbcClientError::Abstract(
                    AbstractError::CannotDowngradeContract {
                        contract: IBC_CLIENT.to_string(),
                        from: big_version.parse().unwrap(),
                        to: version.to_string().parse().unwrap(),
                    },
                ));

            Ok(())
        }

        #[test]
        fn disallow_name_change() -> IbcClientResult<()> {
            let mut deps = mock_dependencies();
            mock_init(&mut deps)?;

            let old_version = "0.0.0";
            let old_name = "old:contract";
            cw2::set_contract_version(deps.as_mut().storage, old_name, old_version)?;

            let res = contract::migrate(deps.as_mut(), mock_env(), MigrateMsg {});

            assert_that!(res)
                .is_err()
                .is_equal_to(IbcClientError::Abstract(
                    AbstractError::ContractNameMismatch {
                        from: old_name.parse().unwrap(),
                        to: IBC_CLIENT.parse().unwrap(),
                    },
                ));

            Ok(())
        }

        #[test]
        fn works() -> IbcClientResult<()> {
            let mut deps = mock_dependencies();
            mock_init(&mut deps)?;

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let small_version = Version {
                minor: version.minor - 1,
                ..version.clone()
            }
            .to_string();
            cw2::set_contract_version(deps.as_mut().storage, IBC_CLIENT, small_version)?;

            let res = contract::migrate(deps.as_mut(), mock_env(), MigrateMsg {})?;
            assert_that!(res.messages).has_length(0);

            assert_that!(cw2::get_contract_version(&deps.storage)?.version)
                .is_equal_to(version.to_string());
            Ok(())
        }
    }

    mod register_infrastructure {
        use std::str::FromStr;

        use abstract_std::{ibc::polytone_callbacks::CallbackRequest, objects::TruncatedChainId};
        use cosmwasm_std::wasm_execute;

        use super::*;
        use crate::commands::PACKET_LIFETIME;

        #[test]
        fn only_admin() -> IbcClientResult<()> {
            test_only_admin(ExecuteMsg::RegisterInfrastructure {
                chain: "host-chain".parse().unwrap(),
                note: String::from("note"),
                host: String::from("host"),
            })
        }

        #[test]
        fn cannot_register_if_already_exists() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            mock_init(&mut deps)?;
            let abstr = AbstractMockAddrs::new(deps.api);
            let note = "note";
            let note_addr = deps.api.addr_make(note);

            IBC_INFRA.save(
                deps.as_mut().storage,
                &TruncatedChainId::from_str(TEST_CHAIN)?,
                &IbcInfrastructure {
                    polytone_note: note_addr.clone(),
                    remote_abstract_host: "test_remote_host".into(),
                    remote_proxy: None,
                },
            )?;

            let msg = ExecuteMsg::RegisterInfrastructure {
                chain: TEST_CHAIN.parse().unwrap(),
                note: note_addr.to_string(),
                host: String::from("test_remote_host"),
            };

            let res = execute_as(deps.as_mut(), &abstr.owner, msg);
            assert_that!(&res)
                .is_err()
                .matches(|e| matches!(e, IbcClientError::HostAddressExists {}));

            Ok(())
        }

        #[test]
        fn register_infrastructure() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            mock_init(&mut deps)?;
            let abstr = AbstractMockAddrs::new(deps.api);

            let chain_name = TruncatedChainId::from_str(TEST_CHAIN)?;
            let note = "note";
            let note_addr = deps.api.addr_make(note);
            let host = String::from("test_remote_host");

            let msg = ExecuteMsg::RegisterInfrastructure {
                chain: chain_name.clone(),
                note: note_addr.to_string(),
                host: host.clone(),
            };

            let note_proxy_msg = wasm_execute(
                note_addr.to_string(),
                &PolytoneNoteExecuteMsg::Execute {
                    msgs: vec![],
                    callback: Some(CallbackRequest {
                        receiver: mock_env().contract.address.to_string(),
                        msg: to_json_binary(&IbcClientCallback::WhoAmI {})?,
                    }),
                    timeout_seconds: PACKET_LIFETIME.into(),
                },
                vec![],
            )?;

            let res = execute_as(deps.as_mut(), &abstr.owner, msg)?;

            assert_eq!(
                IbcClientResponse::action("allow_chain_port").add_message(note_proxy_msg),
                res
            );

            // Verify IBC_INFRA
            let ibc_infra = IBC_INFRA.load(deps.as_ref().storage, &chain_name)?;
            let expected_ibc_infra = IbcInfrastructure {
                polytone_note: note_addr.clone(),
                remote_abstract_host: host.clone(),
                remote_proxy: None,
            };

            assert_eq!(expected_ibc_infra, ibc_infra);

            // Verify REVERSE_POLYTONE_NOTE
            let reverse_note = REVERSE_POLYTONE_NOTE.load(deps.as_ref().storage, &note_addr)?;

            assert_eq!(chain_name, reverse_note);

            // Verify queries
            let host_response: HostResponse = from_json(query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::Host {
                    chain_name: chain_name.clone(),
                },
            )?)?;
            assert_eq!(
                HostResponse {
                    remote_host: host.clone(),
                    remote_polytone_proxy: None
                },
                host_response
            );

            let remote_hosts_response: ListRemoteHostsResponse = from_json(query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::ListRemoteHosts {},
            )?)?;
            let hosts = remote_hosts_response.hosts;
            assert_eq!(vec![(chain_name.clone(), host)], hosts);

            let remote_proxies_response: ListRemoteProxiesResponse = from_json(query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::ListRemoteProxies {},
            )?)?;
            let hosts = remote_proxies_response.proxies;
            assert_eq!(vec![(chain_name.clone(), None)], hosts);

            let ibc_infratructures_response: ListIbcInfrastructureResponse = from_json(query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::ListIbcInfrastructures {},
            )?)?;
            let hosts = ibc_infratructures_response.counterparts;
            assert_eq!(vec![(chain_name, expected_ibc_infra)], hosts);

            Ok(())
        }
    }

    mod remote_action {
        use super::*;
        use std::str::FromStr;

        use abstract_std::{
            account,
            ibc_host::{self, HostAction, InternalAction},
            objects::{version_control::VersionControlError, TruncatedChainId},
        };

        use cosmwasm_std::wasm_execute;

        use crate::commands::PACKET_LIFETIME;

        #[test]
        fn throw_when_sender_is_not_proxy() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            let base = test_account_base(deps.api);
            deps.querier = AbstractMockQuerierBuilder::new(deps.api)
                .account(&base, TEST_ACCOUNT_ID)
                .build();
            mock_init(&mut deps)?;

            let chain_name = TruncatedChainId::from_str(TEST_CHAIN)?;

            let msg = ExecuteMsg::RemoteAction {
                host_chain: chain_name,
                action: HostAction::Dispatch {
                    manager_msgs: vec![account::ExecuteMsg::UpdateInfo {
                        name: None,
                        description: None,
                        link: None,
                    }],
                },
            };

            let res = execute_as(deps.as_mut(), &base.manager, msg);

            assert_that!(res).is_err().matches(|e| {
                matches!(
                    e,
                    IbcClientError::VersionControlError(VersionControlError::NotProxy(..))
                )
            });
            Ok(())
        }

        #[test]
        fn cannot_make_internal_call() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            let base = test_account_base(deps.api);
            deps.querier = AbstractMockQuerierBuilder::new(deps.api)
                .account(&base, TEST_ACCOUNT_ID)
                .build();
            mock_init(&mut deps)?;

            let chain_name = TruncatedChainId::from_str(TEST_CHAIN)?;

            let msg = ExecuteMsg::RemoteAction {
                host_chain: chain_name,
                action: HostAction::Internal(InternalAction::Register {
                    name: String::from("name"),
                    description: None,
                    link: None,
                    namespace: None,
                    install_modules: vec![],
                }),
            };

            let res = execute_as(deps.as_mut(), &base.proxy, msg);

            assert_that!(res)
                .is_err()
                .matches(|e| matches!(e, IbcClientError::ForbiddenInternalCall {}));
            Ok(())
        }

        #[test]
        fn send_packet_with_no_callback() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            let base = test_account_base(deps.api);
            deps.querier = AbstractMockQuerierBuilder::new(deps.api)
                .account(&base, TEST_ACCOUNT_ID)
                .build();
            mock_init(&mut deps)?;

            let chain_name = TruncatedChainId::from_str(TEST_CHAIN)?;
            let note = "note";
            let note_addr = deps.api.addr_make(note);
            let remote_ibc_host = String::from("test_remote_host");

            IBC_INFRA.save(
                deps.as_mut().storage,
                &chain_name,
                &IbcInfrastructure {
                    polytone_note: note_addr.clone(),
                    remote_abstract_host: remote_ibc_host.clone(),
                    remote_proxy: None,
                },
            )?;

            let action = HostAction::Dispatch {
                manager_msgs: vec![manager::ExecuteMsg::UpdateInfo {
                    name: None,
                    description: None,
                    link: None,
                }],
            };

            let msg = ExecuteMsg::RemoteAction {
                host_chain: chain_name,
                action: action.clone(),
            };

            let res = execute_as(deps.as_mut(), &base.proxy, msg)?;

            let note_message = wasm_execute(
                note_addr.to_string(),
                &PolytoneNoteExecuteMsg::Execute {
                    msgs: vec![wasm_execute(
                        // The note's remote proxy will call the ibc host
                        remote_ibc_host,
                        &ibc_host::ExecuteMsg::Execute {
                            account_address: base.proxy.to_string(),
                            account_id: TEST_ACCOUNT_ID,
                            action,
                        },
                        vec![],
                    )?
                    .into()],
                    callback: None,
                    timeout_seconds: PACKET_LIFETIME.into(),
                },
                vec![],
            )?;

            assert_eq!(
                IbcClientResponse::action("handle_send_msgs").add_message(note_message),
                res
            );
            Ok(())
        }
    }

    mod send_funds {
        use super::*;

        use crate::commands::PACKET_LIFETIME;
        use abstract_std::{
            objects::{version_control::VersionControlError, ChannelEntry, TruncatedChainId},
            ICS20,
        };
        use cosmwasm_std::{coins, AnyMsg, Binary, CosmosMsg, IbcMsg};
        use prost::Name;
        use std::str::FromStr;

        #[test]
        fn throw_when_sender_is_not_proxy() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            let base = test_account_base(deps.api);
            deps.querier = AbstractMockQuerierBuilder::new(deps.api)
                .account(&base, TEST_ACCOUNT_ID)
                .build();
            mock_init(&mut deps)?;

            let chain_name = TruncatedChainId::from_str(TEST_CHAIN)?;

            let msg = ExecuteMsg::SendFunds {
                host_chain: chain_name,
                funds: coins(1, "denom"),
                memo: None,
            };

            let res = execute_as(deps.as_mut(), &base.manager, msg);

            assert_that!(res).is_err().matches(|e| {
                matches!(
                    e,
                    IbcClientError::VersionControlError(VersionControlError::NotProxy(..))
                )
            });
            Ok(())
        }

        #[test]
        fn works() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            let chain_name = TruncatedChainId::from_str(TEST_CHAIN)?;
            let channel_entry = ChannelEntry {
                connected_chain: chain_name.clone(),
                protocol: String::from(ICS20),
            };
            let channel_id = String::from("1");
            let channels: Vec<(&ChannelEntry, String)> = vec![(&channel_entry, channel_id.clone())];
            let base = test_account_base(deps.api);
            deps.querier = AbstractMockQuerierBuilder::new(deps.api)
                .account(&base, TEST_ACCOUNT_ID)
                .channels(channels)
                .build();
            mock_init(&mut deps)?;

            let remote_addr = String::from("remote_addr");

            ACCOUNTS.save(
                deps.as_mut().storage,
                (TEST_ACCOUNT_ID.trace(), TEST_ACCOUNT_ID.seq(), &chain_name),
                &remote_addr,
            )?;

            let funds = coins(1, "denom");

            let msg = ExecuteMsg::SendFunds {
                host_chain: chain_name.clone(),
                funds: funds.clone(),
                memo: None,
            };

            let res = execute_as(deps.as_mut(), &base.proxy, msg)?;

            let transfer_msgs: Vec<CosmosMsg> = funds
                .into_iter()
                .map(|amount| {
                    IbcMsg::Transfer {
                        channel_id: channel_id.clone(),
                        to_address: remote_addr.clone(),
                        amount,
                        timeout: mock_env().block.time.plus_seconds(PACKET_LIFETIME).into(),
                        memo: None,
                    }
                    .into()
                })
                .collect();

            assert_eq!(
                IbcClientResponse::action("handle_send_funds").add_messages(transfer_msgs),
                res
            );

            let funds = coins(1, "denom");
            let memo = Some("some_memo".to_owned());

            let msg = ExecuteMsg::SendFunds {
                host_chain: chain_name,
                funds: funds.clone(),
                memo: memo.clone(),
            };

            let res = execute_as(deps.as_mut(), &base.proxy, msg)?;

            use prost::Message;
            let transfer_msgs: Vec<CosmosMsg> = funds
                .into_iter()
                .map(|c| {
                    CosmosMsg::Any(AnyMsg {
                        type_url: ibc_proto::ibc::apps::transfer::v1::MsgTransfer::type_url(),
                        value: Binary::from(
                            ibc_proto::ibc::apps::transfer::v1::MsgTransfer {
                                source_port: "transfer".to_owned(),
                                source_channel: channel_id.clone(),
                                token: Some(ibc_proto::cosmos::base::v1beta1::Coin {
                                    denom: c.denom,
                                    amount: c.amount.to_string(),
                                }),
                                sender: mock_env().contract.address.to_string(),
                                receiver: remote_addr.clone(),
                                timeout_height: None,
                                timeout_timestamp: mock_env()
                                    .block
                                    .time
                                    .plus_seconds(PACKET_LIFETIME)
                                    .nanos(),
                                memo: memo.clone().unwrap(),
                            }
                            .encode_to_vec(),
                        ),
                    })
                })
                .collect();

            assert_eq!(
                IbcClientResponse::action("handle_send_funds").add_messages(transfer_msgs),
                res
            );

            Ok(())
        }
    }

    mod register_account {
        use super::*;

        use crate::commands::PACKET_LIFETIME;
        use abstract_std::{
            ibc::polytone_callbacks::CallbackRequest,
            ibc_host::{self, HostAction, InternalAction},
            manager,
            objects::{version_control::VersionControlError, TruncatedChainId},
        };
        use cosmwasm_std::wasm_execute;
        use std::str::FromStr;

        #[test]
        fn throw_when_sender_is_not_proxy() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            let base = test_account_base(deps.api);
            deps.querier = AbstractMockQuerierBuilder::new(deps.api)
                .account(&base, TEST_ACCOUNT_ID)
                .build();
            mock_init(&mut deps)?;

            let chain_name = TruncatedChainId::from_str(TEST_CHAIN)?;

            let msg = ExecuteMsg::Register {
                host_chain: chain_name,
                namespace: None,
                install_modules: vec![],
            };

            let res = execute_as(deps.as_mut(), &base.manager, msg);

            assert_that!(res).is_err().matches(|e| {
                matches!(
                    e,
                    IbcClientError::VersionControlError(VersionControlError::NotProxy(..))
                )
            });
            Ok(())
        }

        #[test]
        fn works() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            let base = test_account_base(deps.api);
            deps.querier = AbstractMockQuerierBuilder::new(deps.api)
                .account(&base, TEST_ACCOUNT_ID)
                .builder()
                .with_smart_handler(&base.manager, |msg| {
                    match from_json::<manager::QueryMsg>(msg).unwrap() {
                        manager::QueryMsg::Info {} => to_json_binary(&manager::InfoResponse {
                            info: manager::state::AccountInfo {
                                name: String::from("name"),
                                chain_id: String::from("chain-id"),
                                description: None,
                                link: None,
                            },
                        })
                        .map_err(|e| e.to_string()),
                        _ => todo!(),
                    }
                })
                .build();
            mock_init(&mut deps)?;

            let chain_name = TruncatedChainId::from_str(TEST_CHAIN)?;
            let note_contract = deps.api.addr_make("note");
            let remote_ibc_host = String::from("test_remote_host");

            IBC_INFRA.save(
                deps.as_mut().storage,
                &chain_name,
                &IbcInfrastructure {
                    polytone_note: note_contract.clone(),
                    remote_abstract_host: remote_ibc_host.clone(),
                    remote_proxy: None,
                },
            )?;

            let msg = ExecuteMsg::Register {
                host_chain: chain_name,
                namespace: None,
                install_modules: vec![],
            };

            let res = execute_as(deps.as_mut(), &base.proxy, msg)?;

            let note_message = wasm_execute(
                note_contract.to_string(),
                &PolytoneNoteExecuteMsg::Execute {
                    msgs: vec![wasm_execute(
                        // The note's remote proxy will call the ibc host
                        remote_ibc_host,
                        &ibc_host::ExecuteMsg::Execute {
                            account_address: base.proxy.to_string(),
                            account_id: TEST_ACCOUNT_ID,
                            action: HostAction::Internal(InternalAction::Register {
                                description: None,
                                link: None,
                                name: String::from("name"),
                                namespace: None,
                                install_modules: vec![],
                            }),
                        },
                        vec![],
                    )?
                    .into()],
                    callback: Some(CallbackRequest {
                        receiver: mock_env().contract.address.to_string(),
                        msg: to_json_binary(&IbcClientCallback::CreateAccount {
                            account_id: TEST_ACCOUNT_ID,
                        })?,
                    }),
                    timeout_seconds: PACKET_LIFETIME.into(),
                },
                vec![],
            )?;

            assert_eq!(
                IbcClientResponse::action("handle_register").add_message(note_message),
                res
            );

            Ok(())
        }
    }

    mod update_config {
        use std::str::FromStr;

        use abstract_std::objects::TruncatedChainId;

        use super::*;

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
            mock_init(&mut deps)?;
            let abstr = AbstractMockAddrs::new(deps.api);

            let cfg = Config {
                version_control: VersionControlContract::new(abstr.version_control),
                ans_host: AnsHost::new(abstr.ans_host),
            };
            CONFIG.save(deps.as_mut().storage, &cfg)?;

            let new_ans_host = deps.api.addr_make("new_ans_host");

            let msg = ExecuteMsg::UpdateConfig {
                ans_host: Some(new_ans_host.to_string()),
                version_control: None,
            };

            let res = execute_as(deps.as_mut(), &abstr.owner, msg)?;
            assert_that!(res.messages).is_empty();

            let actual = CONFIG.load(deps.as_ref().storage)?;
            assert_that!(actual.ans_host.address).is_equal_to(new_ans_host);

            Ok(())
        }

        #[test]
        pub fn update_version_control() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            mock_init(&mut deps)?;
            let abstr = AbstractMockAddrs::new(deps.api);

            let new_version_control = deps.api.addr_make("new_version_control");

            let msg = ExecuteMsg::UpdateConfig {
                ans_host: None,
                version_control: Some(new_version_control.to_string()),
            };

            let res = execute_as(deps.as_mut(), &abstr.owner, msg)?;
            assert_that!(res.messages).is_empty();

            let cfg = CONFIG.load(deps.as_ref().storage)?;
            assert_that!(cfg.version_control.address).is_equal_to(new_version_control);

            Ok(())
        }

        #[test]
        fn update_version_control_should_clear_accounts() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            mock_init(&mut deps)?;
            let abstr = AbstractMockAddrs::new(deps.api);

            ACCOUNTS.save(
                deps.as_mut().storage,
                (
                    TEST_ACCOUNT_ID.trace(),
                    TEST_ACCOUNT_ID.seq(),
                    &TruncatedChainId::from_str("channel")?,
                ),
                &"some-remote-account".to_string(),
            )?;

            let new_version_control = deps.api.addr_make("new_version_control").to_string();

            let msg = ExecuteMsg::UpdateConfig {
                ans_host: None,
                version_control: Some(new_version_control),
            };

            let res = execute_as(deps.as_mut(), &abstr.owner, msg)?;
            assert_that!(res.messages).is_empty();

            assert_that!(ACCOUNTS.is_empty(&deps.storage)).is_true();

            Ok(())
        }
    }

    mod remove_host {
        use std::str::FromStr;

        use abstract_std::objects::TruncatedChainId;

        use super::*;

        #[test]
        fn only_admin() -> IbcClientTestResult {
            test_only_admin(ExecuteMsg::RemoveHost {
                host_chain: "host-chain".parse().unwrap(),
            })
        }

        #[test]
        fn remove_existing_host() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            mock_init(&mut deps)?;
            let note = deps.api.addr_make("note");
            let abstr = AbstractMockAddrs::new(deps.api);

            IBC_INFRA.save(
                deps.as_mut().storage,
                &TruncatedChainId::from_str(TEST_CHAIN)?,
                &IbcInfrastructure {
                    polytone_note: note,
                    remote_abstract_host: "test_remote_host".into(),
                    remote_proxy: None,
                },
            )?;

            let msg = ExecuteMsg::RemoveHost {
                host_chain: TEST_CHAIN.parse().unwrap(),
            };

            let res = execute_as(deps.as_mut(), &abstr.owner, msg)?;
            assert_that!(res.messages).is_empty();

            assert_that!(IBC_INFRA.is_empty(&deps.storage)).is_true();

            Ok(())
        }

        #[test]
        fn remove_host_nonexistent_should_not_throw() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            mock_init(&mut deps)?;
            let abstr = AbstractMockAddrs::new(deps.api);

            let msg = ExecuteMsg::RemoveHost {
                host_chain: TEST_CHAIN.parse().unwrap(),
            };

            let res = execute_as(deps.as_mut(), &abstr.owner, msg)?;
            assert_that!(res.messages).is_empty();

            Ok(())
        }
    }

    mod callback {
        use std::str::FromStr;

        use abstract_std::{
            ibc::polytone_callbacks::{Callback, CallbackMessage, ExecutionResponse},
            objects::{account::TEST_ACCOUNT_ID, TruncatedChainId},
        };
        use cosmwasm_std::{from_json, Binary, Event, SubMsgResponse};

        use super::*;

        #[test]
        fn invalid_initiator() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            mock_init(&mut deps)?;

            let note_addr = deps.api.addr_make("note");
            let chain_name = TruncatedChainId::from_str(TEST_CHAIN)?;
            REVERSE_POLYTONE_NOTE.save(deps.as_mut().storage, &note_addr, &chain_name)?;

            let msg = ExecuteMsg::Callback(CallbackMessage {
                initiator: Addr::unchecked("invalid_initiator"),
                initiator_msg: Binary::default(),
                result: Callback::Execute(Ok(ExecutionResponse {
                    executed_by: String::from("addr"),
                    result: vec![],
                })),
            });

            let res = execute_as(deps.as_mut(), &note_addr, msg);

            assert_that!(&res)
                .is_err()
                .matches(|e| matches!(e, IbcClientError::Unauthorized { .. }));

            Ok(())
        }

        #[test]
        fn caller_not_note() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            mock_init(&mut deps)?;

            let note_addr = deps.api.addr_make("note");
            let chain_name = TruncatedChainId::from_str(TEST_CHAIN)?;
            REVERSE_POLYTONE_NOTE.save(deps.as_mut().storage, &note_addr, &chain_name)?;

            let msg = ExecuteMsg::Callback(CallbackMessage {
                initiator: Addr::unchecked("invalid_initiator"),
                initiator_msg: Binary::default(),
                result: Callback::Execute(Ok(ExecutionResponse {
                    executed_by: String::from("addr"),
                    result: vec![],
                })),
            });

            let not_note = deps.api.addr_make("not_note");
            let res = execute_as(deps.as_mut(), &not_note, msg);

            assert_that!(&res)
                .is_err()
                .matches(|e| matches!(e, IbcClientError::Unauthorized { .. }));

            Ok(())
        }

        #[test]
        fn who_am_i_unregistered_chain() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            mock_init(&mut deps)?;
            let env = mock_env();

            let note_addr = deps.api.addr_make("note");
            let chain_name = TruncatedChainId::from_str(TEST_CHAIN)?;
            REVERSE_POLYTONE_NOTE.save(deps.as_mut().storage, &note_addr, &chain_name)?;

            let msg = ExecuteMsg::Callback(CallbackMessage {
                initiator: env.contract.address,
                initiator_msg: to_json_binary(&IbcClientCallback::WhoAmI {})?,
                result: Callback::Execute(Ok(ExecutionResponse {
                    executed_by: String::from("addr"),
                    result: vec![],
                })),
            });

            let res = execute_as(deps.as_mut(), &note_addr, msg);

            assert_that!(&res)
                .is_err()
                .matches(|e| matches!(e, IbcClientError::UnregisteredChain { .. }));

            Ok(())
        }

        #[test]
        fn who_am_i_fatal_error() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            mock_init(&mut deps)?;
            let env = mock_env();

            let chain_name = TruncatedChainId::from_str(TEST_CHAIN)?;
            let note_addr = deps.api.addr_make("note");
            let remote_ibc_host = String::from("test_remote_host");

            IBC_INFRA.save(
                deps.as_mut().storage,
                &chain_name,
                &IbcInfrastructure {
                    polytone_note: note_addr.clone(),
                    remote_abstract_host: remote_ibc_host.clone(),
                    remote_proxy: None,
                },
            )?;
            REVERSE_POLYTONE_NOTE.save(deps.as_mut().storage, &note_addr, &chain_name)?;
            let callback_msg = CallbackMessage {
                initiator: env.contract.address,
                initiator_msg: to_json_binary(&IbcClientCallback::WhoAmI {})?,
                result: Callback::FatalError(String::from("error")),
            };

            let msg = ExecuteMsg::Callback(callback_msg.clone());

            let res = execute_as(deps.as_mut(), &note_addr, msg);

            assert_that!(&res)
                .is_err()
                .matches(|e| matches!(e, IbcClientError::IbcFailed(_callback_msg)));

            Ok(())
        }

        #[test]
        fn who_am_i_success() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            mock_init(&mut deps)?;
            let env = mock_env();

            let chain_name = TruncatedChainId::from_str(TEST_CHAIN)?;
            let note_addr = deps.api.addr_make("note");
            let remote_ibc_host = String::from("test_remote_host");

            IBC_INFRA.save(
                deps.as_mut().storage,
                &chain_name,
                &IbcInfrastructure {
                    polytone_note: note_addr.clone(),
                    remote_abstract_host: remote_ibc_host.clone(),
                    remote_proxy: None,
                },
            )?;
            REVERSE_POLYTONE_NOTE.save(deps.as_mut().storage, &note_addr, &chain_name)?;

            let remote_proxy = String::from("remote_proxy");

            let msg = ExecuteMsg::Callback(CallbackMessage {
                initiator: env.contract.address,
                initiator_msg: to_json_binary(&IbcClientCallback::WhoAmI {})?,
                result: Callback::Execute(Ok(ExecutionResponse {
                    executed_by: remote_proxy.clone(),
                    result: vec![],
                })),
            });

            let res = execute_as(deps.as_mut(), &note_addr, msg)?;

            assert_eq!(
                IbcClientResponse::action("register_remote_proxy")
                    .add_attribute("chain", chain_name.to_string()),
                res
            );

            let updated_ibc_infra = IBC_INFRA.load(deps.as_ref().storage, &chain_name)?;

            assert_eq!(
                IbcInfrastructure {
                    polytone_note: note_addr.clone(),
                    remote_abstract_host: remote_ibc_host.clone(),
                    remote_proxy: Some(remote_proxy),
                },
                updated_ibc_infra
            );

            Ok(())
        }

        #[test]
        fn create_account_fatal_error() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            mock_init(&mut deps)?;
            let env = mock_env();

            let chain_name = TruncatedChainId::from_str(TEST_CHAIN)?;
            let note_addr = deps.api.addr_make("note");

            REVERSE_POLYTONE_NOTE.save(deps.as_mut().storage, &note_addr, &chain_name)?;
            let callback_msg = CallbackMessage {
                initiator: env.contract.address,
                initiator_msg: to_json_binary(&IbcClientCallback::CreateAccount {
                    account_id: TEST_ACCOUNT_ID,
                })?,
                result: Callback::FatalError(String::from("error")),
            };

            let msg = ExecuteMsg::Callback(callback_msg.clone());

            let res = execute_as(deps.as_mut(), &note_addr, msg);

            assert_that!(&res)
                .is_err()
                .matches(|e| matches!(e, IbcClientError::IbcFailed(_callback_msg)));

            Ok(())
        }

        #[test]
        fn create_account_missing_wasm_event() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            mock_init(&mut deps)?;
            let env = mock_env();

            let chain_name = TruncatedChainId::from_str(TEST_CHAIN)?;
            let note_addr = deps.api.addr_make("note");
            let remote_proxy = String::from("remote_proxy");

            REVERSE_POLYTONE_NOTE.save(deps.as_mut().storage, &note_addr, &chain_name)?;
            let callback_msg = CallbackMessage {
                initiator: env.contract.address,
                initiator_msg: to_json_binary(&IbcClientCallback::CreateAccount {
                    account_id: TEST_ACCOUNT_ID,
                })?,
                result: Callback::Execute(Ok(ExecutionResponse {
                    executed_by: remote_proxy.clone(),
                    #[allow(deprecated)]
                    result: vec![SubMsgResponse {
                        events: vec![],
                        data: None,
                        msg_responses: vec![],
                    }],
                })),
            };

            let msg = ExecuteMsg::Callback(callback_msg.clone());

            let res = execute_as(deps.as_mut(), &note_addr, msg);

            assert_that!(&res)
                .is_err()
                .matches(|e| matches!(e, IbcClientError::IbcFailed(_callback_msg)));

            Ok(())
        }

        #[test]
        fn create_account_missing_proxy_address_attribute() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            mock_init(&mut deps)?;
            let env = mock_env();

            let chain_name = TruncatedChainId::from_str(TEST_CHAIN)?;
            let note_addr = deps.api.addr_make("note");
            let remote_proxy = String::from("remote_proxy");

            REVERSE_POLYTONE_NOTE.save(deps.as_mut().storage, &note_addr, &chain_name)?;
            let callback_msg = CallbackMessage {
                initiator: env.contract.address,
                initiator_msg: to_json_binary(&IbcClientCallback::CreateAccount {
                    account_id: TEST_ACCOUNT_ID,
                })?,
                result: Callback::Execute(Ok(ExecutionResponse {
                    executed_by: remote_proxy.clone(),
                    #[allow(deprecated)]
                    result: vec![SubMsgResponse {
                        events: vec![Event::new(String::from("wasm"))],
                        data: None,
                        msg_responses: vec![],
                    }],
                })),
            };

            let msg = ExecuteMsg::Callback(callback_msg.clone());

            let res = execute_as(deps.as_mut(), &note_addr, msg);

            assert_that!(&res)
                .is_err()
                .matches(|e| matches!(e, IbcClientError::IbcFailed(_callback_msg)));

            Ok(())
        }

        #[test]
        fn create_account_success() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            mock_init(&mut deps)?;
            let env = mock_env();

            let chain_name = TruncatedChainId::from_str(TEST_CHAIN)?;
            let note_addr = deps.api.addr_make("note");
            let remote_proxy = String::from("remote_proxy");

            REVERSE_POLYTONE_NOTE.save(deps.as_mut().storage, &note_addr, &chain_name)?;
            let callback_msg = CallbackMessage {
                initiator: env.contract.address,
                initiator_msg: to_json_binary(&IbcClientCallback::CreateAccount {
                    account_id: TEST_ACCOUNT_ID,
                })?,
                result: Callback::Execute(Ok(ExecutionResponse {
                    executed_by: remote_proxy.clone(),
                    #[allow(deprecated)]
                    result: vec![SubMsgResponse {
                        events: vec![Event::new(String::from("wasm-abstract"))
                            .add_attribute("action", "create_proxy")
                            .add_attribute("proxy_address", remote_proxy.clone())],
                        data: None,
                        msg_responses: vec![],
                    }],
                })),
            };

            let msg = ExecuteMsg::Callback(callback_msg.clone());

            let res = execute_as(deps.as_mut(), &note_addr, msg)?;

            assert_eq!(
                IbcClientResponse::action("acknowledge_remote_account_registration")
                    .add_attribute("account_id", TEST_ACCOUNT_ID.to_string())
                    .add_attribute("chain", chain_name.to_string()),
                res
            );

            let saved_account = ACCOUNTS.load(
                deps.as_ref().storage,
                (TEST_ACCOUNT_ID.trace(), TEST_ACCOUNT_ID.seq(), &chain_name),
            )?;

            assert_eq!(remote_proxy, saved_account);

            // Verify queries
            let account_response: AccountResponse = from_json(query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::Account {
                    chain_name: chain_name.clone(),
                    account_id: TEST_ACCOUNT_ID,
                },
            )?)?;

            assert_eq!(
                AccountResponse {
                    remote_proxy_addr: Some(remote_proxy.clone())
                },
                account_response
            );

            let accounts_response: ListAccountsResponse = from_json(query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::ListAccounts {
                    start: None,
                    limit: None,
                },
            )?)?;

            assert_eq!(
                ListAccountsResponse {
                    accounts: vec![(TEST_ACCOUNT_ID, chain_name.clone(), remote_proxy.clone())]
                },
                accounts_response
            );

            let proxies_response: ListRemoteProxiesResponse = from_json(query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::ListRemoteProxiesByAccountId {
                    account_id: TEST_ACCOUNT_ID,
                },
            )?)?;

            assert_eq!(
                ListRemoteProxiesResponse {
                    proxies: vec![(chain_name, Some(remote_proxy))]
                },
                proxies_response
            );

            Ok(())
        }
    }
    mod list_proxies_by_account_id {
        use super::*;

        use std::str::FromStr;

        use abstract_std::objects::{account::AccountTrace, AccountId, TruncatedChainId};

        #[test]
        fn works_with_multiple_local_accounts() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            mock_init(&mut deps)?;

            let (trace, seq) = TEST_ACCOUNT_ID.decompose();

            let chain1 = TruncatedChainId::from_str("chain-a")?;
            let proxy1 = String::from("proxy1");

            let chain2 = TruncatedChainId::from_str("chain-b")?;
            let proxy2 = String::from("proxy2");

            ACCOUNTS.save(deps.as_mut().storage, (&trace, seq, &chain1), &proxy1)?;
            ACCOUNTS.save(deps.as_mut().storage, (&trace, seq, &chain2), &proxy2)?;

            let proxies_response: ListRemoteProxiesResponse = from_json(query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::ListRemoteProxiesByAccountId {
                    account_id: TEST_ACCOUNT_ID,
                },
            )?)?;

            assert_eq!(
                ListRemoteProxiesResponse {
                    proxies: vec![(chain1, Some(proxy1)), (chain2, Some(proxy2))]
                },
                proxies_response
            );

            Ok(())
        }

        #[test]
        fn works_with_multiple_remote_accounts() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            mock_init(&mut deps)?;

            let account_id = AccountId::new(
                1,
                AccountTrace::Remote(vec![
                    TruncatedChainId::from_str("juno")?,
                    TruncatedChainId::from_str("osmosis")?,
                ]),
            )?;

            let (trace, seq) = account_id.clone().decompose();

            let terra_chain = TruncatedChainId::from_str("terra")?;
            let terra_proxy = String::from("terra-proxy");

            let archway_chain = TruncatedChainId::from_str("archway")?;
            let archway_proxy = String::from("archway-proxy");

            ACCOUNTS.save(
                deps.as_mut().storage,
                (&trace, seq, &terra_chain),
                &terra_proxy,
            )?;
            ACCOUNTS.save(
                deps.as_mut().storage,
                (&trace, seq, &archway_chain),
                &archway_proxy,
            )?;

            let proxies_response: ListRemoteProxiesResponse = from_json(query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::ListRemoteProxiesByAccountId { account_id },
            )?)?;

            assert_eq!(
                ListRemoteProxiesResponse {
                    proxies: vec![
                        (archway_chain, Some(archway_proxy)),
                        (terra_chain, Some(terra_proxy)),
                    ]
                },
                proxies_response
            );

            Ok(())
        }
    }
}
