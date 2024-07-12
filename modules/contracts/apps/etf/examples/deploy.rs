use abstract_interface::{AppDeployer, DeployStrategy};
use clap::Parser;
use cw_orch::{daemon::networks::parse_network, prelude::*, tokio::runtime::Runtime};
use etf_app::{contract::interface::Etf, ETF_APP_ID};
use semver::Version;

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

fn deploy_etf(networks: Vec<ChainInfo>) -> anyhow::Result<()> {
    let rt = Runtime::new()?;
    let version: Version = CONTRACT_VERSION.parse().unwrap();
    for network in networks {
        let chain = DaemonBuilder::new(network).handle(rt.handle()).build()?;
        let etf = Etf::new(ETF_APP_ID, chain);
        etf.deploy(version.clone(), DeployStrategy::Try)?;
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

    use dotenv::dotenv;

    let args = Arguments::parse();

    let networks = args
        .network_ids
        .iter()
        .map(|n| parse_network(n).unwrap())
        .collect();

    deploy_etf(networks)
}
