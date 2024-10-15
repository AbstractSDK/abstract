use abstract_interface::Abstract;
use cw_orch::prelude::*;
use cw_orch_clone_testing::CloneTesting;

/// Sets up the CloneTesting for chain.
/// Returns the abstract deployment and sender (=mainnet admin)
pub fn setup(chain: ChainInfo) -> anyhow::Result<(Abstract<CloneTesting>, CloneTesting)> {
    let _ = env_logger::builder().is_test(true).try_init();
    // Run migration tests against Juno mainnet
    // We set the state file to be able to clone test
    std::env::set_var("STATE_FILE", "../scripts/state.json");
    let mut app = CloneTesting::new(chain)?;
    app.set_sender(Abstract::mock_admin(&app));

    // TODO: we have nothing to migrate yet, it's a first version of Abstract2
    let abstr_deployment = Abstract::load_from(app.clone())?;
    // let abstr_deployment = Abstract::deploy_on_mock(app.clone())?;
    Ok((abstr_deployment, app))
}
