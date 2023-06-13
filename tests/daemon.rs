mod common;

#[cfg(feature = "node-tests")]
use ::{
    abstract_core::{app::BaseInstantiateMsg, objects::gov_type::GovernanceDetails},
    abstract_interface::{Abstract, AbstractAccount, AppDeployer, VCExecFns, *},
    app::{
        contract::{APP_ID, APP_VERSION},
        msg::{AppInstantiateMsg, ConfigResponse, InstantiateMsg},
        *,
    },
    cw_orch::{anyhow, deploy::Deploy, prelude::*},
};

#[cfg(feature = "node-tests")]
/// Set up the test environment with the contract installed
fn setup() -> anyhow::Result<(AbstractAccount<Daemon>, Abstract<Daemon>, App<Daemon>)> {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    let daemon = Daemon::builder()
        .chain(networks::LOCAL_JUNO)
        .handle(runtime.handle())
        .build()
        .unwrap();

    // Construct the counter interface
    let contract = App::new(APP_ID, daemon.clone());

    // For debugging
    contract.wasm();

    // Deploy Abstract to the mock
    let abstr_deployment = Abstract::deploy_on(daemon.clone(), Empty {})?;

    // Create a new account to install the app onto
    let account =
        abstr_deployment
            .account_factory
            .create_default_account(GovernanceDetails::Monarchy {
                monarch: daemon.sender().to_string(),
            })?;

    // claim the namespace so app can be deployed
    abstr_deployment
        .version_control
        .claim_namespaces(1, vec!["my-namespace".to_string()])?;

    contract.deploy(APP_VERSION.parse()?)?;

    account.install_module(
        APP_ID,
        &InstantiateMsg {
            base: BaseInstantiateMsg {
                ans_host_address: abstr_deployment.ans_host.addr_str()?,
            },
            module: AppInstantiateMsg {},
        },
        None,
    )?;

    let modules = account.manager.module_infos(None, None)?;
    contract.set_address(&modules.module_infos[1].address);

    Ok((account, abstr_deployment, contract))
}

#[test]
#[serial_test::serial]
#[cfg(feature = "node-tests")]
fn successful_install() -> anyhow::Result<()> {
    // Set up the environment and contract
    let (_account, _abstr, app) = setup()?;

    let config = app.config()?;
    assert_eq!(config, ConfigResponse {});
    Ok(())
}
