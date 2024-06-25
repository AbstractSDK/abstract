use abstract_app::abstract_interface::*;
use abstract_subscription::contract::{interface::SubscriptionInterface, SUBSCRIPTION_ID};
use clap::Parser;
use cw_orch::{
    anyhow,
    prelude::{networks::parse_network, *},
    tokio::runtime::Runtime,
};
use dotenv::dotenv;
use semver::Version;

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

fn deploy_subscription(networks: Vec<ChainInfo>) -> anyhow::Result<()> {
    let rt = Runtime::new()?;
    let version: Version = CONTRACT_VERSION.parse().unwrap();
    for network in networks {
        let chain = DaemonBuilder::default()
            .chain(network)
            .handle(rt.handle())
            .build()?;
        let subscription_app = SubscriptionInterface::new(SUBSCRIPTION_ID, chain);
        subscription_app.deploy(version.clone(), DeployStrategy::Try)?;
    }
    Ok(())
}
#[derive(Parser, Default, Debug)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    /// Network Id to deploy on
    #[arg(short, long)]
    network_ids: Vec<String>,
}

fn main() -> anyhow::Result<()> {
    dotenv().ok();
    env_logger::init();

    let args = Arguments::parse();

    let networks = args
        .network_ids
        .iter()
        .map(|n| parse_network(n).unwrap())
        .collect();

    deploy_subscription(networks)
}
