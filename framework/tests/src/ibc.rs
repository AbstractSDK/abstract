#[cfg(test)]
mod test {

    use abstract_core::ibc_host::HostAction;
    use abstract_core::manager::InfoResponse;
    use abstract_core::objects::gov_type::GovernanceDetails;
    use cw_orch::deploy::Deploy;
    use cw_orch::starship::Starship;

    use abstract_core::objects::chain_name::ChainName;
    use abstract_core::{
        manager::ConfigResponse,
        objects::{account::AccountTrace, AccountId},
        PROXY,
    };
    use abstract_interface::{IbcHost, Manager, ManagerExecFns, ManagerQueryFns};
    use cosmwasm_std::{to_binary, wasm_execute};

    use abstract_core::{IBC_CLIENT, IBC_HOST};
    use abstract_interface::{Abstract, AccountDetails};
    use anyhow::Result as AnyResult;

    use cw_orch::prelude::*;

    const JUNO: &str = "juno-1";
    const OSMOSIS: &str = "osmosis-1";

    fn set_env() {
        std::env::set_var("STATE_FILE", "daemon_state.json"); // Set in code for tests
        std::env::set_var("ARTIFACTS_DIR", "../artifacts"); // Set in code for tests
    }

    #[test]
    fn test_create_ibc_account() -> AnyResult<()> {
        set_env();
        env_logger::init();

        // We start by creating an abstract account
        let rt: tokio::runtime::Runtime = tokio::runtime::Runtime::new()?;

        let starship = Starship::new(rt.handle().to_owned(), None)?;
        let interchain: InterchainEnv = starship.interchain_env();

        let juno = interchain.daemon(JUNO)?;
        let osmosis = interchain.daemon(OSMOSIS)?;

        // The setup needs to be done with the setup bin script
        let juno_abstr = Abstract::load_from(juno.clone())?;
        let osmo_abstr = Abstract::load_from(osmosis.clone())?;

        // Create a local account for testing
        let account_name = "osmo-test".to_string();
        let description = Some("Description of the account".to_string());
        let link = Some("https://google.com".to_string());
        osmo_abstr.account_factory.create_new_account(
            AccountDetails {
                name: account_name.clone(),
                description: description.clone(),
                link: link.clone(),
            },
            abstract_core::objects::gov_type::GovernanceDetails::Monarchy {
                monarch: osmosis.sender().to_string(),
            },
        )?;

        let juno_host = IbcHost::new(IBC_HOST, juno.clone());

        // We need to register the ibc client as a module of the manager (account specific)
        osmo_abstr
            .account
            .manager
            .install_module(IBC_CLIENT, &Empty {}, None)?;

        // Now we send a message to the client saying that we want to create an account on osmosis
        let register_tx = osmo_abstr.account.manager.exec_on_module(
            to_binary(&abstract_core::proxy::ExecuteMsg::IbcAction {
                msgs: vec![abstract_core::ibc_client::ExecuteMsg::Register {
                    host_chain: ChainName::from("juno"),
                }],
            })?,
            PROXY.to_string(),
        )?;

        rt.block_on(interchain.await_ibc_execution(OSMOSIS.to_owned(), register_tx.txhash))?;

        // After this is all ended, we query the accounts to make sure everything is executed and setup alright on the distant chain
        // First we query the account id from the manager
        let account_config = osmo_abstr.account.manager.config()?;
        /*

                // Test query the account id corresponding to the proxy
                let test_proxy = Proxy::new("test-proxy", juno.clone());
                test_proxy.set_address(&Addr::unchecked("juno1jfelp8ny0wqfz3j62qy2p42nfnt379c58hpztsdm9qpswf8ycmzqc2n2a8"));

                let test_manager = Manager::new("test-manager", juno.clone());
                test_manager.set_address(&Addr::unchecked("juno170n33vv4zdzq5xt3xzntxqax6easje4akke0d3gfa0f00sa4vetqrt9qua"));

                let accounts = test_manager.config()?;
                log::info!("e{:?}", accounts);
        */

        let distant_account = AccountId::new(
            account_config.account_id.seq(),
            AccountTrace::Remote(vec![ChainName::from("osmosis")]),
        )?;
        let distant_account_config = juno_abstr
            .version_control
            .get_account(distant_account.clone())?;
        // This shouldn't fail as we have just created an account using those characteristics
        log::info!("Distant account config {:?} ", distant_account_config);

        let distant_manager = Manager::new("distant_account_manager", juno.clone());
        distant_manager.set_address(&distant_account_config.manager);

        // Now we need to test some things about this account on the juno chain
        let manager_config = distant_manager.config()?;
        assert_eq!(
            manager_config,
            ConfigResponse {
                account_id: distant_account,
                is_suspended: false,
                module_factory_address: juno_abstr.module_factory.address()?,
                version_control_address: juno_abstr.version_control.address()?,
            }
        );

        let manager_info = distant_manager.info()?;
        assert_eq!(
            manager_info,
            InfoResponse {
                info: abstract_core::manager::state::AccountInfo {
                    name: account_name,
                    governance_details:
                        abstract_core::objects::gov_type::GovernanceDetails::External {
                            governance_address: juno_host.address()?,
                            governance_type: "abstract-ibc".to_string()
                        },
                    chain_id: "juno-1".to_string(),
                    description,
                    link
                }
            }
        );

        // We try to execute a message from the proxy contract (account creation for instance)

        // ii. Now we test that we can indeed create an account remotely from the interchain account
        let create_account_remote_tx = osmo_abstr.account.manager.exec_on_module(
            to_binary(&abstract_core::proxy::ExecuteMsg::IbcAction {
                msgs: vec![abstract_core::ibc_client::ExecuteMsg::SendPacket {
                    host_chain: ChainName::from("juno"),
                    action: HostAction::Dispatch {
                        manager_msg: abstract_core::manager::ExecuteMsg::ExecOnModule {
                            module_id: PROXY.to_string(),
                            exec_msg: to_binary(&abstract_core::proxy::ExecuteMsg::ModuleAction {
                                msgs: vec![wasm_execute(
                                    juno_abstr.account_factory.address()?,
                                    &abstract_core::account_factory::ExecuteMsg::CreateAccount {
                                        governance: GovernanceDetails::Monarchy {
                                            monarch: juno_abstr
                                                .version_control
                                                .address()?
                                                .to_string(),
                                        },
                                        name: "Abstract Test Remote Remote account".to_string(),
                                        description: None,
                                        link: None,
                                        origin: None,
                                    },
                                    vec![],
                                )?
                                .into()],
                            })?,
                        },
                    },
                    callback_info: None,
                    retries: 2,
                }],
            })?,
            PROXY.to_string(),
        )?;

        // The create remote account tx is passed ?
        rt.block_on(
            interchain.await_ibc_execution(OSMOSIS.to_owned(), create_account_remote_tx.txhash),
        )?;

        // We verify there is a new local account

        Ok(())
    }
}
// We need to rewrite this because cosmrs::Msg is not implemented for IBC types
