use abstract_std::objects::{account::AccountTrace, AccountId, TruncatedChainId};
// We need to rewrite this because cosmrs::Msg is not implemented for IBC types
use abstract_interface::{Abstract, AbstractAccount, AccountDetails, ManagerQueryFns};
use anyhow::Result as AnyResult;
use cw_orch::{environment::Environment, prelude::*};
use cw_orch_interchain::prelude::*;

pub const TEST_ACCOUNT_NAME: &str = "account-test";
pub const TEST_ACCOUNT_DESCRIPTION: &str = "Description of an account";
pub const TEST_ACCOUNT_LINK: &str = "https://google.com";

pub fn create_test_remote_account<Chain: IbcQueryHandler, IBC: InterchainEnv<Chain>>(
    abstr_origin: &Abstract<Chain>,
    origin_id: &str,
    remote_id: &str,
    interchain: &IBC,
    funds: Option<Vec<Coin>>,
) -> AnyResult<(AbstractAccount<Chain>, AccountId)> {
    let origin_name = TruncatedChainId::from_chain_id(origin_id);
    let remote_name = TruncatedChainId::from_chain_id(remote_id);

    // Create a local account for testing
    let account_name = TEST_ACCOUNT_NAME.to_string();
    let description = Some(TEST_ACCOUNT_DESCRIPTION.to_string());
    let link = Some(TEST_ACCOUNT_LINK.to_string());
    let origin_account = abstr_origin.account_factory.create_new_account(
        AccountDetails {
            name: account_name.clone(),
            description: description.clone(),
            link: link.clone(),
            base_asset: None,
            install_modules: vec![],
            namespace: None,
            account_id: None,
        },
        abstract_std::objects::gov_type::GovernanceDetails::Monarchy {
            monarch: abstr_origin
                .version_control
                .environment()
                .sender_addr()
                .to_string(),
        },
        funds.as_deref(),
    )?;

    // We need to enable ibc on the account.
    origin_account.manager.set_ibc_status(true)?;

    // Now we send a message to the client saying that we want to create an account on the
    // destination chain
    let register_tx = origin_account.register_remote_account(remote_name)?;

    interchain.await_and_check_packets(origin_id, register_tx)?;

    // After this is all ended, we return the account id of the account we just created on the remote chain
    let account_config = origin_account.manager.config()?;
    let remote_account_id = AccountId::new(
        account_config.account_id.seq(),
        AccountTrace::Remote(vec![origin_name]),
    )?;

    Ok((origin_account, remote_account_id))
}

#[cfg(test)]
mod test {
    #![allow(clippy::needless_borrows_for_generic_args)]
    use super::*;
    use crate::{
        setup::{ibc_abstract_setup, mock_test::logger_test_init},
        JUNO, OSMOSIS, STARGAZE,
    };

    use abstract_interface::{AccountFactoryExecFns, ManagerExecFns};
    use abstract_std::{
        ans_host::ExecuteMsgFns as AnsExecuteMsgFns,
        ibc_client::AccountResponse,
        ibc_host::{
            ExecuteMsg as HostExecuteMsg, ExecuteMsgFns, HelperAction, HostAction, InternalAction,
        },
        manager::{
            state::AccountInfo, ConfigResponse, ExecuteMsg as ManagerExecuteMsg, InfoResponse,
        },
        objects::{gov_type::GovernanceDetails, UncheckedChannelEntry},
        proxy, IBC_CLIENT, ICS20, PROXY,
    };

    use anyhow::Result as AnyResult;
    use cosmwasm_std::{coins, to_json_binary, wasm_execute, IbcTimeout, Uint128};
    use cw_orch::{environment::Environment, mock::cw_multi_test::AppResponse};
    use ibc_relayer_types::core::ics24_host::identifier::PortId;

