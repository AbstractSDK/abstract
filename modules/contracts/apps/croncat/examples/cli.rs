use croncat_app::{contract::CRONCAT_ID, CroncatApp};
use cw_orch::{
    anyhow,
    prelude::{networks, Daemon, DaemonBuilder},
    tokio::runtime::Runtime,
};
use cw_orch_cli::{ContractCli, OrchCliError};

pub enum AppOptions {
    Deploy,
}

impl ::std::fmt::Display for AppOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppOptions::Deploy => f.pad("Deploy"),
        }
    }
}

#[derive(Clone)]
pub struct AppContext {
    pub version: semver::Version,
}

use cw_orch_cli::CwCliAddons;
use semver::Version;

impl CwCliAddons<AppContext> for CroncatApp<Daemon> {
    fn addons(&mut self, context: AppContext) -> cw_orch_cli::OrchCliResult<()>
    where
        Self: cw_orch::prelude::ContractInstance<cw_orch::prelude::Daemon>,
    {
        let option = ::cw_orch_cli::select_msg(vec![AppOptions::Deploy])?;
        match option {
            AppOptions::Deploy => ::abstract_interface::AppDeployer::deploy(self, context.version)
                .map_err(|e| OrchCliError::CustomError { val: e.to_string() }),
        }
    }
}

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");   

pub fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let rt = Runtime::new()?;
    let network = networks::UNI_6;
    let version: Version = CONTRACT_VERSION.parse().unwrap();
    let chain = DaemonBuilder::default()
        .handle(rt.handle())
        .chain(network)
        .build()?;

    let croncat = CroncatApp::new(CRONCAT_ID, chain);

    croncat.select_action(AppContext { version })?;

    Ok(())
}
