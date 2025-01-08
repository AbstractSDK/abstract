use abstract_interface::{Abstract, RegistryQueryFns};
use cw_orch::prelude::*;
use cw_orch_clone_testing::CloneTesting;

/// Sets up the CloneTesting for chain.
/// Returns the abstract deployment and sender (=mainnet admin)
pub fn setup(chain: ChainInfo) -> anyhow::Result<(Abstract<CloneTesting>, CloneTesting)> {
    let _ = env_logger::builder().is_test(true).try_init();
    // Run migration tests against mainnet
    // We set the state file to be able to clone test
    std::env::set_var("STATE_FILE", "../scripts/state.json");
    let mut app = CloneTesting::new(chain)?;

    let abstr_deployment = Abstract::load_from(app.clone())?;
    let owner = Addr::unchecked(abstr_deployment.registry.ownership()?.owner.unwrap());

    app.set_sender(owner.clone());

    Ok((abstr_deployment.call_as(&owner), app))
}
