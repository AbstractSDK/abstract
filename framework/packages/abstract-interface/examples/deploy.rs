use abstract_interface::{Abstract, AccountI};
use abstract_std::objects::gov_type::GovernanceDetails;
use cw_orch::{
    daemon::networks::LOCAL_JUNO,
    prelude::{networks::ChainInfo, *},
};
pub const ABSTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

fn full_deploy(networks: Vec<ChainInfo>) -> cw_orch::anyhow::Result<()> {
    for network in networks {
        let chain = DaemonBuilder::new(network.clone()).build()?;

        let deployment = Abstract::deploy_on(chain.clone(), ())?;
        // Create the Abstract Account because it's needed for the fees for the dex module
        AccountI::create_default_account(
            &deployment,
            GovernanceDetails::Monarchy {
                monarch: chain.sender_addr().to_string(),
            },
        )?;
    }

    Ok(())
}

fn main() {
    dotenv().ok();
    env_logger::init();

    use dotenv::dotenv;

    let networks = vec![LOCAL_JUNO];
    full_deploy(networks).unwrap();
}
