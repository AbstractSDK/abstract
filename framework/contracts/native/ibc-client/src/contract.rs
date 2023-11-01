use crate::ibc;
use crate::{commands, error::IbcClientError, queries};
use abstract_core::objects::module_version::assert_cw_contract_upgrade;
use abstract_core::{
    ibc_client::{state::*, *},
    objects::{
        ans_host::AnsHost,
        module_version::{migrate_module_data, set_module_data},
    },
    IBC_CLIENT,
};
use abstract_macros::abstract_response;
use abstract_sdk::feature_objects::VersionControlContract;
use cosmwasm_std::{to_json_binary, Deps, DepsMut, Env, MessageInfo, QueryResponse, Response};
use cw_semver::Version;

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

    ADMIN.set(deps, Some(info.sender))?;
    Ok(IbcClientResponse::action("instantiate"))
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> IbcClientResult {
    match msg {
        ExecuteMsg::UpdateAdmin { admin } => {
            let new_admin = deps.api.addr_validate(&admin)?;
            ADMIN
                .execute_update_admin(deps, info, Some(new_admin))
                .map_err(Into::into)
        }
        ExecuteMsg::UpdateConfig {
            ans_host,
            version_control,
        } => commands::execute_update_config(deps, info, ans_host, version_control)
            .map_err(Into::into),
        ExecuteMsg::RemoteAction {
            host_chain,
            action,
            callback_info,
        } => commands::execute_send_packet(deps, env, info, host_chain, action, callback_info),
        ExecuteMsg::RemoteQueries {
            host_chain,
            queries,
            callback_info,
        } => commands::execute_send_query(deps, env, host_chain, queries, callback_info),
        ExecuteMsg::RegisterInfrastructure { chain, note, host } => {
            commands::execute_register_infrastructure(deps, env, info, chain, host, note)
        }
        ExecuteMsg::SendFunds { host_chain, funds } => {
            commands::execute_send_funds(deps, env, info, host_chain, funds).map_err(Into::into)
        }
        ExecuteMsg::Register { host_chain } => {
            commands::execute_register_account(deps, info, host_chain)
        }
        ExecuteMsg::RemoveHost { host_chain } => {
            commands::execute_remove_host(deps, info, host_chain).map_err(Into::into)
        }
        ExecuteMsg::Callback(c) => {
            ibc::receive_action_callback(deps, env, info, c).map_err(Into::into)
        }
    }
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> IbcClientResult<QueryResponse> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&queries::config(deps)?),
        QueryMsg::Host { chain_name } => to_json_binary(&queries::host(deps, chain_name)?),
        QueryMsg::Account { chain, account_id } => {
            to_json_binary(&queries::account(deps, chain, account_id)?)
        }
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
    use crate::{queries::config, test_common::mock_init};
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        Addr,
    };
    use cw2::CONTRACT;

    use abstract_testing::addresses::TEST_CREATOR;
    use abstract_testing::prelude::{TEST_ANS_HOST, TEST_VERSION_CONTROL};
    use speculoos::prelude::*;

    type IbcClientTestResult = Result<(), IbcClientError>;

    fn execute_as(deps: DepsMut, sender: &str, msg: ExecuteMsg) -> IbcClientResult {
        execute(deps, mock_env(), mock_info(sender, &[]), msg)
    }

    fn execute_as_admin(deps: DepsMut, msg: ExecuteMsg) -> IbcClientResult {
        execute_as(deps, TEST_CREATOR, msg)
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

    #[test]
    fn instantiate_works() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            ans_host_address: TEST_ANS_HOST.into(),
            version_control_address: TEST_VERSION_CONTROL.into(),
        };
        let info = mock_info(TEST_CREATOR, &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_that!(res.messages).is_empty();

        // config
        let expected_config = Config {
            version_control: VersionControlContract::new(Addr::unchecked(TEST_VERSION_CONTROL)),
            ans_host: AnsHost::new(Addr::unchecked(TEST_ANS_HOST)),
        };

        let config_resp = config(deps.as_ref()).unwrap();
        assert_that!(config_resp.admin.as_str()).is_equal_to(TEST_CREATOR);

        let actual_config = CONFIG.load(deps.as_ref().storage).unwrap();
        assert_that!(actual_config).is_equal_to(expected_config);

        // CW2
        let cw2_info = CONTRACT.load(&deps.storage).unwrap();
        assert_that!(cw2_info.version).is_equal_to(CONTRACT_VERSION.to_string());
        assert_that!(cw2_info.contract).is_equal_to(IBC_CLIENT.to_string());
    }

    mod migrate {
        use super::*;
        use crate::contract;

        use abstract_core::AbstractError;
        use cosmwasm_std::testing::mock_dependencies;

        #[test]
        fn disallow_same_version() -> IbcClientResult<()> {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

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
            mock_init(deps.as_mut())?;

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
            mock_init(deps.as_mut())?;

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
            mock_init(deps.as_mut())?;

            let small_version = "0.0.0";
            cw2::set_contract_version(deps.as_mut().storage, IBC_CLIENT, small_version)?;

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let res = contract::migrate(deps.as_mut(), mock_env(), MigrateMsg {})?;
            assert_that!(res.messages).has_length(0);

            assert_that!(cw2::get_contract_version(&deps.storage)?.version)
                .is_equal_to(version.to_string());
            Ok(())
        }
    }

    mod register_infrastructure {
        use std::str::FromStr;

        use abstract_core::objects::chain_name::ChainName;
        use abstract_testing::prelude::TEST_CHAIN;
        use cosmwasm_std::{from_json, wasm_execute};
        use polytone::callbacks::CallbackRequest;

        use crate::commands::PACKET_LIFETIME;

        use super::*;

        #[test]
        fn only_admin() -> IbcClientResult<()> {
            test_only_admin(ExecuteMsg::RegisterInfrastructure {
                chain: String::from("host-chain"),
                note: String::from("note"),
                host: String::from("host"),
            })
        }

        #[test]
        fn cannot_register_if_already_exists() -> IbcClientTestResult {
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

            let msg = ExecuteMsg::RegisterInfrastructure {
                chain: String::from(TEST_CHAIN),
                note: String::from("note"),
                host: String::from("test_remote_host"),
            };

            let res = execute_as_admin(deps.as_mut(), msg);
            assert_that!(&res)
                .is_err()
                .matches(|e| matches!(e, IbcClientError::HostAddressExists {}));

            Ok(())
        }

        #[test]
        fn register_infrastructure() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let chain_name = ChainName::from_str(TEST_CHAIN)?;
            let note = String::from("note");
            let host = String::from("test_remote_host");

            let msg = ExecuteMsg::RegisterInfrastructure {
                chain: chain_name.to_string(),
                note: note.clone(),
                host: host.clone(),
            };

            let note_proxy_msg = wasm_execute(
                note.clone(),
                &polytone_note::msg::ExecuteMsg::Execute {
                    msgs: vec![],
                    callback: Some(CallbackRequest {
                        receiver: mock_env().contract.address.to_string(),
                        msg: to_json_binary(&IbcClientCallback::WhoAmI {})?,
                    }),
                    timeout_seconds: PACKET_LIFETIME.into(),
                },
                vec![],
            )?;

            let res = execute_as_admin(deps.as_mut(), msg)?;

            assert_eq!(
                IbcClientResponse::action("allow_chain_port").add_message(note_proxy_msg),
                res
            );

            // Verify IBC_INFRA
            let ibc_infra = IBC_INFRA.load(deps.as_ref().storage, &chain_name)?;
            let expected_ibc_infra = IbcInfrastructure {
                polytone_note: Addr::unchecked(note.clone()),
                remote_abstract_host: host.clone(),
                remote_proxy: None,
            };

            assert_eq!(expected_ibc_infra, ibc_infra);

            // Verify REVERSE_POLYTONE_NOTE
            let reverse_note =
                REVERSE_POLYTONE_NOTE.load(deps.as_ref().storage, &Addr::unchecked(note))?;

            assert_eq!(chain_name, reverse_note);

            // Verify queries
            let host_response: HostResponse = from_json(query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::Host {
                    chain_name: chain_name.to_string(),
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
        use cosmwasm_std::Binary;
        use std::str::FromStr;

        use abstract_core::{
            ibc::CallbackInfo,
            ibc_host::{self, HostAction, InternalAction},
            manager,
            objects::{account::TEST_ACCOUNT_ID, chain_name::ChainName},
        };
        use abstract_sdk::AbstractSdkError;
        use abstract_testing::prelude::{
            mocked_account_querier_builder, TEST_CHAIN, TEST_MANAGER, TEST_PROXY,
        };
        use cosmwasm_std::wasm_execute;

        use crate::commands::PACKET_LIFETIME;
        use polytone::callbacks::CallbackRequest;

        use super::*;

        #[test]
        fn throw_when_sender_is_not_proxy() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            deps.querier = mocked_account_querier_builder().build();
            mock_init(deps.as_mut())?;

            let chain_name = ChainName::from_str(TEST_CHAIN)?;

            let msg = ExecuteMsg::RemoteAction {
                host_chain: chain_name.to_string(),
                action: HostAction::Dispatch {
                    manager_msg: manager::ExecuteMsg::UpdateInfo {
                        name: None,
                        description: None,
                        link: None,
                    },
                },
                callback_info: None,
            };

            let res = execute_as(deps.as_mut(), TEST_MANAGER, msg);

            assert_that!(res).is_err().matches(|e| {
                matches!(
                    e,
                    IbcClientError::AbstractSdk(AbstractSdkError::NotProxy(..))
                )
            });
            Ok(())
        }

        #[test]
        fn cannot_make_internal_call() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            deps.querier = mocked_account_querier_builder().build();
            mock_init(deps.as_mut())?;

            let chain_name = ChainName::from_str(TEST_CHAIN)?;

            let msg = ExecuteMsg::RemoteAction {
                host_chain: chain_name.to_string(),
                action: HostAction::Internal(InternalAction::Register {
                    name: String::from("name"),
                    description: None,
                    link: None,
                }),
                callback_info: None,
            };

            let res = execute_as(deps.as_mut(), TEST_PROXY, msg);

            assert_that!(res)
                .is_err()
                .matches(|e| matches!(e, IbcClientError::ForbiddenInternalCall {}));
            Ok(())
        }

        #[test]
        fn send_packet_with_no_callback() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            deps.querier = mocked_account_querier_builder().build();
            mock_init(deps.as_mut())?;

            let chain_name = ChainName::from_str(TEST_CHAIN)?;
            let note_contract = Addr::unchecked("note");
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

            let action = HostAction::Dispatch {
                manager_msg: manager::ExecuteMsg::UpdateInfo {
                    name: None,
                    description: None,
                    link: None,
                },
            };

            let msg = ExecuteMsg::RemoteAction {
                host_chain: chain_name.to_string(),
                action: action.clone(),
                callback_info: None,
            };

            let res = execute_as(deps.as_mut(), TEST_PROXY, msg)?;

            let note_message = wasm_execute(
                note_contract.to_string(),
                &polytone_note::msg::ExecuteMsg::Execute {
                    msgs: vec![wasm_execute(
                        // The note's remote proxy will call the ibc host
                        remote_ibc_host,
                        &ibc_host::ExecuteMsg::Execute {
                            proxy_address: TEST_PROXY.to_owned(),
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

        #[test]
        fn send_packet_with_callback() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            deps.querier = mocked_account_querier_builder().build();
            mock_init(deps.as_mut())?;

            let chain_name = ChainName::from_str(TEST_CHAIN)?;
            let note_contract = Addr::unchecked("note");
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

            let action = HostAction::Dispatch {
                manager_msg: manager::ExecuteMsg::UpdateInfo {
                    name: None,
                    description: None,
                    link: None,
                },
            };

            let callback_info = CallbackInfo {
                id: String::from("id"),
                receiver: String::from("receiver"),
                msg: Some(Binary(vec![])),
            };

            let callback_request = CallbackRequest {
                msg: to_json_binary(&IbcClientCallback::UserRemoteAction(callback_info.clone()))?,
                receiver: mock_env().contract.address.to_string(),
            };

            let msg = ExecuteMsg::RemoteAction {
                host_chain: chain_name.to_string(),
                action: action.clone(),
                callback_info: Some(callback_info),
            };

            let res = execute_as(deps.as_mut(), TEST_PROXY, msg)?;

            let note_message = wasm_execute(
                note_contract.to_string(),
                &polytone_note::msg::ExecuteMsg::Execute {
                    msgs: vec![wasm_execute(
                        // The note's remote proxy will call the ibc host
                        remote_ibc_host,
                        &ibc_host::ExecuteMsg::Execute {
                            proxy_address: TEST_PROXY.to_owned(),
                            account_id: TEST_ACCOUNT_ID,
                            action,
                        },
                        vec![],
                    )?
                    .into()],
                    callback: Some(callback_request),
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

    mod remote_query {
        use std::str::FromStr;

        use crate::commands::PACKET_LIFETIME;
        use abstract_core::{ibc::CallbackInfo, objects::chain_name::ChainName};
        use abstract_testing::prelude::{mocked_account_querier_builder, TEST_CHAIN};
        use cosmwasm_std::{wasm_execute, BankQuery, Binary, QueryRequest};
        use polytone::callbacks::CallbackRequest;

        use super::*;

        #[test]
        fn works() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            deps.querier = mocked_account_querier_builder().build();
            mock_init(deps.as_mut())?;

            let chain_name = ChainName::from_str(TEST_CHAIN)?;
            let note_contract = Addr::unchecked("note");
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

            let callback_info = CallbackInfo {
                id: String::from("id"),
                receiver: String::from("receiver"),
                msg: Some(Binary(vec![])),
            };

            let callback_request = CallbackRequest {
                msg: to_json_binary(&IbcClientCallback::UserRemoteAction(callback_info.clone()))?,
                receiver: mock_env().contract.address.to_string(),
            };

            let queries = vec![QueryRequest::Bank(BankQuery::AllBalances {
                address: String::from("addr"),
            })];

            let msg = ExecuteMsg::RemoteQueries {
                host_chain: chain_name.to_string(),
                queries: queries.clone(),
                callback_info,
            };

            let res = execute_as(deps.as_mut(), "sender", msg)?;

            let note_message = wasm_execute(
                note_contract.to_string(),
                &polytone_note::msg::ExecuteMsg::Query {
                    msgs: queries,
                    callback: callback_request,
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
        use std::str::FromStr;

        use crate::commands::PACKET_LIFETIME;

        use super::*;
        use abstract_core::{
            objects::{account::TEST_ACCOUNT_ID, chain_name::ChainName, ChannelEntry},
            ICS20,
        };
        use abstract_sdk::AbstractSdkError;
        use abstract_testing::prelude::{
            mocked_account_querier_builder, TEST_CHAIN, TEST_MANAGER, TEST_PROXY,
        };
        use cosmwasm_std::{coins, Coin, CosmosMsg, IbcMsg};

        #[test]
        fn throw_when_sender_is_not_proxy() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            deps.querier = mocked_account_querier_builder().build();
            mock_init(deps.as_mut())?;

            let chain_name = ChainName::from_str(TEST_CHAIN)?;

            let msg = ExecuteMsg::SendFunds {
                host_chain: chain_name.to_string(),
                funds: coins(1, "denom"),
            };

            let res = execute_as(deps.as_mut(), TEST_MANAGER, msg);

            assert_that!(res).is_err().matches(|e| {
                matches!(
                    e,
                    IbcClientError::AbstractSdk(AbstractSdkError::NotProxy(..))
                )
            });
            Ok(())
        }

        #[test]
        fn works() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            let chain_name = ChainName::from_str(TEST_CHAIN)?;
            let channel_entry = ChannelEntry {
                connected_chain: chain_name.clone(),
                protocol: String::from(ICS20),
            };
            let channel_id = String::from("1");
            let channels: Vec<(&ChannelEntry, String)> = vec![(&channel_entry, channel_id.clone())];
            deps.querier = mocked_account_querier_builder().channels(channels).build();
            mock_init(deps.as_mut())?;

            let remote_addr = String::from("remote_addr");

            ACCOUNTS.save(
                deps.as_mut().storage,
                (TEST_ACCOUNT_ID.trace(), TEST_ACCOUNT_ID.seq(), &chain_name),
                &remote_addr,
            )?;

            let funds: Vec<Coin> = coins(1, "denom");

            let msg = ExecuteMsg::SendFunds {
                host_chain: chain_name.to_string(),
                funds: funds.clone(),
            };

            let res = execute_as(deps.as_mut(), TEST_PROXY, msg)?;

            let transfer_msgs: Vec<CosmosMsg> = funds
                .into_iter()
                .map(|c| {
                    IbcMsg::Transfer {
                        channel_id: channel_id.clone(),
                        to_address: remote_addr.clone(),
                        amount: c,
                        timeout: mock_env().block.time.plus_seconds(PACKET_LIFETIME).into(),
                    }
                    .into()
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
        use std::str::FromStr;

        use abstract_core::{
            ibc_host::{self, HostAction, InternalAction},
            manager,
            objects::{
                account::TEST_ACCOUNT_ID, chain_name::ChainName, gov_type::GovernanceDetails,
            },
        };
        use abstract_sdk::AbstractSdkError;
        use abstract_testing::prelude::{
            mocked_account_querier_builder, TEST_CHAIN, TEST_MANAGER, TEST_PROXY,
        };
        use cosmwasm_std::{from_json, wasm_execute};

        use crate::commands::PACKET_LIFETIME;

        use super::*;

        #[test]
        fn throw_when_sender_is_not_proxy() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            deps.querier = mocked_account_querier_builder().build();
            mock_init(deps.as_mut())?;

            let chain_name = ChainName::from_str(TEST_CHAIN)?;

            let msg = ExecuteMsg::Register {
                host_chain: chain_name.to_string(),
            };

            let res = execute_as(deps.as_mut(), TEST_MANAGER, msg);

            assert_that!(res).is_err().matches(|e| {
                matches!(
                    e,
                    IbcClientError::AbstractSdk(AbstractSdkError::NotProxy(..))
                )
            });
            Ok(())
        }

        #[test]
        fn works() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            deps.querier = mocked_account_querier_builder()
                .builder()
                .with_smart_handler(TEST_MANAGER, |msg| {
                    match from_json::<manager::QueryMsg>(msg).unwrap() {
                        manager::QueryMsg::Info {} => to_json_binary(&manager::InfoResponse {
                            info: manager::state::AccountInfo {
                                name: String::from("name"),
                                governance_details: GovernanceDetails::Monarchy {
                                    monarch: Addr::unchecked("monarch"),
                                },
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
            mock_init(deps.as_mut())?;

            let chain_name = ChainName::from_str(TEST_CHAIN)?;
            let note_contract = Addr::unchecked("note");
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
                host_chain: chain_name.to_string(),
            };

            let res = execute_as(deps.as_mut(), TEST_PROXY, msg)?;

            let note_message = wasm_execute(
                note_contract.to_string(),
                &polytone_note::msg::ExecuteMsg::Execute {
                    msgs: vec![wasm_execute(
                        // The note's remote proxy will call the ibc host
                        remote_ibc_host,
                        &ibc_host::ExecuteMsg::Execute {
                            proxy_address: TEST_PROXY.to_string(),
                            account_id: TEST_ACCOUNT_ID,
                            action: HostAction::Internal(InternalAction::Register {
                                description: None,
                                link: None,
                                name: String::from("name"),
                            }),
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
                IbcClientResponse::action("handle_register").add_message(note_message),
                res
            );

            Ok(())
        }
    }

    mod update_config {
        use std::str::FromStr;

        use super::*;
        use abstract_core::{
            ibc_client::state::Config,
            objects::{account::TEST_ACCOUNT_ID, chain_name::ChainName},
        };
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
                (
                    TEST_ACCOUNT_ID.trace(),
                    TEST_ACCOUNT_ID.seq(),
                    &ChainName::from_str("channel")?,
                ),
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
        use std::str::FromStr;

        use abstract_core::objects::chain_name::ChainName;
        use abstract_testing::prelude::TEST_CHAIN;

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

    mod callback {
        use abstract_core::{
            ibc::{CallbackInfo, IbcResponseMsg},
            objects::{account::TEST_ACCOUNT_ID, chain_name::ChainName},
        };
        use abstract_testing::prelude::TEST_CHAIN;
        use cosmwasm_std::{from_json, Binary, Event, SubMsgResponse};
        use polytone::callbacks::{Callback, CallbackMessage, ExecutionResponse};
        use std::str::FromStr;

        use super::*;

        #[test]
        fn invalid_initiator() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let note = Addr::unchecked("note");
            let chain_name = ChainName::from_str(TEST_CHAIN)?;
            REVERSE_POLYTONE_NOTE.save(deps.as_mut().storage, &note, &chain_name)?;

            let msg = ExecuteMsg::Callback(CallbackMessage {
                initiator: Addr::unchecked("invalid_initiator"),
                initiator_msg: Binary(vec![]),
                result: Callback::Execute(Ok(ExecutionResponse {
                    executed_by: String::from("addr"),
                    result: vec![],
                })),
            });

            let res = execute_as(deps.as_mut(), note.as_ref(), msg);

            assert_that!(&res)
                .is_err()
                .matches(|e| matches!(e, IbcClientError::Unauthorized { .. }));

            Ok(())
        }

        #[test]
        fn caller_not_note() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let note = Addr::unchecked("note");
            let chain_name = ChainName::from_str(TEST_CHAIN)?;
            REVERSE_POLYTONE_NOTE.save(deps.as_mut().storage, &note, &chain_name)?;

            let msg = ExecuteMsg::Callback(CallbackMessage {
                initiator: Addr::unchecked("invalid_initiator"),
                initiator_msg: Binary(vec![]),
                result: Callback::Execute(Ok(ExecutionResponse {
                    executed_by: String::from("addr"),
                    result: vec![],
                })),
            });

            let res = execute_as(deps.as_mut(), "not_note", msg);

            assert_that!(&res)
                .is_err()
                .matches(|e| matches!(e, IbcClientError::Unauthorized { .. }));

            Ok(())
        }

        #[test]
        fn who_am_i_unregistered_chain() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;
            let env = mock_env();

            let note = Addr::unchecked("note");
            let chain_name = ChainName::from_str(TEST_CHAIN)?;
            REVERSE_POLYTONE_NOTE.save(deps.as_mut().storage, &note, &chain_name)?;

            let msg = ExecuteMsg::Callback(CallbackMessage {
                initiator: env.contract.address,
                initiator_msg: to_json_binary(&IbcClientCallback::WhoAmI {})?,
                result: Callback::Execute(Ok(ExecutionResponse {
                    executed_by: String::from("addr"),
                    result: vec![],
                })),
            });

            let res = execute_as(deps.as_mut(), note.as_ref(), msg);

            assert_that!(&res)
                .is_err()
                .matches(|e| matches!(e, IbcClientError::UnregisteredChain { .. }));

            Ok(())
        }

        #[test]
        fn who_am_i_fatal_error() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;
            let env = mock_env();

            let chain_name = ChainName::from_str(TEST_CHAIN)?;
            let note = Addr::unchecked("note");
            let remote_ibc_host = String::from("test_remote_host");

            IBC_INFRA.save(
                deps.as_mut().storage,
                &chain_name,
                &IbcInfrastructure {
                    polytone_note: note.clone(),
                    remote_abstract_host: remote_ibc_host.clone(),
                    remote_proxy: None,
                },
            )?;
            REVERSE_POLYTONE_NOTE.save(deps.as_mut().storage, &note, &chain_name)?;
            let callback_msg = CallbackMessage {
                initiator: env.contract.address,
                initiator_msg: to_json_binary(&IbcClientCallback::WhoAmI {})?,
                result: Callback::FatalError(String::from("error")),
            };

            let msg = ExecuteMsg::Callback(callback_msg.clone());

            let res = execute_as(deps.as_mut(), note.as_ref(), msg);

            assert_that!(&res)
                .is_err()
                .matches(|e| matches!(e, IbcClientError::IbcFailed(_callback_msg)));

            Ok(())
        }

        #[test]
        fn who_am_i_success() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;
            let env = mock_env();

            let chain_name = ChainName::from_str(TEST_CHAIN)?;
            let note = Addr::unchecked("note");
            let remote_ibc_host = String::from("test_remote_host");

            IBC_INFRA.save(
                deps.as_mut().storage,
                &chain_name,
                &IbcInfrastructure {
                    polytone_note: note.clone(),
                    remote_abstract_host: remote_ibc_host.clone(),
                    remote_proxy: None,
                },
            )?;
            REVERSE_POLYTONE_NOTE.save(deps.as_mut().storage, &note, &chain_name)?;

            let remote_proxy = String::from("remote_proxy");

            let msg = ExecuteMsg::Callback(CallbackMessage {
                initiator: env.contract.address,
                initiator_msg: to_json_binary(&IbcClientCallback::WhoAmI {})?,
                result: Callback::Execute(Ok(ExecutionResponse {
                    executed_by: remote_proxy.clone(),
                    result: vec![],
                })),
            });

            let res = execute_as(deps.as_mut(), note.as_ref(), msg)?;

            assert_eq!(
                IbcClientResponse::action("register_remote_proxy")
                    .add_attribute("chain", chain_name.to_string()),
                res
            );

            let updated_ibc_infra = IBC_INFRA.load(deps.as_ref().storage, &chain_name)?;

            assert_eq!(
                IbcInfrastructure {
                    polytone_note: note.clone(),
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
            mock_init(deps.as_mut())?;
            let env = mock_env();

            let chain_name = ChainName::from_str(TEST_CHAIN)?;
            let note = Addr::unchecked("note");

            REVERSE_POLYTONE_NOTE.save(deps.as_mut().storage, &note, &chain_name)?;
            let callback_msg = CallbackMessage {
                initiator: env.contract.address,
                initiator_msg: to_json_binary(&IbcClientCallback::CreateAccount {
                    account_id: TEST_ACCOUNT_ID,
                })?,
                result: Callback::FatalError(String::from("error")),
            };

            let msg = ExecuteMsg::Callback(callback_msg.clone());

            let res = execute_as(deps.as_mut(), note.as_ref(), msg);

            assert_that!(&res)
                .is_err()
                .matches(|e| matches!(e, IbcClientError::IbcFailed(_callback_msg)));

            Ok(())
        }

        #[test]
        fn create_account_missing_wasm_event() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;
            let env = mock_env();

            let chain_name = ChainName::from_str(TEST_CHAIN)?;
            let note = Addr::unchecked("note");
            let remote_proxy = String::from("remote_proxy");

            REVERSE_POLYTONE_NOTE.save(deps.as_mut().storage, &note, &chain_name)?;
            let callback_msg = CallbackMessage {
                initiator: env.contract.address,
                initiator_msg: to_json_binary(&IbcClientCallback::CreateAccount {
                    account_id: TEST_ACCOUNT_ID,
                })?,
                result: Callback::Execute(Ok(ExecutionResponse {
                    executed_by: remote_proxy.clone(),
                    result: vec![SubMsgResponse {
                        events: vec![],
                        data: None,
                    }],
                })),
            };

            let msg = ExecuteMsg::Callback(callback_msg.clone());

            let res = execute_as(deps.as_mut(), note.as_ref(), msg);

            assert_that!(&res)
                .is_err()
                .matches(|e| matches!(e, IbcClientError::IbcFailed(_callback_msg)));

            Ok(())
        }

        #[test]
        fn create_account_missing_proxy_address_attribute() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;
            let env = mock_env();

            let chain_name = ChainName::from_str(TEST_CHAIN)?;
            let note = Addr::unchecked("note");
            let remote_proxy = String::from("remote_proxy");

            REVERSE_POLYTONE_NOTE.save(deps.as_mut().storage, &note, &chain_name)?;
            let callback_msg = CallbackMessage {
                initiator: env.contract.address,
                initiator_msg: to_json_binary(&IbcClientCallback::CreateAccount {
                    account_id: TEST_ACCOUNT_ID,
                })?,
                result: Callback::Execute(Ok(ExecutionResponse {
                    executed_by: remote_proxy.clone(),
                    result: vec![SubMsgResponse {
                        events: vec![Event::new(String::from("wasm"))],
                        data: None,
                    }],
                })),
            };

            let msg = ExecuteMsg::Callback(callback_msg.clone());

            let res = execute_as(deps.as_mut(), note.as_ref(), msg);

            assert_that!(&res)
                .is_err()
                .matches(|e| matches!(e, IbcClientError::IbcFailed(_callback_msg)));

            Ok(())
        }

        #[test]
        fn create_account_success() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;
            let env = mock_env();

            let chain_name = ChainName::from_str(TEST_CHAIN)?;
            let note = Addr::unchecked("note");
            let remote_proxy = String::from("remote_proxy");

            REVERSE_POLYTONE_NOTE.save(deps.as_mut().storage, &note, &chain_name)?;
            let callback_msg = CallbackMessage {
                initiator: env.contract.address,
                initiator_msg: to_json_binary(&IbcClientCallback::CreateAccount {
                    account_id: TEST_ACCOUNT_ID,
                })?,
                result: Callback::Execute(Ok(ExecutionResponse {
                    executed_by: remote_proxy.clone(),
                    result: vec![SubMsgResponse {
                        events: vec![Event::new(String::from("wasm"))
                            .add_attribute("action", "create_proxy")
                            .add_attribute("proxy_address", remote_proxy.clone())],
                        data: None,
                    }],
                })),
            };

            let msg = ExecuteMsg::Callback(callback_msg.clone());

            let res = execute_as(deps.as_mut(), note.as_ref(), msg)?;

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
                    chain: chain_name.to_string(),
                    account_id: TEST_ACCOUNT_ID,
                },
            )?)?;

            assert_eq!(
                AccountResponse {
                    remote_proxy_addr: remote_proxy.clone()
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

        #[test]
        fn user_remote_action() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;
            let env = mock_env();

            let chain_name = ChainName::from_str(TEST_CHAIN)?;
            let note = Addr::unchecked("note");
            let remote_proxy = String::from("remote_proxy");

            REVERSE_POLYTONE_NOTE.save(deps.as_mut().storage, &note, &chain_name)?;
            let id = String::from("id");
            let callback_info_msg = Binary(vec![]);
            let receiver = String::from("receiver");
            let callback_msg = CallbackMessage {
                initiator: env.contract.address,
                initiator_msg: to_json_binary(&IbcClientCallback::UserRemoteAction(
                    CallbackInfo {
                        id: id.clone(),
                        msg: Some(callback_info_msg.clone()),
                        receiver: receiver.clone(),
                    },
                ))?,
                result: Callback::Execute(Ok(ExecutionResponse {
                    executed_by: remote_proxy.clone(),
                    result: vec![],
                })),
            };
            let msg = ExecuteMsg::Callback(callback_msg.clone());

            let res = execute_as(deps.as_mut(), note.as_ref(), msg)?;

            assert_eq!(
                IbcClientResponse::action("user_specific_callback")
                    .add_message(
                        IbcResponseMsg {
                            id: id.clone(),
                            msg: Some(callback_info_msg),
                            result: callback_msg.result,
                        }
                        .into_cosmos_msg(receiver)?
                    )
                    .add_attribute("chain", chain_name.to_string())
                    .add_attribute("callback_id", id),
                res
            );

            Ok(())
        }
    }
}
