use std::env;
use tokio::runtime::Runtime;

use abstract_dex_adapter::interface::DexAdapter;
use abstract_dex_adapter::interface::DEX_ADAPTER_CHAIN_NAME_VAR;
use abstract_interface::AdapterDeployer;
use abstract_interface::DeployStrategy;
use cw_orch::daemon::ChainInfo;
use cw_orch::daemon::DaemonBuilder;

use cw_orch::daemon::networks::parse_network;

use abstract_dex_adapter::{msg::DexInstantiateMsg, DEX_ADAPTER_ID};
use cosmwasm_std::Decimal;

use semver::Version;

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

fn deploy_dex(network: ChainInfo) -> anyhow::Result<()> {
    let rt = Runtime::new()?;
    let version: Version = CONTRACT_VERSION.parse().unwrap();
    let chain = DaemonBuilder::default()
        .handle(rt.handle())
        .chain(network)
        .build()?;
    let dex = DexAdapter::new(DEX_ADAPTER_ID, chain);
    dex.deploy(
        version,
        DexInstantiateMsg {
            swap_fee: Decimal::percent(1),
            recipient_account: 0,
        },
        DeployStrategy::Try,
    )?;
    Ok(())
}

use clap::Parser;

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

    let network = parse_network(&args.network_id);

    deploy_dex(network)
}
