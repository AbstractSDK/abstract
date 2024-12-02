use abstract_dex_adapter::{
    contract::CONTRACT_VERSION, interface::DexAdapter, msg::DexInstantiateMsg, DEX_ADAPTER_ID,
};
use abstract_interface::{AdapterDeployer, DeployStrategy};
use cosmwasm_std::Decimal;
use cw_orch::daemon::networks::parse_network;
use cw_orch::prelude::*;
use semver::Version;

fn deploy_dex(networks: Vec<ChainInfo>) -> anyhow::Result<()> {
    // run for each requested network
    for network in networks {
        let version: Version = CONTRACT_VERSION.parse().unwrap();
        let chain = DaemonBuilder::new(network).build()?;
        let dex = DexAdapter::new(DEX_ADAPTER_ID, chain);
        dex.deploy(
            version,
            DexInstantiateMsg {
                swap_fee: Decimal::percent(1),
                recipient_account: 0,
            },
            DeployStrategy::Try,
        )?;
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
    dotenv().ok();
    env_logger::init();

    use dotenv::dotenv;

    let args = Arguments::parse();
    let networks = args
        .network_ids
        .iter()
        .map(|n| parse_network(n).unwrap())
        .collect();

    deploy_dex(networks)
}