    #[test]
    fn ibc_account_action() -> AnyResult<()> {
        logger_test_init();
        let mock_interchain =
            MockBech32InterchainEnv::new(vec![(JUNO, "juno"), (STARGAZE, "stargaze")]);

        // We just verified all steps pass
        let (abstr_origin, abstr_remote) = ibc_abstract_setup(&mock_interchain, JUNO, STARGAZE)?;

        let remote_name = TruncatedChainId::from_chain_id(STARGAZE);

        let (origin_account, remote_account_id) =
            create_test_remote_account(&abstr_origin, JUNO, STARGAZE, &mock_interchain, None)?;

        let new_name = "Funky Crazy Name";
        let new_description = "Funky new account with wonderful capabilities";
        let new_link = "https://abstract.money";

        // The user on origin chain wants to change the account description
        let ibc_action_result = origin_account.manager.execute_on_remote(
            remote_name,
            ManagerExecuteMsg::UpdateInfo {
                name: Some(new_name.to_string()),
                description: Some(new_description.to_string()),
                link: Some(new_link.to_string()),
            },
        )?;

        mock_interchain.await_and_check_packets(JUNO, ibc_action_result)?;

        // We check the account description changed on chain 2
        let remote_abstract_account =
            AbstractAccount::new(&abstr_remote, remote_account_id.clone());

        let account_info = remote_abstract_account.manager.info()?;

        assert_eq!(account_info.info.name, new_name.to_string());
        assert_eq!(
            account_info.info.description,
            Some(new_description.to_string())
        );
        assert_eq!(account_info.info.link, Some(new_link.to_string()));

        // Verify that remote account has been saved correctly.
        let account_response: AccountResponse =
            abstr_origin
                .ibc
                .client
                .query(&abstract_std::ibc_client::QueryMsg::Account {
                    chain_name: TruncatedChainId::from_chain_id(STARGAZE),
                    account_id: AccountId::new(1, AccountTrace::Local)?,
                })?;

        assert_eq!(
            AccountResponse {
                remote_proxy_addr: Some(remote_abstract_account.proxy.address()?.to_string()),
            },
            account_response
        );

        Ok(())
    }

    #[test]
    fn ibc_stargate_action() -> AnyResult<()> {
        logger_test_init();
        let mock_interchain =
            MockBech32InterchainEnv::new(vec![(JUNO, "juno"), (STARGAZE, "stargaze")]);

        // We just verified all steps pass
        let (abstr_origin, abstr_remote) = ibc_abstract_setup(&mock_interchain, JUNO, STARGAZE)?;

        let (origin_account, remote_account_id) =
            create_test_remote_account(&abstr_origin, JUNO, STARGAZE, &mock_interchain, None)?;
        let remote_abstract_account =
            AbstractAccount::new(&abstr_remote, remote_account_id.clone());

        // Do stargate action on proxy to verify it's enabled
        let amount = Coin {
            denom: "ujuno".to_owned(),
            amount: Uint128::new(100),
        };
        mock_interchain
            .get_chain(JUNO)
            .unwrap()
            .add_balance(&origin_account.proxy.address()?, vec![amount.clone()])?;
        let interchain_channel = mock_interchain.create_channel(
            JUNO,
            STARGAZE,
            &PortId::transfer(),
            &PortId::transfer(),
            "ics20-1",
            None, // Unordered channel
        )?;

        // The user on origin chain wants to change the account description
        let ibc_transfer_result = origin_account.manager.execute_on_module(
            PROXY,
            &proxy::ExecuteMsg::ModuleAction {
                msgs: vec![cosmwasm_std::CosmosMsg::Ibc(
                    cosmwasm_std::IbcMsg::Transfer {
                        channel_id: interchain_channel
                            .interchain_channel
                            .port_a
                            .channel
                            .unwrap()
                            .to_string(),
                        to_address: remote_abstract_account.proxy.address()?.to_string(),
                        amount,
                        timeout: IbcTimeout::with_timestamp(
                            mock_interchain
                                .get_chain(JUNO)
                                .unwrap()
                                .block_info()
                                .unwrap()
                                .time
                                .plus_days(1),
                        ),
                    },
                )],
            },
        )?;

        mock_interchain.await_and_check_packets(JUNO, ibc_transfer_result)?;

        let remote_proxy_balance = mock_interchain
            .get_chain(STARGAZE)
            .unwrap()
            .balance(remote_abstract_account.proxy.address()?, None)?;
        assert_eq!(remote_proxy_balance, coins(100, "ibc/channel-0/ujuno"));

        Ok(())
    }

