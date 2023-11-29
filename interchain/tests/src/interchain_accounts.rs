use std::str::FromStr;

use abstract_core::{
    objects::{account::AccountTrace, chain_name::ChainName, AccountId},
    IBC_CLIENT,
};
// We need to rewrite this because cosmrs::Msg is not implemented for IBC types

use abstract_interface::{Abstract, AbstractAccount, AccountDetails, ManagerQueryFns};
use anyhow::Result as AnyResult;
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
    let origin_account = AbstractAccount::new(origin, AccountId::local(0));

    // Create a local account for testing
    let account_name = TEST_ACCOUNT_NAME.to_string();
    let description = Some(TEST_ACCOUNT_DESCRIPTION.to_string());
    let link = Some(TEST_ACCOUNT_LINK.to_string());
    let account = origin.account_factory.create_new_account(
        AccountDetails {
            name: account_name.clone(),
            description: description.clone(),
            link: link.clone(),
            base_asset: None,
            install_modules: vec![],
            namespace: None,
        },
        abstract_core::objects::gov_type::GovernanceDetails::Monarchy {
            monarch: origin_account.manager.get_chain().sender().to_string(),
        },
        None,
    )?;

    // We need to enable IBC on the account.
    account.manager.update_settings(Some(true))?;

    // Now we send a message to the client saying that we want to create an account on the
    // destination chain
    let register_tx = account.register_remote_account(&destination_name)?;

    interchain.wait_ibc(&origin_id.to_owned(), register_tx)?;

    // After this is all ended, we return the account id of the account we just created on the remote chain
    let account_config = account.manager.config()?;

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
    use abstract_core::manager::state::AccountInfo;
    use abstract_core::manager::InfoResponse;
    use abstract_core::objects::gov_type::GovernanceDetails;
    use abstract_core::IBC_CLIENT;

    use abstract_core::{manager::ConfigResponse, PROXY};
    use abstract_interface::AbstractAccount;
    use abstract_interface::AccountFactoryExecFns;
    use abstract_interface::{ManagerExecFns, ManagerQueryFns};
    use cosmwasm_std::{to_json_binary, wasm_execute};

    use anyhow::Result as AnyResult;
    use cw_orch::mock::cw_multi_test::AppResponse;

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
        let abstr1_account = AbstractAccount::new(&abstr1, AccountId::local(1));
        let abstr2_account = AbstractAccount::new(&abstr2, AccountId::local(0));

        // The user on chain 1 want to change the account description
        let ibc_action_result = abstr1_account.manager.execute_on_remote(
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
        abstr2_account.manager.set_address(&manager);

        let account_info = abstr2_account.manager.info()?;

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
        mock_interchain.create_contract_channel(
            &polytone_1.note,
            &polytone_2.voice,
            POLYTONE_VERSION,
        )?;

        mock_interchain.create_contract_channel(
            &polytone_2.note,
            &polytone_3.voice,
            POLYTONE_VERSION,
        )?;

        // Create the connection between client and host
        abstract_ibc_connection_with(&origin, &mock_interchain, &intermediate, &polytone_1)?;
        abstract_ibc_connection_with(&intermediate, &mock_interchain, &destination, &polytone_2)?;

        // END SETUP

        // Create a local account for testing
        let account_name = TEST_ACCOUNT_NAME.to_string();
        let description = Some(TEST_ACCOUNT_DESCRIPTION.to_string());
        let link = Some(TEST_ACCOUNT_LINK.to_string());
        let new_account = origin.account_factory.create_new_account(
            AccountDetails {
                name: account_name.clone(),
                description: description.clone(),
                link: link.clone(),
                base_asset: None,
                install_modules: vec![],
                namespace: None,
            },
            abstract_core::objects::gov_type::GovernanceDetails::Monarchy {
                monarch: origin.version_control.get_chain().sender().to_string(),
            },
            None,
        )?;

        // We need to register the ibc client as a module of the manager (account specific)
        new_account
            .manager
            .install_module::<Empty>(IBC_CLIENT, None, None)?;

        // Now we send a message to the client saying that we want to create an account on the
        // destination chain
        let register_tx =
            new_account.register_remote_account(&ChainName::from_chain_id(STARGAZE).to_string())?;

        mock_interchain.wait_ibc(&JUNO.to_owned(), register_tx)?;

        // Register the IBC_CLIENT on STARGAZE from JUNO.
        let register_module_tx = new_account.manager.execute_on_remote(
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
        let create_account_remote_tx = new_account.manager.execute_on_remote_module(
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

        mock_interchain.wait_ibc(&JUNO.to_owned(), create_account_remote_tx)?;

        let destination_account_id = AccountId::new(
            new_account.manager.config()?.account_id.seq(),
            AccountTrace::Remote(vec![
                ChainName::from_chain_id(JUNO),
                ChainName::from_chain_id(STARGAZE),
            ]),
        )?;

        let account = AbstractAccount::new(&destination, destination_account_id.clone());

        let manager_config = account.manager.config()?;
        assert_eq!(
            manager_config,
            ConfigResponse {
                account_id: destination_account_id,
                is_suspended: false,
                module_factory_address: destination.module_factory.address()?,
                version_control_address: destination.version_control.address()?,
            }
        );

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

        let abstr_juno_account = AbstractAccount::new(&abstr_juno, AccountId::local(1));
        let remote_abstract_account = AbstractAccount::new(&abstr_stargaze, remote_account.clone());
        let remote_manager = remote_abstract_account.manager;

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
        let account_name = String::from("Abstract Test Remote Remote account");
        let create_account_remote_tx = abstr_juno_account.manager.execute_on_remote_module(
            &ChainName::from_chain_id(STARGAZE).to_string(),
            PROXY,
            to_json_binary(&abstract_core::proxy::ExecuteMsg::ModuleAction {
                msgs: vec![wasm_execute(
                    abstr_stargaze.account_factory.address()?,
                    &abstract_core::account_factory::ExecuteMsg::CreateAccount {
                        governance: GovernanceDetails::Monarchy {
                            monarch: abstr_stargaze.version_control.address()?.to_string(),
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
            None,
        )?;

        // The create remote account tx is passed ?
        mock_interchain.wait_ibc(&JUNO.to_owned(), create_account_remote_tx)?;

        // Can get the account from stargaze.
        let account_id = AccountId::new(1, AccountTrace::Local)?;

        let abstr_account = AbstractAccount::new(&abstr_stargaze, account_id);

        let account_info: AccountInfo<Addr> = abstr_account.manager.info()?.info;

        assert_eq!(
            AccountInfo {
                chain_id: STARGAZE.to_owned(),
                governance_details: GovernanceDetails::Monarchy {
                    monarch: abstr_stargaze.version_control.address()?.to_string(),
                },
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
        let sender = Addr::unchecked("sender");
        let mock_interchain = MockInterchainEnv::new(vec![(JUNO, &sender), (STARGAZE, &sender)]);

        let (_origin, destination) = ibc_abstract_setup(&mock_interchain, JUNO, STARGAZE)?;

        let res = try_create_remote_account(&destination, &Addr::unchecked("user"));
        assert!(res.is_err());

        let res = try_create_remote_account(&destination, &destination.ibc.host.address()?);
        assert!(res.is_ok());

        Ok(())
    }

    fn try_create_remote_account(abstr: &Abstract<Mock>, sender: &Addr) -> AnyResult<AppResponse> {
        Ok(abstr.account_factory.call_as(sender).create_account(
            GovernanceDetails::Monarchy {
                monarch: String::from("user"),
            },
            vec![],
            String::from("name"),
            Some(AccountId::new(
                2,
                AccountTrace::Remote(vec![ChainName::from_chain_id(JUNO)]),
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
        let sender = Addr::unchecked("sender");
        let mock_interchain = MockInterchainEnv::new(vec![(JUNO, &sender), (STARGAZE, &sender)]);

        // We just verified all steps pass
        let (abstr1, abstr2) = ibc_abstract_setup(&mock_interchain, JUNO, STARGAZE)?;
        let abstr2_account = AbstractAccount::new(&abstr2, AccountId::local(0));

        let remote_account_id =
            create_test_remote_account(&abstr1, JUNO, STARGAZE, &mock_interchain)?;

        let remote_account = abstr2.version_control.account_base(remote_account_id)?;

        let manager = remote_account.account_base.manager;
        abstr2_account.manager.set_address(&manager);

        let new_name = String::from("Funky Crazy Name");
        let new_description = String::from("Funky new account with wonderful capabilities");
        let new_link = String::from("https://abstract.money");

        // Cannot call with sender that is not host.
        let result = abstr2_account.manager.call_as(&sender).update_info(
            Some(new_description.clone()),
            Some(new_link.clone()),
            Some(new_name.clone()),
        );

        assert!(result.is_err());

        // Can call with host.
        let result = abstr2_account
            .manager
            .call_as(&abstr2.ibc.host.address()?)
            .update_info(Some(new_description), Some(new_link), Some(new_name));

        assert!(result.is_ok());

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
            .remove_chain_proxy(ChainName::from_chain_id(STARGAZE).to_string());

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
            .register_chain_proxy(
                ChainName::from_chain_id(OSMOSIS).to_string(),
                PROXY.to_owned(),
            );
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
        let abstr1_account = AbstractAccount::new(&abstr1, AccountId::local(0));

        let remote_account_id =
            create_test_remote_account(&abstr1, JUNO, STARGAZE, &mock_interchain)?;

        let result = abstr2.ibc.host.execute(
            &HostExecuteMsg::Execute {
                account_id: remote_account_id,
                proxy_address: abstr1_account.proxy.address()?.to_string(),
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
        let abstr1_account = AbstractAccount::new(&abstr1, AccountId::local(0));

        let remote_account_id =
            create_test_remote_account(&abstr1, JUNO, STARGAZE, &mock_interchain)?;

        let result = abstr2.ibc.host.execute(
            &HostExecuteMsg::Execute {
                account_id: remote_account_id,
                proxy_address: abstr1_account.proxy.address()?.to_string(),
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
