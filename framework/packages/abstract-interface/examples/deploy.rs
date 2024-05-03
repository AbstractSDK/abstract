use abstract_interface::Abstract;
use abstract_std::objects::gov_type::GovernanceDetails;
use cw_orch::{
    daemon::networks::LOCAL_JUNO,
    prelude::{
        networks::{parse_network, ChainInfo},
        *,
    },
};
pub const ABSTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

fn full_deploy(networks: Vec<ChainInfo>) -> cw_orch::anyhow::Result<()> {
    for network in networks {
        let chain = DaemonBuilder::default().chain(network.clone()).build()?;
        let sender = chain.sender();

        let deployment = Abstract::deploy_on(chain, sender.to_string())?;
        // Create the Abstract Account because it's needed for the fees for the dex module
        deployment
            .account_factory
            .create_default_account(GovernanceDetails::Monarchy {
                monarch: sender.to_string(),
            })?;
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