    #[test]
    fn test_multi_hop_account_creation() -> AnyResult<()> {
        logger_test_init();
        let mock_interchain = MockBech32InterchainEnv::new(vec![
            (JUNO, "juno"),
            (STARGAZE, "stargaze"),
            (OSMOSIS, "osmosis"),
        ]);

        // SETUP
        let chain1 = mock_interchain.get_chain(JUNO).unwrap();
        let chain2 = mock_interchain.get_chain(STARGAZE).unwrap();
        let chain3 = mock_interchain.get_chain(OSMOSIS).unwrap();

        // Deploying abstract and the IBC abstract logic
        let abstr_origin = Abstract::deploy_on(chain1.clone(), chain1.sender_addr().to_string())?;
        let abstr_intermediate_remote =
            Abstract::deploy_on(chain2.clone(), chain2.sender_addr().to_string())?;
        let abstr_destination_remote =
            Abstract::deploy_on(chain3.clone(), chain3.sender_addr().to_string())?;

        // Creating a connection between 2 abstract deployments
        abstr_origin.connect_to(&abstr_intermediate_remote, &mock_interchain)?;
        abstr_intermediate_remote.connect_to(&abstr_destination_remote, &mock_interchain)?;
        // END SETUP

        // Create a local account for testing
        let account_name = TEST_ACCOUNT_NAME.to_string();
        let description = Some(TEST_ACCOUNT_DESCRIPTION.to_string());
        let link = Some(TEST_ACCOUNT_LINK.to_string());
        let origin_account: AbstractAccount<MockBech32> =
            abstr_origin.account_factory.create_new_account(
                AccountDetails {
                    name: account_name.clone(),
                    description: description.clone(),
                    link: link.clone(),
                    base_asset: None,
                    install_modules: vec![],
                    namespace: None,
                    account_id: None,
                },
                abstract_std::objects::gov_type::GovernanceDetails::Monarchy {
                    monarch: abstr_origin
                        .version_control
                        .environment()
                        .sender_addr()
                        .to_string(),
                },
                None,
            )?;

        // We need to enable ibc on the account.
        origin_account.manager.set_ibc_status(true)?;

        // Now we send a message to the client saying that we want to create an account on the
        // destination chain
        let register_tx =
            origin_account.register_remote_account(TruncatedChainId::from_chain_id(STARGAZE))?;

        mock_interchain.await_and_check_packets(JUNO, register_tx)?;

        // Create account from JUNO on OSMOSIS by going through STARGAZE
        let create_account_remote_tx = origin_account.manager.execute_on_remote_module(
            TruncatedChainId::from_chain_id(STARGAZE),
            PROXY,
            to_json_binary(&abstract_std::proxy::ExecuteMsg::IbcAction {
                msg: abstract_std::ibc_client::ExecuteMsg::Register {
                    host_chain: TruncatedChainId::from_chain_id(OSMOSIS),
                    base_asset: None,
                    namespace: None,
                    install_modules: vec![],
                },
            })?,
        )?;

        mock_interchain.await_and_check_packets(JUNO, create_account_remote_tx)?;

        let destination_remote_account_id = AccountId::new(
            origin_account.manager.config()?.account_id.seq(),
            AccountTrace::Remote(vec![
                TruncatedChainId::from_chain_id(JUNO),
                TruncatedChainId::from_chain_id(STARGAZE),
            ]),
        )?;

        let destination_remote_account = AbstractAccount::new(
            &abstr_destination_remote,
            destination_remote_account_id.clone(),
        );

        let manager_config = destination_remote_account.manager.config()?;
        assert_eq!(
            manager_config,
            ConfigResponse {
                account_id: destination_remote_account_id,
                is_suspended: false,
                module_factory_address: abstr_destination_remote.module_factory.address()?,
                version_control_address: abstr_destination_remote.version_control.address()?,
            }
        );

        Ok(())
    }

