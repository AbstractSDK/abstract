use cw_orch::{
    anyhow,
    prelude::{networks::parse_network, DaemonBuilder},
    tokio::runtime::Runtime,
};

use abstract_interface::AppDeployer;
use abstract_jury_duty_multisig::contract::interface::JuryDutyApp;
use abstract_jury_duty_multisig::contract::JURY_DUTY_APP_ID;
use semver::Version;

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

fn deploy_app(network: ChainInfo) -> anyhow::Result<()> {
    let version: Version = CONTRACT_VERSION.parse().unwrap();
    let rt = Runtime::new()?;
    let chain = DaemonBuilder::default()
        .chain(network)
        .handle(rt.handle())
        .build()?;
    let app = JuryDutyApp::new(JURY_DUTY_APP_ID, chain);

    app.deploy(version)?;
    Ok(())
}

use clap::Parser;
use cw_orch::daemon::ChainInfo;

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

    deploy_app(network)
}
