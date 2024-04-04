use abstract_interface::{AdapterDeployer, DeployStrategy};
use abstract_oracle_adapter::{
    interface::OracleAdapter, msg::OracleInstantiateMsg, ORACLE_ADAPTER_ID,
};
use cosmwasm_std::Decimal;
use cw_orch::daemon::{networks::parse_network, ChainInfo, DaemonBuilder};
use semver::Version;

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

fn deploy_oracle(network: ChainInfo) -> anyhow::Result<()> {
    let rt = Runtime::new()?;
    let version: Version = CONTRACT_VERSION.parse().unwrap();
    let chain = DaemonBuilder::default()
        .handle(rt.handle())
        .chain(network)
        .build()?;
    let dex = OracleAdapter::new(ORACLE_ADAPTER_ID, chain);
    dex.deploy(
        version,
        OracleInstantiateMsg {
            external_age_max: 120,
            providers: vec![],
        },
        DeployStrategy::Try,
    )?;
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
