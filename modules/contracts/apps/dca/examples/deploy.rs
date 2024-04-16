use abstract_app::abstract_interface::{AppDeployer, DeployStrategy};
use cw_orch::{
    anyhow,
    prelude::{networks::parse_network, DaemonBuilder},
    tokio::runtime::Runtime,
};
use dca_app::{contract::DCA_APP_ID, DCA};
use semver::Version;

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> anyhow::Result<()> {
    dotenv().ok();
    env_logger::init();
    let chain = parse_network("juno-1").unwrap();
    use dotenv::dotenv;
    let version: Version = CONTRACT_VERSION.parse().unwrap();
    let rt = Runtime::new()?;
    let chain = DaemonBuilder::default()
        .chain(chain)
        .handle(rt.handle())
        .build()?;
    let app = DCA::new(DCA_APP_ID, chain);

    app.deploy(version, DeployStrategy::Try)?;
    Ok(())
}
