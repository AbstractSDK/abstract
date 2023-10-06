use abstract_interface::AppDeployer;
use abstract_interface::DeployStrategy;
use cw_orch::daemon::ChainInfo;
use cw_orch::daemon::DaemonBuilder;

use cw_orch::daemon::networks::parse_network;
use cw_orch::tokio::runtime::Runtime;

use clap::Parser;
use betting_app::contract::interface::BetApp;
use betting_app::BET_APP_ID;
use semver::Version;

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

fn deploy_etf(networks: Vec<ChainInfo>) -> anyhow::Result<()> {
    let rt = Runtime::new()?;
    let version: Version = CONTRACT_VERSION.parse().unwrap();
    for network in networks {
        let chain = DaemonBuilder::default()
            .chain(network)
            .handle(rt.handle())
            .build()?;
        let etf = BetApp::new(BET_APP_ID, chain);
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

    let networks = args.network_ids.iter().map(|n| parse_network(n)).collect();

    deploy_etf(networks)
}
