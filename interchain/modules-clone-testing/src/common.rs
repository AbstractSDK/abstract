use abstract_client::{AbstractClient, Environment};
use abstract_interface::VCQueryFns;
use abstract_interface::{Abstract, VCExecFns};
use cosmwasm_std::coins;
use cw_orch::prelude::*;
use cw_orch_clone_testing::CloneTesting;

/// Sets up the CloneTesting for chain.
/// Returns the abstract client
pub fn load_abstr(chain: ChainInfo, sender: Addr) -> anyhow::Result<AbstractClient<CloneTesting>> {
    let _ = env_logger::builder().is_test(true).try_init();
    // We set the state file to be able to clone test
    std::env::set_var("STATE_FILE", "../scripts/state.json");
    // We set the state file to be able to clone test
    let gas_denom = chain.gas_denom;
    let mut app = CloneTesting::new(chain)?;
    // Make sure sender have enough gas
    app.add_balance(&sender, coins(1_000_000_000_000_000, gas_denom))?;
    app.set_sender(sender);

    // TODO: first version, nothing to load yet
    // let abstr_deployment = AbstractClient::new(app)?;
    let abstr_deployment = AbstractClient::builder(app).build_mock()?;

    // Migrate if needed
    {
        let deployment = Abstract::load_from(abstr_deployment.environment())?;
        deployment.migrate_if_version_changed()?;
    }

    abstr_deployment.version_control().ownership()?;

    // Allow registration of any module
    abstr_deployment
        .version_control()
        .update_config(None, Some(true))?;

    Ok(abstr_deployment)
}
