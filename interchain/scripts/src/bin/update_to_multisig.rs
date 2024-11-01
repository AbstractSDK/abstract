use abstract_interface::Abstract;
use cw_orch::prelude::*;
use networks::LOCAL_JUNO;

fn main() -> anyhow::Result<()> {
    dotenv().ok();
    env_logger::init();
    use dotenv::dotenv;

    // Fill members
    let members = vec![];
    assert!(!members.is_empty(), "Fill multisig members first");

    // Change network
    let network = LOCAL_JUNO;

    let chain = DaemonBuilder::new(network).build()?;
    let deployment = Abstract::load_from(chain.clone())?;

    deployment.update_admin_to_multisig(chain.sender_addr().to_string(), members, [])?;

    Ok(())
}
