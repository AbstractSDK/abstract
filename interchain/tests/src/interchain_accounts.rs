use std::str::FromStr;

use abstract_core::{
    objects::{account::AccountTrace, chain_name::ChainName, AccountId},
    IBC_CLIENT,
};
// We need to rewrite this because cosmrs::Msg is not implemented for IBC types

use abstract_interface::{Abstract, AccountDetails, ManagerQueryFns};
use anyhow::Result as AnyResult;
use cosmwasm_std::Empty;
use cw_orch::prelude::*;

pub const TEST_ACCOUNT_NAME: &str = "account-test";
pub const TEST_ACCOUNT_DESCRIPTION: &str = "Description of the account";
pub const TEST_ACCOUNT_LINK: &str = "https://google.com";

pub fn set_env() {
    std::env::set_var("STATE_FILE", "daemon_state.json"); // Set in code for tests
    std::env::set_var("ARTIFACTS_DIR", "../artifacts"); // Set in code for tests
}

pub fn create_test_remote_account<Chain: IbcQueryHandler, IBC: InterchainEnv<Chain>>(
    origin: &Abstract<Chain>,
    origin_id: &str,
    destination_id: &str,
    interchain: &IBC,
) -> AnyResult<AccountId> {
    let origin_name = ChainName::from_chain_id(origin_id).to_string();
    let destination_name = ChainName::from_chain_id(destination_id).to_string();

    // Create a local account for testing
    let account_name = TEST_ACCOUNT_NAME.to_string();
    let description = Some(TEST_ACCOUNT_DESCRIPTION.to_string());
    let link = Some(TEST_ACCOUNT_LINK.to_string());
    origin.account_factory.create_new_account(
        AccountDetails {
            name: account_name.clone(),
            description: description.clone(),
            link: link.clone(),
            base_asset: None,
            install_modules: vec![],
            namespace: None,
        },
        abstract_core::objects::gov_type::GovernanceDetails::Monarchy {
            monarch: origin.account.manager.get_chain().sender().to_string(),
        },
        None,
    )?;

    // We need to register the ibc client as a module of the manager (account specific)
    origin
        .account
        .manager
        .install_module::<Empty>(IBC_CLIENT, None, None)?;

    // Now we send a message to the client saying that we want to create an account on the
    // destination chain
    let register_tx = origin.account.register_remote_account(&destination_name)?;

    interchain.wait_ibc(&origin_id.to_owned(), register_tx)?;

    // After this is all ended, we return the account id of the account we just created on the remote chain
    let account_config = origin.account.manager.config()?;

    Ok(AccountId::new(
        account_config.account_id.seq(),
        AccountTrace::Remote(vec![ChainName::from_str(&origin_name)?]),
    )?)
}

#[cfg(test)]
mod test {

    use abstract_core::ibc_host::ExecuteMsg as HostExecuteMsg;
    use abstract_core::ibc_host::ExecuteMsgFns;
    use abstract_core::ibc_host::{HostAction, InternalAction};
    use abstract_core::manager::InfoResponse;
    use abstract_core::objects::gov_type::GovernanceDetails;
    use abstract_core::IBC_HOST;

    use abstract_core::{manager::ConfigResponse, PROXY};
    use abstract_interface::{Manager, ManagerExecFns, ManagerQueryFns};
    use cosmwasm_std::{to_json_binary, wasm_execute};

    use anyhow::Result as AnyResult;

    use super::*;
    use crate::interchain_accounts::create_test_remote_account;
    use crate::setup::ibc_abstract_setup;

    use crate::setup::mock_test::logger_test_init;
    use crate::JUNO;
    use crate::OSMOSIS;
    use crate::STARGAZE;

    use abstract_core::manager::ModuleInstallConfig;
    use abstract_core::objects::module::ModuleInfo;
    use abstract_core::objects::module::ModuleVersion;
    use abstract_core::{
        manager::ExecuteMsg as ManagerExecuteMsg,
        objects::{chain_name::ChainName, AccountId},
    };
    use abstract_interface::VCQueryFns;
    use abstract_scripts::abstract_ibc::abstract_ibc_connection_with;
    use cosmwasm_std::Addr;
    use cw_orch::prelude::ContractInstance;
    use cw_orch_polytone::Polytone;
    use polytone::handshake::POLYTONE_VERSION;