    #[test]
    fn test_create_ibc_account() -> AnyResult<()> {
        logger_test_init();

        let mock_interchain =
            MockBech32InterchainEnv::new(vec![(JUNO, "juno"), (STARGAZE, "stargaze")]);

        // We just verified all steps pass
        let (abstr_origin, abstr_remote) = ibc_abstract_setup(&mock_interchain, JUNO, STARGAZE)?;

        let (origin_account, remote_account_id) =
            create_test_remote_account(&abstr_origin, JUNO, STARGAZE, &mock_interchain, None)?;

        // We assert the account was created with the right properties
        let remote_abstract_account =
            AbstractAccount::new(&abstr_remote, remote_account_id.clone());
        let manager_config = remote_abstract_account.manager.config()?;
        assert_eq!(
            manager_config,
            ConfigResponse {
                account_id: remote_account_id,
                is_suspended: false,
                module_factory_address: abstr_remote.module_factory.address()?,
                version_control_address: abstr_remote.version_control.address()?,
            }
        );

        let manager_info = remote_abstract_account.manager.info()?;

        let account_name = TEST_ACCOUNT_NAME.to_string();
        let description = Some(TEST_ACCOUNT_DESCRIPTION.to_string());
        let link = Some(TEST_ACCOUNT_LINK.to_string());
        assert_eq!(
            manager_info,
            InfoResponse {
                info: abstract_std::manager::state::AccountInfo {
                    name: account_name,
                    chain_id: STARGAZE.to_string(),
                    description,
                    link
                }
            }
        );
        // We make sure the ibc client is installed on the remote account
        let installed_remote_modules = remote_abstract_account.manager.module_infos(None, None)?;
        assert!(installed_remote_modules
            .module_infos
            .iter()
            .any(|m| m.id == IBC_CLIENT));

        // We try to execute a message from the proxy contract (account creation for instance)

        // ii. Now we test that we can indeed create an account remotely from the interchain account
        let account_name = String::from("Abstract Test Remote Remote account");
        let create_account_remote_tx = origin_account.manager.execute_on_remote_module(
            TruncatedChainId::from_chain_id(STARGAZE),
            PROXY,
            to_json_binary(&abstract_std::proxy::ExecuteMsg::ModuleAction {
                msgs: vec![wasm_execute(
                    abstr_remote.account_factory.address()?,
                    &abstract_std::account_factory::ExecuteMsg::CreateAccount {
                        governance: GovernanceDetails::Monarchy {
                            monarch: abstr_remote.version_control.address()?.to_string(),
                        },
                        name: account_name.clone(),
                        description: None,
                        link: None,
                        account_id: None,
                        base_asset: None,
                        namespace: None,
                        install_modules: vec![],
                    },
                    vec![],
                )?
                .into()],
            })?,
        )?;

        // The create remote account tx is passed ?
        mock_interchain.await_and_check_packets(JUNO, create_account_remote_tx)?;

        // Can get the account from stargaze.
        let created_account_id = AccountId::new(1, AccountTrace::Local)?;

        let created_abstract_account = AbstractAccount::new(&abstr_remote, created_account_id);

        let account_info: AccountInfo = created_abstract_account.manager.info()?.info;

        assert_eq!(
            AccountInfo {
                chain_id: STARGAZE.to_owned(),
                description: None,
                name: account_name,
                link: None,
            },
            account_info.into()
        );

        Ok(())
    }

    #[test]
    fn test_cannot_create_remote_account_when_caller_is_not_host() -> AnyResult<()> {
        let mock_interchain =
            MockBech32InterchainEnv::new(vec![(JUNO, "juno"), (STARGAZE, "stargaze")]);
        let stargaze = mock_interchain.get_chain(STARGAZE)?;

        let (_abstr_origin, abstr_remote) = ibc_abstract_setup(&mock_interchain, JUNO, STARGAZE)?;

        try_create_remote_account(&abstr_remote, &stargaze.addr_make("user")).unwrap_err();

        try_create_remote_account(&abstr_remote, &abstr_remote.ibc.host.address()?)?;

        Ok(())
    }

    fn try_create_remote_account(
        abstr: &Abstract<MockBech32>,
        sender: &Addr,
    ) -> AnyResult<AppResponse> {
        Ok(abstr.account_factory.call_as(sender).create_account(
            GovernanceDetails::Monarchy {
                monarch: abstr
                    .account_factory
                    .environment()
                    .addr_make("user")
                    .to_string(),
            },
            vec![],
            String::from("name"),
            Some(AccountId::new(
                2,
                AccountTrace::Remote(vec![TruncatedChainId::from_chain_id(JUNO)]),
            )?),
            None,
            None,
            None,
            None,
            &[],
        )?)
    }

    #[test]
    fn test_cannot_call_remote_manager_from_non_host_account() -> AnyResult<()> {
        logger_test_init();
        let mock_interchain =
            MockBech32InterchainEnv::new(vec![(JUNO, "juno"), (STARGAZE, "stargaze")]);

        // We just verified all steps pass
        let (abstr_juno, abstr_stargaze) = ibc_abstract_setup(&mock_interchain, JUNO, STARGAZE)?;

        let (_origin_account, remote_account_id) =
            create_test_remote_account(&abstr_juno, JUNO, STARGAZE, &mock_interchain, None)?;

        let remote_account = AbstractAccount::new(&abstr_stargaze, remote_account_id);

        let new_name = String::from("Funky Crazy Name");
        let new_description = String::from("Funky new account with wonderful capabilities");
        let new_link = String::from("https://abstract.money");

        // Cannot call with sender that is not host.
        let result = remote_account
            .manager
            .call_as(&mock_interchain.get_chain(STARGAZE)?.sender_addr())
            .update_info(
                Some(new_description.clone()),
                Some(new_link.clone()),
                Some(new_name.clone()),
            );

        assert!(result.is_err());

        // Can call with host.
        let result = remote_account
            .manager
            .call_as(&abstr_stargaze.ibc.host.address()?)
            .update_info(Some(new_description), Some(new_link), Some(new_name));

        assert!(result.is_ok());

        Ok(())
    }

