use abstract_app::abstract_interface::{AppDeployer, DeployStrategy};
use challenge_app::{contract::CHALLENGE_APP_ID, Challenge};
use cw_orch::{
    anyhow,
    prelude::{networks::parse_network, DaemonBuilder},
    tokio::runtime::Runtime,
};
use semver::Version;

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> anyhow::Result<()> {
    dotenv().ok();
    env_logger::init();
    let chain = parse_network("uni-6").unwrap();
    use dotenv::dotenv;
    let version: Version = CONTRACT_VERSION.parse().unwrap();
    let rt = Runtime::new()?;
    let chain = DaemonBuilder::new(chain).handle(rt.handle()).build()?;
    let app = Challenge::new(CHALLENGE_APP_ID, chain);

    app.deploy(version, DeployStrategy::Try)?;
    Ok(())
}
