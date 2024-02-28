mod astroport;

use abstract_client::{AbstractClient, Environment};
use abstract_interface::{Abstract, VCExecFns};
use cw_orch::daemon::ChainInfo;
use cw_orch::prelude::*;
use cw_orch_clone_testing::CloneTesting;

use super::common::rt;

/// Sets up the CloneTesting for chain.
/// Returns the abstract client
pub fn load_abstr(chain: ChainInfo, sender: Addr) -> anyhow::Result<AbstractClient<CloneTesting>> {
    let _ = env_logger::builder().is_test(true).try_init();
    // We set the state file to be able to clone test
    std::env::set_var("STATE_FILE", "../scripts/state.json");
    let mut app = CloneTesting::new(rt(), chain)?;
    app.set_sender(sender);

    let abstr_deployment = AbstractClient::new(app)?;

    // Migrate if needed
    // TODO: can we expose it somehow for client?
    {
        let deployment = Abstract::load_from(abstr_deployment.environment())?;
        deployment.migrate_if_version_changed()?;
    }

    // Allow registration of any module
    abstr_deployment
        .version_control()
        .update_config(None, Some(true), None)?;

    Ok(abstr_deployment)
}
