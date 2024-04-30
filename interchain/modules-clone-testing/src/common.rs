use abstract_client::{AbstractClient, Environment};
use abstract_interface::VCQueryFns;
use abstract_interface::{Abstract, VCExecFns};
use cosmwasm_std::coins;
use cw_orch::daemon::ChainInfo;
use cw_orch::prelude::*;
use cw_orch_clone_testing::CloneTesting;

/// Returns a shared tokio runtime for all tests
pub fn rt() -> &'static tokio::runtime::Runtime {
    lazy_static::lazy_static! {
        static ref RT: tokio::runtime::Runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Should create a tokio runtime");
    }
    &RT
}

/// Sets up the CloneTesting for chain.
/// Returns the abstract client
pub fn load_abstr(chain: ChainInfo, sender: Addr) -> anyhow::Result<AbstractClient<CloneTesting>> {
    let _ = env_logger::builder().is_test(true).try_init();
    // We set the state file to be able to clone test
    std::env::set_var("STATE_FILE", "../scripts/state.json");
    // We set the state file to be able to clone test
    let gas_denom = chain.gas_denom;
    let mut app = CloneTesting::new(rt(), chain)?;
    // Make sure sender have enough gas
    app.add_balance(&sender, coins(1_000_000_000_000_000, gas_denom))?;
    app.set_sender(sender);

    let abstr_deployment = AbstractClient::new(app)?;

    // Migrate if needed
    {
        let deployment = Abstract::load_from(abstr_deployment.environment())?;
        deployment.migrate_if_version_changed()?;
    }

    abstr_deployment.version_control().ownership()?;

    // Allow registration of any module
    abstr_deployment
        .version_control()
        .update_config(None, Some(true), None)?;

    Ok(abstr_deployment)
}
