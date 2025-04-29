use abstract_client::{AbstractClient, Environment};
use abstract_interface::RegistryQueryFns;
use abstract_interface::{Abstract, RegistryExecFns};
use cosmwasm_std::coins;
use cw_orch::prelude::*;
use cw_orch_clone_testing::CloneTesting;

/// Sets up the CloneTesting for chain.
/// Returns the abstract client
pub fn load_abstr(chain: ChainInfo) -> anyhow::Result<AbstractClient<CloneTesting>> {
    let _ = env_logger::builder().is_test(true).try_init();
    // We set the state file to be able to clone test
    std::env::set_var("STATE_FILE", "../scripts/state.json");
    // We set the state file to be able to clone test
    let gas_denom = chain.gas_denom;
    let mut app = CloneTesting::new(chain)?;

    // TODO: first version, nothing to load yet
    // let abstr_deployment = AbstractClient::new(app)?;
    let abstr_deployment = AbstractClient::builder(app.clone()).build()?;

    let creator = app
        .wasm_querier()
        .code(abstr_deployment.registry().code_id()?)?
        .creator;
    // Make sure creator have enough gas
    app.add_balance(&creator, coins(1_000_000_000_000_000, gas_denom))?;
    app.set_sender(creator);

    // Migrate if needed
    {
        let deployment = Abstract::load_from(abstr_deployment.environment())?;
        deployment.migrate_if_version_changed()?;
    }

    abstr_deployment.registry().ownership()?;

    // Allow registration of any module
    abstr_deployment
        .registry()
        .update_config(None, Some(false))?;

    Ok(abstr_deployment)
}