    #[test]
    fn ibc_account_action() -> AnyResult<()> {
        logger_test_init();
        let sender = Addr::unchecked("sender");
        let mock_interchain = MockInterchainEnv::new(vec![(JUNO, &sender), (STARGAZE, &sender)]);

        // We just verified all steps pass
        let (abstr1, abstr2) = ibc_abstract_setup(&mock_interchain, JUNO, STARGAZE)?;

        let remote_name = ChainName::from_chain_id(STARGAZE).to_string();

        let remote_account_id =
            create_test_remote_account(&abstr1, JUNO, STARGAZE, &mock_interchain)?;

        let new_name = "Funky Crazy Name";
        let new_description = "Funky new account with wonderful capabilities";
        let new_link = "https://abstract.money";

        // Ad client to account

        // The user on chain 1 want to change the account description

        let ibc_action_result = abstr1.account.manager.execute_on_remote(
            &remote_name,
            ManagerExecuteMsg::UpdateInfo {
                name: Some(new_name.to_string()),
                description: Some(new_description.to_string()),
                link: Some(new_link.to_string()),
            },
            None,
        )?;

        mock_interchain.wait_ibc(&JUNO.to_string(), ibc_action_result)?;

        // We check the account description changed on chain 2
        let remote_account = abstr2.version_control.account_base(remote_account_id)?;

        let manager = remote_account.account_base.manager;
        abstr2.account.manager.set_address(&manager);

        let account_info = abstr2.account.manager.info()?;

        assert_eq!(account_info.info.name, new_name.to_string());
        assert_eq!(
            account_info.info.description,
            Some(new_description.to_string())
        );
        assert_eq!(account_info.info.link, Some(new_link.to_string()));

        Ok(())
    }

    #[test]
    fn test_multi_hop_account_creation() -> AnyResult<()> {
        logger_test_init();
        let sender = Addr::unchecked("sender");
        let mock_interchain = MockInterchainEnv::new(vec![
            (JUNO, &sender),
            (STARGAZE, &sender),
            (OSMOSIS, &sender),
        ]);

        // SETUP
        let chain1 = mock_interchain.chain(JUNO).unwrap();
        let chain2 = mock_interchain.chain(STARGAZE).unwrap();
        let chain3 = mock_interchain.chain(OSMOSIS).unwrap();

        // Deploying abstract and the IBC abstract logic
        let origin = Abstract::deploy_on(chain1.clone(), chain1.sender().to_string())?;
        let intermediate = Abstract::deploy_on(chain2.clone(), chain2.sender().to_string())?;
        let destination = Abstract::deploy_on(chain3.clone(), chain3.sender().to_string())?;

        // Deploying polytone on both chains
        let polytone_1 = Polytone::deploy_on(chain1.clone(), None)?;
        let polytone_2 = Polytone::deploy_on(chain2.clone(), None)?;
        let polytone_3 = Polytone::deploy_on(chain3.clone(), None)?;

        // Creating a connection between 2 polytone deployments
        let res = mock_interchain.create_contract_channel(
            &polytone_1.note,
            &polytone_2.voice,
            POLYTONE_VERSION,
        )?;

        println!("Channel between 1 and 2 {:?}", res.interchain_channel);

        let res = mock_interchain.create_contract_channel(
            &polytone_2.note,
            &polytone_3.voice,
            POLYTONE_VERSION,
        )?;

        println!("Channel between 2 and 3 {:?}", res.interchain_channel);

        // Create the connection between client and host
        abstract_ibc_connection_with(&origin, &mock_interchain, &intermediate, &polytone_1)?;
        abstract_ibc_connection_with(&intermediate, &mock_interchain, &destination, &polytone_2)?;

        // END SETUP

        // Create a local account for testing
        let account_name = TEST_ACCOUNT_NAME.to_string();
        let description = Some(TEST_ACCOUNT_DESCRIPTION.to_string());
        let link = Some(TEST_ACCOUNT_LINK.to_string());
        origin.account_factory.create_new_account(
            AccountDetails {
                name: account_name.clone(),
                description: description.clone(),
                link: link.clone(),
                base_asset: None,
                install_modules: vec![],
                namespace: None,
            },
            abstract_core::objects::gov_type::GovernanceDetails::Monarchy {
                monarch: origin.account.manager.get_chain().sender().to_string(),
            },
            None,
        )?;

        println!("Created test account");

        // We need to register the ibc client as a module of the manager (account specific)
        origin
            .account
            .manager
            .install_module::<Empty>(IBC_CLIENT, None, None)?;

        // Now we send a message to the client saying that we want to create an account on the
        // destination chain
        let register_tx = origin
            .account
            .register_remote_account(&ChainName::from_chain_id(STARGAZE).to_string())?;

        mock_interchain.wait_ibc(&JUNO.to_owned(), register_tx)?;

        // TODO: Install module using IBC on the created account.
        let register_module_tx = origin.account.manager.execute_on_remote(
            &ChainName::from_chain_id(STARGAZE).to_string(),
            ManagerExecuteMsg::InstallModules {
                modules: vec![ModuleInstallConfig::new(
                    ModuleInfo::from_id(IBC_CLIENT, ModuleVersion::Latest)?,
                    None,
                )],
            },
            None,
        )?;

        mock_interchain.wait_ibc(&JUNO.to_owned(), register_module_tx)?;

        // Create account from JUNO on OSMOSIS by going through STARGAZE
        let create_account_remote_tx = origin.account.manager.execute_on_remote_module(
            &ChainName::from_chain_id(STARGAZE).to_string(),
            PROXY,
            to_json_binary(&abstract_core::proxy::ExecuteMsg::IbcAction {
                msgs: vec![abstract_core::ibc_client::ExecuteMsg::Register {
                    host_chain: ChainName::from_chain_id(OSMOSIS).to_string(),
                    base_asset: None,
                    namespace: None,
                    install_modules: vec![],
                }],
            })?,
            None,
        )?;
        println!("OSMOSIS host_addr: {:?}", destination.ibc.host.addr_str());

        println!("Begin waiting");
        mock_interchain.wait_ibc(&JUNO.to_owned(), create_account_remote_tx)?;

        println!("Done waiting");

        let destination_account_id = AccountId::new(
            origin.account.manager.config()?.account_id.seq(),
            AccountTrace::Remote(vec![
                ChainName::from_chain_id(JUNO),
                ChainName::from_chain_id(STARGAZE),
            ]),
        )?;

        let remote_account_config = destination
            .version_control
            .get_account(destination_account_id.clone())?;
        // This shouldn't fail as we have just created an account using those characteristics
        /*log::info!("Remote account config {:?} ", remote_account_config);

        let remote_manager = Manager::new(
            "remote_account_manager",
            mock_interchain.chain(OSMOSIS)?.clone(),
        );
        remote_manager.set_address(&remote_account_config.manager);

        // Now we need to test some things about this account on the juno chain
        let manager_config = remote_manager.config()?;
        assert_eq!(
            manager_config,
            ConfigResponse {
                account_id: destination_account_id,
                is_suspended: false,
                module_factory_address: destination.module_factory.address()?,
                version_control_address: destination.version_control.address()?,
            }
        );*/

        Ok(())
    }

