use abstract_interface::{AppDeployer, DeployStrategy};
use cw_orch::{
    anyhow,
    prelude::{networks::parse_network, DaemonBuilder},
    tokio::runtime::Runtime,
};
use payment_app::{contract::APP_ID, PaymentAppInterface};
use semver::Version;

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> anyhow::Result<()> {
    dotenv().ok();
    env_logger::init();
    let chain = parse_network("juno-1").unwrap();
    use dotenv::dotenv;
    let version: Version = CONTRACT_VERSION.parse().unwrap();
    let rt = Runtime::new()?;
    let chain = DaemonBuilder::new(chain).handle(rt.handle()).build()?;
    let app = PaymentAppInterface::new(APP_ID, chain);

    app.deploy(version, DeployStrategy::Try)?;
    Ok(())
}
