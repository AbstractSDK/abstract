use crate::msg::*;
use crate::state::{Config, CONFIG};
use abstract_macros::abstract_response;
use abstract_sdk::feature_objects::VersionControlContract;
use abstract_std::{
    objects::{
        ans_host::AnsHost,
        module_version::{assert_cw_contract_upgrade, migrate_module_data},
    },
    IBC_CLIENT,
};
use cosmwasm_std::{to_json_binary, Deps, DepsMut, Env, MessageInfo, QueryResponse, Response};
use cw_semver::Version;

use crate::{error::IcaClientError, queries};

pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub(crate) type IcaClientResult<T = Response> = Result<T, IcaClientError>;

#[abstract_response(IBC_CLIENT)]
pub(crate) struct IbcClientResponse;

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> IcaClientResult {
    cw2::set_contract_version(deps.storage, IBC_CLIENT, CONTRACT_VERSION)?;
    let cfg = Config {
        version_control: VersionControlContract::new(
            deps.api.addr_validate(&msg.version_control_address)?,
        ),
        ans_host: AnsHost {
            address: deps.api.addr_validate(&msg.ans_host_address)?,
        },
    };
    CONFIG.save(deps.storage, &cfg)?;

    // cw_ownable::initialize_owner(deps.storage, deps.api, Some(info.sender.as_str()))?;
    Ok(IbcClientResponse::action("instantiate"))
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn execute(_deps: DepsMut, _env: Env, _info: MessageInfo, msg: ExecuteMsg) -> IcaClientResult {
    match msg {}
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> IcaClientResult<QueryResponse> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&queries::config(deps)?).map_err(Into::into),
        QueryMsg::Ownership {} => {
            to_json_binary(&cw_ownable::get_ownership(deps.storage)?).map_err(Into::into)
        }
        QueryMsg::IcaAction {
            proxy_address,
            chain,
            actions,
        } => to_json_binary(&queries::ica_action(
            deps,
            env,
            proxy_address,
            chain,
            actions,
        )?)
        .map_err(Into::into),
    }
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> IcaClientResult {
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
    use abstract_testing::prelude::{TEST_ANS_HOST, TEST_VERSION_CONTROL, *};
    use cosmwasm_std::{
        from_json,
        testing::{mock_dependencies, mock_env, mock_info},
        Addr,
    };
    use cw2::CONTRACT;
    use cw_ownable::{Ownership, OwnershipError};
    use speculoos::prelude::*;

    type IbcClientTestResult = Result<(), IcaClientError>;

    fn execute_as(deps: DepsMut, sender: &str, msg: ExecuteMsg) -> IcaClientResult {
        execute(deps, mock_env(), mock_info(sender, &[]), msg)
    }

    fn execute_as_admin(deps: DepsMut, msg: ExecuteMsg) -> IcaClientResult {
        execute_as(deps, OWNER, msg)
    }

    fn test_only_admin(msg: ExecuteMsg) -> IbcClientTestResult {
        let mut deps = mock_dependencies();
        mock_init(deps.as_mut())?;

        let res = execute_as(deps.as_mut(), "not_admin", msg);
        assert_that!(&res)
            .is_err()
            .matches(|e| matches!(e, IcaClientError::Ownership(OwnershipError::NotOwner)));

        Ok(())
    }

    #[test]
    fn instantiate_works() -> IcaClientResult<()> {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            ans_host_address: TEST_ANS_HOST.into(),
            version_control_address: TEST_VERSION_CONTROL.into(),
        };
        let info = mock_info(OWNER, &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_that!(res.messages).is_empty();

        // config
        let expected_config = Config {
            version_control: VersionControlContract::new(Addr::unchecked(TEST_VERSION_CONTROL)),
            ans_host: AnsHost::new(Addr::unchecked(TEST_ANS_HOST)),
        };

        let ownership_resp: Ownership<Addr> =
            from_json(query(deps.as_ref(), mock_env(), QueryMsg::Ownership {})?)?;

        assert_eq!(
            ownership_resp.owner,
            Some(Addr::unchecked(OWNER.to_owned()))
        );

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
        fn disallow_same_version() -> IcaClientResult<()> {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let res = contract::migrate(deps.as_mut(), mock_env(), MigrateMsg {});

            assert_that!(res)
                .is_err()
                .is_equal_to(IcaClientError::Abstract(
                    AbstractError::CannotDowngradeContract {
                        contract: IBC_CLIENT.to_string(),
                        from: version.to_string().parse().unwrap(),
                        to: version.to_string().parse().unwrap(),
                    },
                ));

            Ok(())
        }

        #[test]
        fn disallow_downgrade() -> IcaClientResult<()> {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let big_version = "999.999.999";
            cw2::set_contract_version(deps.as_mut().storage, IBC_CLIENT, big_version)?;

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let res = contract::migrate(deps.as_mut(), mock_env(), MigrateMsg {});

            assert_that!(res)
                .is_err()
                .is_equal_to(IcaClientError::Abstract(
                    AbstractError::CannotDowngradeContract {
                        contract: IBC_CLIENT.to_string(),
                        from: big_version.parse().unwrap(),
                        to: version.to_string().parse().unwrap(),
                    },
                ));

            Ok(())
        }

        #[test]
        fn disallow_name_change() -> IcaClientResult<()> {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let old_version = "0.0.0";
            let old_name = "old:contract";
            cw2::set_contract_version(deps.as_mut().storage, old_name, old_version)?;

            let res = contract::migrate(deps.as_mut(), mock_env(), MigrateMsg {});

            assert_that!(res)
                .is_err()
                .is_equal_to(IcaClientError::Abstract(
                    AbstractError::ContractNameMismatch {
                        from: old_name.parse().unwrap(),
                        to: IBC_CLIENT.parse().unwrap(),
                    },
                ));

            Ok(())
        }

        #[test]
        fn works() -> IcaClientResult<()> {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

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

    mod remote_action {
        use super::*;
        use std::str::FromStr;

        use abstract_std::{
            ibc_host::{self, HostAction, InternalAction},
            manager,
            objects::{version_control::VersionControlError, TruncatedChainId},
        };

        use cosmwasm_std::wasm_execute;

        #[test]
        fn throw_when_sender_is_not_proxy() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            deps.querier = mocked_account_querier_builder().build();
            mock_init(deps.as_mut())?;

            let chain_name = TruncatedChainId::from_str(TEST_CHAIN)?;

            let msg = ExecuteMsg::RemoteAction {
                host_chain: chain_name,
                action: HostAction::Dispatch {
                    manager_msgs: vec![manager::ExecuteMsg::UpdateInfo {
                        name: None,
                        description: None,
                        link: None,
                    }],
                },
            };

            let res = execute_as(deps.as_mut(), TEST_MANAGER, msg);

            assert_that!(res).is_err().matches(|e| {
                matches!(
                    e,
                    IcaClientError::VersionControlError(VersionControlError::NotProxy(..))
                )
            });
            Ok(())
        }

        #[test]
        fn cannot_make_internal_call() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            deps.querier = mocked_account_querier_builder().build();
            mock_init(deps.as_mut())?;

            let chain_name = TruncatedChainId::from_str(TEST_CHAIN)?;

            let msg = ExecuteMsg::RemoteAction {
                host_chain: chain_name,
                action: HostAction::Internal(InternalAction::Register {
                    name: String::from("name"),
                    description: None,
                    link: None,
                    base_asset: None,
                    namespace: None,
                    install_modules: vec![],
                }),
            };

            let res = execute_as(deps.as_mut(), TEST_PROXY, msg);

            assert_that!(res)
                .is_err()
                .matches(|e| matches!(e, IcaClientError::ForbiddenInternalCall {}));
            Ok(())
        }

        #[test]
        fn send_packet_with_no_callback() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            deps.querier = mocked_account_querier_builder().build();
            mock_init(deps.as_mut())?;

            let chain_name = TruncatedChainId::from_str(TEST_CHAIN)?;
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
    }

    mod send_funds {
        use super::*;

        use crate::commands::PACKET_LIFETIME;
        use abstract_std::{
            objects::{version_control::VersionControlError, ChannelEntry, TruncatedChainId},
            ICS20,
        };
        use cosmwasm_std::{coins, Binary, CosmosMsg, IbcMsg};
        use prost::Name;
        use std::str::FromStr;

        #[test]
        fn throw_when_sender_is_not_proxy() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            deps.querier = mocked_account_querier_builder().build();
            mock_init(deps.as_mut())?;

            let chain_name = TruncatedChainId::from_str(TEST_CHAIN)?;

            let msg = ExecuteMsg::SendFunds {
                host_chain: chain_name,
                funds: coins(1, "denom"),
                memo: None,
            };

            let res = execute_as(deps.as_mut(), TEST_MANAGER, msg);

            assert_that!(res).is_err().matches(|e| {
                matches!(
                    e,
                    IcaClientError::VersionControlError(VersionControlError::NotProxy(..))
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
            deps.querier = mocked_account_querier_builder().channels(channels).build();
            mock_init(deps.as_mut())?;

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

            let res = execute_as(deps.as_mut(), TEST_PROXY, msg)?;

            let transfer_msgs: Vec<CosmosMsg> = funds
                .into_iter()
                .map(|amount| {
                    IbcMsg::Transfer {
                        channel_id: channel_id.clone(),
                        to_address: remote_addr.clone(),
                        amount,
                        timeout: mock_env().block.time.plus_seconds(PACKET_LIFETIME).into(),
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

            let res = execute_as(deps.as_mut(), TEST_PROXY, msg)?;

            use prost::Message;
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
            ibc_host::{self, HostAction, InternalAction},
            manager,
            objects::{version_control::VersionControlError, TruncatedChainId},
        };
        use cosmwasm_std::wasm_execute;
        use polytone::callbacks::CallbackRequest;
        use std::str::FromStr;

        #[test]
        fn throw_when_sender_is_not_proxy() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            deps.querier = mocked_account_querier_builder().build();
            mock_init(deps.as_mut())?;

            let chain_name = TruncatedChainId::from_str(TEST_CHAIN)?;

            let msg = ExecuteMsg::Register {
                host_chain: chain_name,
                base_asset: None,
                namespace: None,
                install_modules: vec![],
            };

            let res = execute_as(deps.as_mut(), TEST_MANAGER, msg);

            assert_that!(res).is_err().matches(|e| {
                matches!(
                    e,
                    IcaClientError::VersionControlError(VersionControlError::NotProxy(..))
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

            let chain_name = TruncatedChainId::from_str(TEST_CHAIN)?;
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
                host_chain: chain_name,
                base_asset: None,
                namespace: None,
                install_modules: vec![],
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
                                base_asset: None,
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
                    &TruncatedChainId::from_str("channel")?,
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
}
