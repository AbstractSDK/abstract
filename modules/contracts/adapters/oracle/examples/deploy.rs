use abstract_interface::{AdapterDeployer, DeployStrategy};
use abstract_oracle_adapter::interface::OracleAdapter;
use cw_orch::daemon::networks::parse_network;
use cw_orch::prelude::*;
use semver::Version;

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

fn deploy_oracle(network: ChainInfo) -> anyhow::Result<()> {
    let rt = Runtime::new()?;
    let version: Version = CONTRACT_VERSION.parse().unwrap();
    let chain = DaemonBuilder::new(network).handle(rt.handle()).build()?;
    let oracle = OracleAdapter::new(chain);
    oracle.deploy(version, Empty {}, DeployStrategy::Try)?;
    Ok(())
}

use clap::Parser;
use tokio::runtime::Runtime;

#[derive(Parser, Default, Debug)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    /// Network Id to deploy on
    #[arg(short, long)]
    network_id: String,
}

fn main() -> anyhow::Result<()> {
    dotenv().ok();
    env_logger::init();

    use dotenv::dotenv;

    let args = Arguments::parse();

    let network = parse_network(&args.network_id).unwrap();

    deploy_oracle(network)
}