    #[test]
    fn test_create_ibc_account() -> AnyResult<()> {
        logger_test_init();

        let sender = Addr::unchecked("sender");
        let mock_interchain = MockInterchainEnv::new(vec![(JUNO, &sender), (STARGAZE, &sender)]);

        // We just verified all steps pass
        let (abstr_juno, abstr_stargaze) = ibc_abstract_setup(&mock_interchain, JUNO, STARGAZE)?;

        let remote_account =
            create_test_remote_account(&abstr_juno, JUNO, STARGAZE, &mock_interchain)?;

        let remote_account_config = abstr_stargaze
            .version_control
            .get_account(remote_account.clone())?;
        // This shouldn't fail as we have just created an account using those characteristics
        log::info!("Remote account config {:?} ", remote_account_config);

        let remote_manager = Manager::new(
            "remote_account_manager",
            mock_interchain.chain(STARGAZE)?.clone(),
        );
        remote_manager.set_address(&remote_account_config.manager);

        // Now we need to test some things about this account on the juno chain
        let manager_config = remote_manager.config()?;
        assert_eq!(
            manager_config,
            ConfigResponse {
                account_id: remote_account,
                is_suspended: false,
                module_factory_address: abstr_stargaze.module_factory.address()?,
                version_control_address: abstr_stargaze.version_control.address()?,
            }
        );

        let manager_info = remote_manager.info()?;

        let account_name = TEST_ACCOUNT_NAME.to_string();
        let description = Some(TEST_ACCOUNT_DESCRIPTION.to_string());
        let link = Some(TEST_ACCOUNT_LINK.to_string());
        assert_eq!(
            manager_info,
            InfoResponse {
                info: abstract_core::manager::state::AccountInfo {
                    name: account_name,
                    governance_details:
                        abstract_core::objects::gov_type::GovernanceDetails::External {
                            governance_address: abstr_juno.ibc.host.address()?,
                            governance_type: "abstract-ibc".to_string()
                        },
                    chain_id: STARGAZE.to_string(),
                    description,
                    link
                }
            }
        );

        // We try to execute a message from the proxy contract (account creation for instance)

        // ii. Now we test that we can indeed create an account remotely from the interchain account
        let create_account_remote_tx = abstr_juno.account.manager.execute_on_remote_module(
            &ChainName::from_chain_id(STARGAZE).to_string(),
            PROXY,
            to_json_binary(&abstract_core::proxy::ExecuteMsg::ModuleAction {
                msgs: vec![wasm_execute(
                    abstr_stargaze.account_factory.address()?,
                    &abstract_core::account_factory::ExecuteMsg::CreateAccount {
                        governance: GovernanceDetails::Monarchy {
                            monarch: abstr_stargaze.version_control.address()?.to_string(),
                        },
                        name: "Abstract Test Remote Remote account".to_string(),
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
            None,
        )?;

        // The create remote account tx is passed ?
        mock_interchain.wait_ibc(&JUNO.to_owned(), create_account_remote_tx)?;

        Ok(())
    }

    #[test]
    fn test_cannot_call_remote_manager_from_other_account() -> AnyResult<()> {
        logger_test_init();
        let sender = Addr::unchecked("sender");
        let mock_interchain = MockInterchainEnv::new(vec![(JUNO, &sender), (STARGAZE, &sender)]);

        // We just verified all steps pass
        let (abstr1, abstr2) = ibc_abstract_setup(&mock_interchain, JUNO, STARGAZE)?;

        create_test_remote_account(&abstr1, JUNO, STARGAZE, &mock_interchain)?;

        let account_id = abstr1.account.id()?;

        // We check the account description changed on chain 2
        let remote_account = abstr2.version_control.account_base(AccountId::remote(
            account_id.seq(),
            vec![ChainName::from_chain_id(JUNO)],
        )?)?;

        let manager = remote_account.account_base.manager;
        abstr2.account.manager.set_address(&manager);

        let new_name = String::from("Funky Crazy Name");
        let new_description = String::from("Funky new account with wonderful capabilities");
        let new_link = String::from("https://abstract.money");

        let result = abstr2.account.manager.call_as(&sender).update_info(
            Some(new_description),
            Some(new_link),
            Some(new_name),
        );

        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_cannot_call_ibc_host_directly_with_remove_chain_proxy() -> AnyResult<()> {
        logger_test_init();
        let sender = Addr::unchecked("sender");
        let mock_interchain = MockInterchainEnv::new(vec![(JUNO, &sender), (STARGAZE, &sender)]);

        // We just verified all steps pass
        let (_abstr1, abstr2) = ibc_abstract_setup(&mock_interchain, JUNO, STARGAZE)?;

        let result = abstr2
            .ibc
            .host
            .call_as(&Addr::unchecked("rando"))
            .remove_chain_proxy(STARGAZE.to_owned());

        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_cannot_call_ibc_host_directly_with_register_chain_proxy() -> AnyResult<()> {
        logger_test_init();
        let sender = Addr::unchecked("sender");
        let mock_interchain = MockInterchainEnv::new(vec![(JUNO, &sender), (STARGAZE, &sender)]);

        // We just verified all steps pass
        let (_abstr1, abstr2) = ibc_abstract_setup(&mock_interchain, JUNO, STARGAZE)?;

        let result = abstr2
            .ibc
            .host
            .call_as(&Addr::unchecked("rando"))
            .register_chain_proxy(OSMOSIS.to_owned(), PROXY.to_owned());
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_cannot_call_ibc_host_directly_with_dispatch_action() -> AnyResult<()> {
        logger_test_init();
        let sender = Addr::unchecked("sender");
        let mock_interchain = MockInterchainEnv::new(vec![(JUNO, &sender), (STARGAZE, &sender)]);

        // We just verified all steps pass
        let (abstr1, abstr2) = ibc_abstract_setup(&mock_interchain, JUNO, STARGAZE)?;

        create_test_remote_account(&abstr1, JUNO, STARGAZE, &mock_interchain)?;
        let account_id = abstr1.account.id()?;

        // We check the account description changed on chain 2
        let remote_account_id =
            AccountId::remote(account_id.seq(), vec![ChainName::from_chain_id(JUNO)])?;

        let result = abstr2.ibc.host.execute(
            &HostExecuteMsg::Execute {
                account_id: remote_account_id,
                proxy_address: abstr1.account.proxy.address()?.to_string(),
                action: HostAction::Dispatch {
                    manager_msg: ManagerExecuteMsg::UpdateInfo {
                        name: Some("name".to_owned()),
                        description: Some("description".to_owned()),
                        link: Some("link".to_owned()),
                    },
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
        let sender = Addr::unchecked("sender");
        let mock_interchain = MockInterchainEnv::new(vec![(JUNO, &sender), (STARGAZE, &sender)]);

        // We just verified all steps pass
        let (abstr1, abstr2) = ibc_abstract_setup(&mock_interchain, JUNO, STARGAZE)?;

        let remote_account_id =
            create_test_remote_account(&abstr1, JUNO, STARGAZE, &mock_interchain)?;

        let result = abstr2.ibc.host.execute(
            &HostExecuteMsg::Execute {
                account_id: remote_account_id,
                proxy_address: abstr1.account.proxy.address()?.to_string(),
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
}
