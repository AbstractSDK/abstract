use abstract_app::cli::AppContext;
use croncat_app::{contract::CRONCAT_ID, CroncatApp};
use cw_orch::{anyhow, prelude::Daemon, tokio::runtime::Runtime, deploy::Deploy};
use cw_orch_cli::{ContractCli, DaemonFromCli};
use semver::Version;

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let rt = Runtime::new()?;
    let chain = Daemon::from_cli(rt.handle())?;

    let croncat = CroncatApp::new(CRONCAT_ID, chain);
    
    let version: Version = CONTRACT_VERSION.parse().unwrap();
    croncat.select_action(AppContext { version })?;

    Ok(())
}