    #[test]
    fn test_cannot_call_ibc_host_directly_with_remove_chain_proxy() -> AnyResult<()> {
        logger_test_init();
        let mock_interchain =
            MockBech32InterchainEnv::new(vec![(JUNO, "juno"), (STARGAZE, "stargaze")]);

        // We just verified all steps pass
        let (_abstr_origin, abstr_remote) = ibc_abstract_setup(&mock_interchain, JUNO, STARGAZE)?;

        let result = abstr_remote
            .ibc
            .host
            .call_as(&Addr::unchecked("rando"))
            .remove_chain_proxy(TruncatedChainId::from_chain_id(STARGAZE));

        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_cannot_call_ibc_host_directly_with_register_chain_proxy() -> AnyResult<()> {
        logger_test_init();
        let mock_interchain =
            MockBech32InterchainEnv::new(vec![(JUNO, "juno"), (STARGAZE, "stargaze")]);

        // We just verified all steps pass
        let (_abstr_origin, abstr_remote) = ibc_abstract_setup(&mock_interchain, JUNO, STARGAZE)?;

        let result = abstr_remote
            .ibc
            .host
            .call_as(&Addr::unchecked("rando"))
            .register_chain_proxy(TruncatedChainId::from_chain_id(OSMOSIS), PROXY.to_owned());
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_cannot_call_ibc_host_directly_with_dispatch_action() -> AnyResult<()> {
        logger_test_init();
        let mock_interchain =
            MockBech32InterchainEnv::new(vec![(JUNO, "juno"), (STARGAZE, "stargaze")]);

        // We just verified all steps pass
        let (abstr_origin, abstr_remote) = ibc_abstract_setup(&mock_interchain, JUNO, STARGAZE)?;

        let (origin_account, remote_account_id) =
            create_test_remote_account(&abstr_origin, JUNO, STARGAZE, &mock_interchain, None)?;

        let result = abstr_remote.ibc.host.execute(
            &HostExecuteMsg::Execute {
                account_id: remote_account_id,
                proxy_address: origin_account.proxy.address()?.to_string(),
                action: HostAction::Dispatch {
                    manager_msgs: vec![ManagerExecuteMsg::UpdateInfo {
                        name: Some("name".to_owned()),
                        description: Some("description".to_owned()),
                        link: Some("link".to_owned()),
                    }],
                },
            },
            None,
        );

        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_cannot_call_ibc_host_directly_with_internal_action() -> AnyResult<()> {
        logger_test_init();
        let mock_interchain =
            MockBech32InterchainEnv::new(vec![(JUNO, "juno"), (STARGAZE, "stargaze")]);

        // We just verified all steps pass
        let (abstr_origin, abstr_remote) = ibc_abstract_setup(&mock_interchain, JUNO, STARGAZE)?;

        let (origin_account, remote_account_id) =
            create_test_remote_account(&abstr_origin, JUNO, STARGAZE, &mock_interchain, None)?;

        let result = abstr_remote.ibc.host.execute(
            &HostExecuteMsg::Execute {
                account_id: remote_account_id,
                proxy_address: origin_account.proxy.address()?.to_string(),
                action: HostAction::Internal(InternalAction::Register {
                    name: "name".to_owned(),
                    description: None,
                    link: None,
                    base_asset: None,
                    namespace: None,
                    install_modules: vec![],
                }),
            },
            None,
        );

        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_cannot_call_ibc_host_directly_with_helper_action() -> AnyResult<()> {
        logger_test_init();
        let mock_interchain =
            MockBech32InterchainEnv::new(vec![(JUNO, "juno"), (STARGAZE, "stargaze")]);

        // We just verified all steps pass
        let (abstr_origin, abstr_remote) = ibc_abstract_setup(&mock_interchain, JUNO, STARGAZE)?;

        let (origin_account, remote_account_id) =
            create_test_remote_account(&abstr_origin, JUNO, STARGAZE, &mock_interchain, None)?;

        let result = abstr_remote.ibc.host.execute(
            &HostExecuteMsg::Execute {
                account_id: remote_account_id,
                proxy_address: origin_account.proxy.address()?.to_string(),
                action: HostAction::Helpers(HelperAction::SendAllBack),
            },
            None,
        );

        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_send_all_back() -> AnyResult<()> {
        logger_test_init();
        let mock_interchain =
            MockBech32InterchainEnv::new(vec![(JUNO, "juno"), (STARGAZE, "stargaze")]);
        let origin_denom = "ujuno";
        let remote_denom: &str = &format!("ibc/channel-0/{}", origin_denom);

        // We just verified all steps pass
        let (abstr_origin, abstr_remote) = ibc_abstract_setup(&mock_interchain, JUNO, STARGAZE)?;

        mock_interchain.get_chain(JUNO)?.set_balance(
            &abstr_origin.version_control.environment().sender_addr(),
            coins(100, origin_denom),
        )?;
        let (origin_account, remote_account_id) = create_test_remote_account(
            &abstr_origin,
            JUNO,
            STARGAZE,
            &mock_interchain,
            Some(coins(10, origin_denom)),
        )?;

        //let interchain_channel = create_transfer_channel(JUNO, STARGAZE, &mock_interchain)?;

        let interchain_channel = mock_interchain.create_channel(
            JUNO,
            STARGAZE,
            &PortId::transfer(),
            &PortId::transfer(),
            "ics20-1",
            None, // Unordered channel
        )?;

        abstr_origin.ans_host.update_channels(
            vec![(
                UncheckedChannelEntry {
                    connected_chain: "stargaze".to_string(),
                    protocol: ICS20.to_string(),
                },
                interchain_channel
                    .interchain_channel
                    .get_chain(JUNO)?
                    .channel
                    .unwrap()
                    .to_string(),
            )],
            vec![],
        )?;

        abstr_remote.ans_host.update_channels(
            vec![(
                UncheckedChannelEntry {
                    connected_chain: "juno".to_string(),
                    protocol: ICS20.to_string(),
                },
                interchain_channel
                    .interchain_channel
                    .get_chain(STARGAZE)?
                    .channel
                    .unwrap()
                    .to_string(),
            )],
            vec![],
        )?;

        // Verify origin balance before sending funds.
        let origin_balance = mock_interchain
            .get_chain(JUNO)?
            .query_balance(&origin_account.proxy.address()?, origin_denom)?;
        assert_eq!(Uint128::from(10u128), origin_balance);

        // Send funds from juno to stargaze.
        let send_funds_tx = origin_account.manager.execute_on_module(
            PROXY,
            abstract_std::proxy::ExecuteMsg::IbcAction {
                msg: abstract_std::ibc_client::ExecuteMsg::SendFunds {
                    funds: coins(10, origin_denom),
                    host_chain: TruncatedChainId::from_chain_id(STARGAZE),
                    memo: None,
                    receiver: None,
                },
            },
        )?;

        mock_interchain.await_and_check_packets(JUNO, send_funds_tx)?;

        // Verify local balance after sending funds.
        let origin_balance = mock_interchain
            .get_chain(JUNO)?
            .query_balance(&origin_account.proxy.address()?, origin_denom)?;
        assert!(origin_balance.is_zero());

        let remote_account = AbstractAccount::new(&abstr_remote, remote_account_id.clone());

        // Check balance on remote chain.
        let remote_balance = mock_interchain
            .get_chain(STARGAZE)?
            .query_balance(&remote_account.proxy.address()?, remote_denom)?;
        assert_eq!(Uint128::from(10u128), remote_balance);

        // Send all back.
        let send_funds_back_tx = origin_account
            .manager
            .send_all_funds_back(TruncatedChainId::from_chain_id(STARGAZE))?;

        mock_interchain.await_and_check_packets(JUNO, send_funds_back_tx)?;

        // Check balance on remote chain.
        let remote_balance = mock_interchain
            .get_chain(STARGAZE)?
            .query_balance(&remote_account.proxy.address()?, remote_denom)?;
        assert!(remote_balance.is_zero());

        // Check balance on local chain.
        let origin_balance = mock_interchain
            .get_chain(JUNO)?
            .query_balance(&origin_account.proxy.address()?, origin_denom)?;
        assert_eq!(Uint128::from(10u128), origin_balance);

        Ok(())
    }
}
