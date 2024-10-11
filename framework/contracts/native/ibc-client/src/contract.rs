use abstract_macros::abstract_response;
use abstract_std::{
    ibc_client::*,
    objects::module_version::{assert_cw_contract_upgrade, migrate_module_data, set_module_data},
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
    _msg: InstantiateMsg,
) -> IbcClientResult {
    cw2::set_contract_version(deps.storage, IBC_CLIENT, CONTRACT_VERSION)?;
    set_module_data(
        deps.storage,
        IBC_CLIENT,
        CONTRACT_VERSION,
        &[],
        None::<String>,
    )?;

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
        ExecuteMsg::RemoteAction { host_chain, action } => {
            commands::execute_send_packet(deps, env, info, host_chain, action)
        }
        ExecuteMsg::RegisterInfrastructure { chain, note, host } => {
            commands::execute_register_infrastructure(deps, env, info, chain, host, note)
        }
        ExecuteMsg::SendFunds { host_chain, memo } => {
            commands::execute_send_funds(deps, env, info, host_chain, memo).map_err(Into::into)
        }
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
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> IbcClientResult<QueryResponse> {
    match msg {
        QueryMsg::Ownership {} => to_json_binary(&cw_ownable::get_ownership(deps.storage)?),
        QueryMsg::Config {} => to_json_binary(&queries::config(deps, &env)?),
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
        QueryMsg::ListRemoteAccountsByAccountId { account_id } => {
            to_json_binary(&queries::list_proxies_by_account_id(deps, account_id)?)
        }
    }
    .map_err(Into::into)
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn migrate(deps: DepsMut, env: Env, msg: MigrateMsg) -> IbcClientResult {
    match msg {
        MigrateMsg::Instantiate(instantiate_msg) => {
            abstract_sdk::cw_helpers::migrate_instantiate(deps, env, instantiate_msg, instantiate)
        }
        MigrateMsg::Migrate {} => {
            let to_version: Version = CONTRACT_VERSION.parse().unwrap();

            assert_cw_contract_upgrade(deps.storage, IBC_CLIENT, to_version)?;
            cw2::set_contract_version(deps.storage, IBC_CLIENT, CONTRACT_VERSION)?;
            migrate_module_data(deps.storage, IBC_CLIENT, CONTRACT_VERSION, None::<String>)?;
            Ok(IbcClientResponse::action("migrate"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test_common::mock_init;
    use abstract_std::{account, ibc_client::state::*, registry};
    use abstract_testing::prelude::*;
    use cosmwasm_std::{
        from_json,
        testing::{message_info, mock_dependencies},
        Addr,
    };
    use cw2::CONTRACT;
    use cw_ownable::{Ownership, OwnershipError};
    use speculoos::prelude::*;

    type IbcClientTestResult = Result<(), IbcClientError>;

    fn execute_as(deps: &mut MockDeps, sender: &Addr, msg: ExecuteMsg) -> IbcClientResult {
        let env = mock_env_validated(deps.api);
        execute(deps.as_mut(), env, message_info(sender, &[]), msg)
    }

    fn test_only_admin(msg: ExecuteMsg) -> IbcClientTestResult {
        let mut deps = mock_dependencies();
        mock_init(&mut deps)?;
        let not_admin = deps.api.addr_make("not_admin");

        let res = execute_as(&mut deps, &not_admin, msg);
        assert_that!(&res)
            .is_err()
            .matches(|e| matches!(e, IbcClientError::Ownership(OwnershipError::NotOwner)));

        Ok(())
    }

    #[test]
    fn instantiate_works() -> IbcClientResult<()> {
        let mut deps = mock_dependencies();
        let env = mock_env_validated(deps.api);
        let abstr = AbstractMockAddrs::new(deps.api);
        let owner = abstr.owner;
        let msg = InstantiateMsg {};
        let info = message_info(&owner, &[]);
        let res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();
        assert_that!(res.messages).is_empty();

        let ownership_resp: Ownership<Addr> =
            from_json(query(deps.as_ref(), env, QueryMsg::Ownership {})?)?;

        assert_eq!(ownership_resp.owner, Some(owner));

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
            let env = mock_env_validated(deps.api);
            mock_init(&mut deps)?;

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let res = contract::migrate(deps.as_mut(), env, MigrateMsg::Migrate {});

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
            let env = mock_env_validated(deps.api);
            mock_init(&mut deps)?;

            let big_version = "999.999.999";
            cw2::set_contract_version(deps.as_mut().storage, IBC_CLIENT, big_version)?;

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let res = contract::migrate(deps.as_mut(), env, MigrateMsg::Migrate {});

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
            let env = mock_env_validated(deps.api);
            mock_init(&mut deps)?;

            let old_version = "0.0.0";
            let old_name = "old:contract";
            cw2::set_contract_version(deps.as_mut().storage, old_name, old_version)?;

            let res = contract::migrate(deps.as_mut(), env, MigrateMsg::Migrate {});

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
            let env = mock_env_validated(deps.api);
            mock_init(&mut deps)?;

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let small_version = Version {
                minor: version.minor - 1,
                ..version.clone()
            }
            .to_string();
            cw2::set_contract_version(deps.as_mut().storage, IBC_CLIENT, small_version)?;

            let res = contract::migrate(deps.as_mut(), env, MigrateMsg::Migrate {})?;
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

            let res = execute_as(&mut deps, &abstr.owner, msg);
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

            let note_account_msg = wasm_execute(
                note_addr.to_string(),
                &PolytoneNoteExecuteMsg::Execute {
                    msgs: vec![],
                    callback: Some(CallbackRequest {
                        receiver: mock_env_validated(deps.api).contract.address.to_string(),
                        msg: to_json_binary(&IbcClientCallback::WhoAmI {})?,
                    }),
                    timeout_seconds: PACKET_LIFETIME.into(),
                },
                vec![],
            )?;

            let res = execute_as(&mut deps, &abstr.owner, msg)?;

            assert_eq!(
                IbcClientResponse::action("allow_chain_port").add_message(note_account_msg),
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
                mock_env_validated(deps.api),
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
                mock_env_validated(deps.api),
                QueryMsg::ListRemoteHosts {},
            )?)?;
            let hosts = remote_hosts_response.hosts;
            assert_eq!(vec![(chain_name.clone(), host)], hosts);

            let remote_proxies_response: ListRemoteAccountsResponse = from_json(query(
                deps.as_ref(),
                mock_env_validated(deps.api),
                QueryMsg::ListRemoteProxies {},
            )?)?;
            let hosts = remote_proxies_response.accounts;
            assert_eq!(vec![(chain_name.clone(), None)], hosts);

            let ibc_infratructures_response: ListIbcInfrastructureResponse = from_json(query(
                deps.as_ref(),
                mock_env_validated(deps.api),
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
            objects::{registry::RegistryError, TruncatedChainId},
        };

        use cosmwasm_std::wasm_execute;

        use crate::commands::PACKET_LIFETIME;

        #[test]
        fn throw_when_sender_is_not_account() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            let abstract_addrs = AbstractMockAddrs::new(deps.api);
            let account = test_account(deps.api);
            let not_account = deps.api.addr_make("not_account");
            deps.querier = MockQuerierBuilder::new(deps.api)
                // Account pretends as different account
                .with_contract_item(&not_account, account::state::ACCOUNT_ID, &TEST_ACCOUNT_ID)
                .with_contract_map_entry(
                    &abstract_addrs.registry,
                    registry::state::ACCOUNT_ADDRESSES,
                    (&TEST_ACCOUNT_ID, account.clone()),
                )
                .build();
            mock_init(&mut deps)?;

            let chain_name = TruncatedChainId::from_str(TEST_CHAIN)?;

            let msg = ExecuteMsg::RemoteAction {
                host_chain: chain_name,
                action: HostAction::Dispatch {
                    account_msgs: vec![account::ExecuteMsg::UpdateInfo {
                        name: None,
                        description: None,
                        link: None,
                    }],
                },
            };

            let res = execute_as(&mut deps, &not_account, msg);

            assert_that!(res).is_err().matches(|e| {
                matches!(
                    e,
                    IbcClientError::RegistryError(RegistryError::NotAccount(..))
                )
            });
            Ok(())
        }

        #[test]
        fn cannot_make_internal_call() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            let account = test_account(deps.api);
            deps.querier = MockQuerierBuilder::new(deps.api)
                .account(&account, TEST_ACCOUNT_ID)
                .build();
            mock_init(&mut deps)?;

            let chain_name = TruncatedChainId::from_str(TEST_CHAIN)?;

            let msg = ExecuteMsg::RemoteAction {
                host_chain: chain_name,
                action: HostAction::Internal(InternalAction::Register {
                    name: Some(String::from("name")),
                    description: None,
                    link: None,
                    namespace: None,
                    install_modules: vec![],
                }),
            };

            let res = execute_as(&mut deps, account.addr(), msg);

            assert_that!(res)
                .is_err()
                .matches(|e| matches!(e, IbcClientError::ForbiddenInternalCall {}));
            Ok(())
        }

        #[test]
        fn send_packet_with_no_callback() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            let account = test_account(deps.api);
            deps.querier = MockQuerierBuilder::new(deps.api)
                .account(&account, TEST_ACCOUNT_ID)
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
                account_msgs: vec![account::ExecuteMsg::UpdateInfo {
                    name: None,
                    description: None,
                    link: None,
                }],
            };

            let msg = ExecuteMsg::RemoteAction {
                host_chain: chain_name,
                action: action.clone(),
            };

            let res = execute_as(&mut deps, account.addr(), msg)?;

            let note_message = wasm_execute(
                note_addr.to_string(),
                &PolytoneNoteExecuteMsg::Execute {
                    msgs: vec![wasm_execute(
                        // The note's remote account will call the ibc host
                        remote_ibc_host,
                        &ibc_host::ExecuteMsg::Execute {
                            account_address: account.addr().to_string(),
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
            objects::{registry::RegistryError, ChannelEntry, TruncatedChainId},
            ICS20,
        };
        use cosmwasm_std::{coin, coins, Binary, CosmosMsg, IbcMsg};
        use prost::Name;
        use std::str::FromStr;

        #[test]
        fn throw_when_sender_is_not_account() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            let abstract_addrs = AbstractMockAddrs::new(deps.api);
            let account = test_account(deps.api);
            let module = deps.api.addr_make("application");
            deps.querier = MockQuerierBuilder::new(deps.api)
                // Module is not account
                .with_contract_item(&module, account::state::ACCOUNT_ID, &TEST_ACCOUNT_ID)
                .with_contract_map_entry(
                    &abstract_addrs.registry,
                    registry::state::ACCOUNT_ADDRESSES,
                    (&TEST_ACCOUNT_ID, account.clone()),
                )
                .build();
            mock_init(&mut deps)?;

            let chain_name = TruncatedChainId::from_str(TEST_CHAIN)?;

            let msg = ExecuteMsg::SendFunds {
                host_chain: chain_name,
                memo: None,
            };

            let res = execute_as(&mut deps, &module, msg);

            assert!(matches!(
                res,
                Err(IbcClientError::RegistryError(RegistryError::NotAccount(..)))
            ));
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
            let account = test_account(deps.api);
            deps.querier = MockQuerierBuilder::new(deps.api)
                .account(&account, TEST_ACCOUNT_ID)
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
                memo: None,
            };

            let env = mock_env_validated(deps.api);
            let res = execute(
                deps.as_mut(),
                env,
                message_info(account.addr(), &funds.clone()),
                msg,
            )?;

            let transfer_msgs: Vec<CosmosMsg> = funds
                .into_iter()
                .map(|amount| {
                    IbcMsg::Transfer {
                        channel_id: channel_id.clone(),
                        to_address: remote_addr.clone(),
                        amount,
                        timeout: mock_env_validated(deps.api)
                            .block
                            .time
                            .plus_seconds(PACKET_LIFETIME)
                            .into(),
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
                memo: memo.clone(),
            };

            let res = execute_as(&mut deps, account.addr(), msg)?;

            use prost::Message;
            #[allow(deprecated)]
            let transfer_msgs: Vec<CosmosMsg> = funds
                .into_iter()
                .map(|c| CosmosMsg::Stargate {
                    type_url: ibc_proto::ibc::apps::transfer::v1::MsgTransfer::type_url(),
                    value: Binary::from(
                        ibc_proto::ibc::apps::transfer::v1::MsgTransfer {
                            source_port: "transfer".to_owned(),
                            source_channel: channel_id.clone(),
                            token: Some(ibc_proto::cosmos::base::v1beta1::Coin {
                                denom: c.denom,
                                amount: c.amount.to_string(),
                            }),
                            sender: mock_env_validated(deps.api).contract.address.to_string(),
                            receiver: remote_addr.clone(),
                            timeout_height: None,
                            timeout_timestamp: mock_env_validated(deps.api)
                                .block
                                .time
                                .plus_seconds(PACKET_LIFETIME)
                                .nanos(),
                            memo: memo.clone().unwrap(),
                        }
                        .encode_to_vec(),
                    ),
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
            account,
            ibc::polytone_callbacks::CallbackRequest,
            ibc_host::{self, HostAction, InternalAction},
            objects::{registry::RegistryError, TruncatedChainId},
        };
        use cosmwasm_std::wasm_execute;
        use std::str::FromStr;

        #[test]
        fn throw_when_sender_is_not_account() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            let abstract_addrs = AbstractMockAddrs::new(deps.api);
            let account = test_account(deps.api);
            let not_account = deps.api.addr_make("not_account");
            deps.querier = MockQuerierBuilder::new(deps.api)
                // Account pretends as different account
                .with_contract_item(&not_account, account::state::ACCOUNT_ID, &TEST_ACCOUNT_ID)
                .with_contract_map_entry(
                    &abstract_addrs.registry,
                    registry::state::ACCOUNT_ADDRESSES,
                    (&TEST_ACCOUNT_ID, account.clone()),
                )
                .build();
            mock_init(&mut deps)?;

            let chain_name = TruncatedChainId::from_str(TEST_CHAIN)?;

            let msg = ExecuteMsg::Register {
                host_chain: chain_name,
                namespace: None,
                install_modules: vec![],
            };

            let res = execute_as(&mut deps, &not_account, msg);

            assert_that!(res).is_err().matches(|e| {
                matches!(
                    e,
                    IbcClientError::RegistryError(RegistryError::NotAccount(..))
                )
            });
            Ok(())
        }

        #[test]
        fn works() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            let account = test_account(deps.api);
            deps.querier = MockQuerierBuilder::new(deps.api)
                .account(&account, TEST_ACCOUNT_ID)
                .with_smart_handler(
                    account.addr(),
                    |msg| match from_json::<account::QueryMsg>(msg).unwrap() {
                        account::QueryMsg::Info {} => to_json_binary(&account::InfoResponse {
                            info: account::state::AccountInfo {
                                name: Some(String::from("name")),
                                description: None,
                                link: None,
                            },
                        })
                        .map_err(|e| e.to_string()),
                        _ => todo!(),
                    },
                )
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

            let res = execute_as(&mut deps, account.addr(), msg)?;

            let note_message = wasm_execute(
                note_contract.to_string(),
                &PolytoneNoteExecuteMsg::Execute {
                    msgs: vec![wasm_execute(
                        // The note's remote account will call the ibc host
                        remote_ibc_host,
                        &ibc_host::ExecuteMsg::Execute {
                            account_address: account.addr().to_string(),
                            account_id: TEST_ACCOUNT_ID,
                            action: HostAction::Internal(InternalAction::Register {
                                description: None,
                                link: None,
                                name: Some(String::from("name")),
                                namespace: None,
                                install_modules: vec![],
                            }),
                        },
                        vec![],
                    )?
                    .into()],
                    callback: Some(CallbackRequest {
                        receiver: mock_env_validated(deps.api).contract.address.to_string(),
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

            let res = execute_as(&mut deps, &abstr.owner, msg)?;
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

            let res = execute_as(&mut deps, &abstr.owner, msg)?;
            assert_that!(res.messages).is_empty();

            Ok(())
        }
    }

    mod callback {
        use std::str::FromStr;

        use abstract_std::{
            ibc::polytone_callbacks::{Callback, CallbackMessage, ExecutionResponse},
            objects::TruncatedChainId,
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

            let res = execute_as(&mut deps, &note_addr, msg);

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
            let res = execute_as(&mut deps, &not_note, msg);

            assert_that!(&res)
                .is_err()
                .matches(|e| matches!(e, IbcClientError::Unauthorized { .. }));

            Ok(())
        }

        #[test]
        fn who_am_i_unregistered_chain() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            mock_init(&mut deps)?;
            let env = mock_env_validated(deps.api);

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

            let res = execute_as(&mut deps, &note_addr, msg);

            assert_that!(&res)
                .is_err()
                .matches(|e| matches!(e, IbcClientError::UnregisteredChain { .. }));

            Ok(())
        }

        #[test]
        fn who_am_i_fatal_error() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            mock_init(&mut deps)?;
            let env = mock_env_validated(deps.api);

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

            let res = execute_as(&mut deps, &note_addr, msg);

            assert_that!(&res)
                .is_err()
                .matches(|e| matches!(e, IbcClientError::IbcFailed(_callback_msg)));

            Ok(())
        }

        #[test]
        fn who_am_i_success() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            mock_init(&mut deps)?;
            let env = mock_env_validated(deps.api);

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

            let remote_account = String::from("remote_account");

            let msg = ExecuteMsg::Callback(CallbackMessage {
                initiator: env.contract.address,
                initiator_msg: to_json_binary(&IbcClientCallback::WhoAmI {})?,
                result: Callback::Execute(Ok(ExecutionResponse {
                    executed_by: remote_account.clone(),
                    result: vec![],
                })),
            });

            let res = execute_as(&mut deps, &note_addr, msg)?;

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
                    remote_proxy: Some(remote_account),
                },
                updated_ibc_infra
            );

            Ok(())
        }

        #[test]
        fn create_account_fatal_error() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            mock_init(&mut deps)?;
            let env = mock_env_validated(deps.api);

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

            let res = execute_as(&mut deps, &note_addr, msg);

            assert_that!(&res)
                .is_err()
                .matches(|e| matches!(e, IbcClientError::IbcFailed(_callback_msg)));

            Ok(())
        }

        #[test]
        fn create_account_missing_wasm_event() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            mock_init(&mut deps)?;
            let env = mock_env_validated(deps.api);

            let chain_name = TruncatedChainId::from_str(TEST_CHAIN)?;
            let note_addr = deps.api.addr_make("note");
            let remote_account = String::from("remote_account");

            REVERSE_POLYTONE_NOTE.save(deps.as_mut().storage, &note_addr, &chain_name)?;
            let callback_msg = CallbackMessage {
                initiator: env.contract.address,
                initiator_msg: to_json_binary(&IbcClientCallback::CreateAccount {
                    account_id: TEST_ACCOUNT_ID,
                })?,
                result: Callback::Execute(Ok(ExecutionResponse {
                    executed_by: remote_account.clone(),
                    #[allow(deprecated)]
                    result: vec![SubMsgResponse {
                        events: vec![],
                        data: None,
                        msg_responses: vec![],
                    }],
                })),
            };

            let msg = ExecuteMsg::Callback(callback_msg.clone());

            let res = execute_as(&mut deps, &note_addr, msg);

            assert_that!(&res)
                .is_err()
                .matches(|e| matches!(e, IbcClientError::IbcFailed(_callback_msg)));

            Ok(())
        }

        #[test]
        fn create_account_missing_account_address_attribute() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            mock_init(&mut deps)?;
            let env = mock_env_validated(deps.api);

            let chain_name = TruncatedChainId::from_str(TEST_CHAIN)?;
            let note_addr = deps.api.addr_make("note");
            let remote_account = String::from("remote_account");

            REVERSE_POLYTONE_NOTE.save(deps.as_mut().storage, &note_addr, &chain_name)?;
            let callback_msg = CallbackMessage {
                initiator: env.contract.address,
                initiator_msg: to_json_binary(&IbcClientCallback::CreateAccount {
                    account_id: TEST_ACCOUNT_ID,
                })?,
                result: Callback::Execute(Ok(ExecutionResponse {
                    executed_by: remote_account.clone(),
                    #[allow(deprecated)]
                    result: vec![SubMsgResponse {
                        events: vec![Event::new(String::from("wasm"))],
                        data: None,
                        msg_responses: vec![],
                    }],
                })),
            };

            let msg = ExecuteMsg::Callback(callback_msg.clone());

            let res = execute_as(&mut deps, &note_addr, msg);

            assert_that!(&res)
                .is_err()
                .matches(|e| matches!(e, IbcClientError::IbcFailed(_callback_msg)));

            Ok(())
        }

        #[test]
        fn create_account_success() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            mock_init(&mut deps)?;
            let env = mock_env_validated(deps.api);

            let chain_name = TruncatedChainId::from_str(TEST_CHAIN)?;
            let note_addr = deps.api.addr_make("note");
            let remote_account = String::from("remote_account");

            REVERSE_POLYTONE_NOTE.save(deps.as_mut().storage, &note_addr, &chain_name)?;
            let callback_msg = CallbackMessage {
                initiator: env.contract.address,
                initiator_msg: to_json_binary(&IbcClientCallback::CreateAccount {
                    account_id: TEST_ACCOUNT_ID,
                })?,
                result: Callback::Execute(Ok(ExecutionResponse {
                    executed_by: remote_account.clone(),
                    #[allow(deprecated)]
                    result: vec![SubMsgResponse {
                        events: vec![Event::new(String::from("wasm-abstract"))
                            .add_attribute("action", "create_account")
                            .add_attribute("account_address", remote_account.clone())],
                        data: None,
                        msg_responses: vec![],
                    }],
                })),
            };

            let msg = ExecuteMsg::Callback(callback_msg.clone());

            let res = execute_as(&mut deps, &note_addr, msg)?;

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

            assert_eq!(remote_account, saved_account);

            // Verify queries
            let account_response: AccountResponse = from_json(query(
                deps.as_ref(),
                mock_env_validated(deps.api),
                QueryMsg::Account {
                    chain_name: chain_name.clone(),
                    account_id: TEST_ACCOUNT_ID,
                },
            )?)?;

            assert_eq!(
                AccountResponse {
                    remote_account_addr: Some(remote_account.clone())
                },
                account_response
            );

            let accounts_response: ListAccountsResponse = from_json(query(
                deps.as_ref(),
                mock_env_validated(deps.api),
                QueryMsg::ListAccounts {
                    start: None,
                    limit: None,
                },
            )?)?;

            assert_eq!(
                ListAccountsResponse {
                    accounts: vec![(TEST_ACCOUNT_ID, chain_name.clone(), remote_account.clone())]
                },
                accounts_response
            );

            let proxies_response: ListRemoteAccountsResponse = from_json(query(
                deps.as_ref(),
                mock_env_validated(deps.api),
                QueryMsg::ListRemoteAccountsByAccountId {
                    account_id: TEST_ACCOUNT_ID,
                },
            )?)?;

            assert_eq!(
                ListRemoteAccountsResponse {
                    accounts: vec![(chain_name, Some(remote_account))]
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
            let account1 = String::from("account1");

            let chain2 = TruncatedChainId::from_str("chain-b")?;
            let account2 = String::from("account2");

            ACCOUNTS.save(deps.as_mut().storage, (&trace, seq, &chain1), &account1)?;
            ACCOUNTS.save(deps.as_mut().storage, (&trace, seq, &chain2), &account2)?;

            let proxies_response: ListRemoteAccountsResponse = from_json(query(
                deps.as_ref(),
                mock_env_validated(deps.api),
                QueryMsg::ListRemoteAccountsByAccountId {
                    account_id: TEST_ACCOUNT_ID,
                },
            )?)?;

            assert_eq!(
                ListRemoteAccountsResponse {
                    accounts: vec![(chain1, Some(account1)), (chain2, Some(account2))]
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
            let terra_account = String::from("terra-account");

            let archway_chain = TruncatedChainId::from_str("archway")?;
            let archway_account = String::from("archway-account");

            ACCOUNTS.save(
                deps.as_mut().storage,
                (&trace, seq, &terra_chain),
                &terra_account,
            )?;
            ACCOUNTS.save(
                deps.as_mut().storage,
                (&trace, seq, &archway_chain),
                &archway_account,
            )?;

            let proxies_response: ListRemoteAccountsResponse = from_json(query(
                deps.as_ref(),
                mock_env_validated(deps.api),
                QueryMsg::ListRemoteAccountsByAccountId { account_id },
            )?)?;

            assert_eq!(
                ListRemoteAccountsResponse {
                    accounts: vec![
                        (archway_chain, Some(archway_account)),
                        (terra_chain, Some(terra_account)),
                    ]
                },
                proxies_response
            );

            Ok(())
        }
    }
}
