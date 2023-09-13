use std::str::FromStr;

use abstract_core::{
    objects::{account::AccountTrace, chain_name::ChainName, AccountId},
    IBC_CLIENT,
};
// We need to rewrite this because cosmrs::Msg is not implemented for IBC types

use abstract_interface::{Abstract, AccountDetails, ManagerQueryFns};
use anyhow::Result as AnyResult;
use cosmwasm_std::Empty;
use cw_orch::deploy::Deploy;
use cw_orch_interchain_core::{channel::IbcQueryHandler, InterchainEnv};
use tokio::runtime::Runtime;

pub const TEST_ACCOUNT_NAME: &str = "account-test";
pub const TEST_ACCOUNT_DESCRIPTION: &str = "Description of the account";
pub const TEST_ACCOUNT_LINK: &str = "https://google.com";

pub fn set_env() {
    std::env::set_var("STATE_FILE", "daemon_state.json"); // Set in code for tests
    std::env::set_var("ARTIFACTS_DIR", "../artifacts"); // Set in code for tests
}

pub fn create_test_remote_account<Chain: IbcQueryHandler, IBC: InterchainEnv<Chain>>(
    rt: &Runtime,
    origin: &Chain,
    origin_name: &str,
    destination: &str,
    interchain: &IBC,
) -> AnyResult<AccountId> {
    let origin_abstract = Abstract::load_from(origin.clone())?;

    // Create a local account for testing
    let account_name = TEST_ACCOUNT_NAME.to_string();
    let description = Some(TEST_ACCOUNT_DESCRIPTION.to_string());
    let link = Some(TEST_ACCOUNT_LINK.to_string());
    origin_abstract.account_factory.create_new_account(
        AccountDetails {
            name: account_name.clone(),
            description: description.clone(),
            link: link.clone(),
            base_asset: None,
            install_modules: vec![],
            namespace: None,
        },
        abstract_core::objects::gov_type::GovernanceDetails::Monarchy {
            monarch: origin.sender().to_string(),
        },
        None,
    )?;

    // We need to register the ibc client as a module of the manager (account specific)
    origin_abstract
        .account
        .manager
        .install_module(IBC_CLIENT, &Empty {}, None)?;

    // Now we send a message to the client saying that we want to create an account on osmosis
    let register_tx = origin_abstract
        .account
        .register_remote_account(destination)?;

    rt.block_on(interchain.wait_ibc(&origin_name.to_owned(), register_tx))?;

    // After this is all ended, we return the account id of the account we just created on the remote chain
    let account_config = origin_abstract.account.manager.config()?;

    Ok(AccountId::new(
        account_config.account_id.seq(),
        AccountTrace::Remote(vec![ChainName::from_str(origin_name)?]),
    )?)
}

#[cfg(all(feature = "starship-tests", test))]
mod test {

    use abstract_core::manager::InfoResponse;
    use abstract_core::objects::gov_type::GovernanceDetails;
    use cw_orch::deploy::Deploy;
    use cw_orch::starship::Starship;

    use abstract_core::{manager::ConfigResponse, PROXY};
    use abstract_interface::{IbcHost, Manager, ManagerQueryFns};
    use cosmwasm_std::{to_binary, wasm_execute};

    use abstract_core::IBC_HOST;
    use anyhow::Result as AnyResult;

    use super::*;
    use crate::ibc::create_test_remote_account;
    use cw_orch::prelude::*;

    use crate::JUNO;
    use crate::STARGAZE;

    #[test]
    fn test_create_ibc_account() -> AnyResult<()> {
        set_env();
        env_logger::init();

        // We start by creating an abstract account
        let rt: tokio::runtime::Runtime = tokio::runtime::Runtime::new()?;

        let config_path = format!("{}{}", env!("CARGO_MANIFEST_DIR"), TEST_STARSHIP_CONFIG);

        let starship = Starship::new(rt.handle().to_owned(), &config_path, None)?;
        let interchain: InterchainEnv = starship.interchain_env();

        let juno = interchain.daemon(JUNO)?;
        let osmosis = interchain.daemon(STARGAZE)?;

        // The setup needs to be done with the setup bin script
        let juno_abstr = Abstract::load_from(juno.clone())?;
        let osmo_abstr = Abstract::load_from(osmosis.clone())?;

        let juno_host = IbcHost::new(IBC_HOST, juno.clone());

        let remote_account =
            create_test_remote_account(&rt, &osmosis, "stargaze", "juno", &interchain)?;
        let remote_account_config = juno_abstr
            .version_control
            .get_account(remote_account.clone())?;
        // This shouldn't fail as we have just created an account using those characteristics
        log::info!("Remote account config {:?} ", remote_account_config);

        let remote_manager = Manager::new("remote_account_manager", juno.clone());
        remote_manager.set_address(&remote_account_config.manager);

        // Now we need to test some things about this account on the juno chain
        let manager_config = remote_manager.config()?;
        assert_eq!(
            manager_config,
            ConfigResponse {
                account_id: remote_account,
                is_suspended: false,
                module_factory_address: juno_abstr.module_factory.address()?,
                version_control_address: juno_abstr.version_control.address()?,
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

        let create_account_remote_tx = osmo_abstr.account.manager.execute_on_remote_module(
            "juno",
            PROXY,
            to_binary(&abstract_core::proxy::ExecuteMsg::ModuleAction {
                msgs: vec![wasm_execute(
                    juno_abstr.account_factory.address()?,
                    &abstract_core::account_factory::ExecuteMsg::CreateAccount {
                        governance: GovernanceDetails::Monarchy {
                            monarch: juno_abstr.version_control.address()?.to_string(),
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
            None,
        )?;

        // The create remote account tx is passed ?
        rt.block_on(
            interchain.await_ibc_execution(STARGAZE.to_owned(), create_account_remote_tx.txhash),
        )?;

        Ok(())
    }
}
