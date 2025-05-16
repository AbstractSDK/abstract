use abstract_interface::{AdapterDeployer, DeployStrategy};
use abstract_oracle_adapter::{contract::CONTRACT_VERSION, interface::OracleAdapter};
use cw_orch::daemon::networks::parse_network;
use cw_orch::prelude::*;
use semver::Version;

fn deploy_oracle(networks: Vec<ChainInfo>) -> anyhow::Result<()> {
    // run for each requested network
    for network in networks {
        let version: Version = CONTRACT_VERSION.parse().unwrap();
        let chain = DaemonBuilder::new(network).build()?;
        let oracle = OracleAdapter::new(chain);
        oracle.deploy(version, Empty {}, DeployStrategy::Try)?;
    }
    Ok(())
}

use clap::Parser;

#[derive(Parser, Default, Debug)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    /// Network Id to deploy on
    #[arg(short, long, value_delimiter = ' ', num_args = 1..)]
    network_ids: Vec<String>,
}

fn main() -> anyhow::Result<()> {
    dotenv::dotenv()?;
    env_logger::init();

    let args = Arguments::parse();
    let networks = args
        .network_ids
        .iter()
        .map(|n| parse_network(n).unwrap())
        .collect();

    deploy_oracle(networks)
}
