use croncat_app::{CroncatApp, contract::CRONCAT_ID};
use cw_orch::{
    anyhow,
    prelude::{networks, DaemonBuilder},
    tokio::runtime::Runtime,
};
use cw_orch_cli::ContractCli;

pub fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let rt = Runtime::new()?;
    let network = networks::UNI_6;
    // let version: Version = CONTRACT_VERSION.parse().unwrap();
    let chain = DaemonBuilder::default()
        .handle(rt.handle())
        .chain(network)
        .build()?;

    let croncat = CroncatApp::new(CRONCAT_ID, chain);

    ContractCli::select_action(croncat)?;

    Ok(())
}
