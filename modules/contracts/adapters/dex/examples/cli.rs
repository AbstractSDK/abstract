use abstract_adapter::cli::AdapterContext;
use cw_orch::{anyhow, prelude::Daemon, tokio::runtime::Runtime};
use cw_orch_cli::{ContractCli, DaemonFromCli};
use dex_adapter::{contract::DEX_ID, DexAdapter};
use semver::Version;

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let rt = Runtime::new()?;
    let chain = Daemon::from_cli(rt.handle())?;

    let dex = DexAdapter::new(CRONCAT_ID, chain);

    let version: Version = CONTRACT_VERSION.parse().unwrap();
    dex.select_action(AdapterContext {
        version,
        init_msg: DexInstantiateMsg {
            swap_fee: Decimal::percent(1),
            recipient_account: 0,
        },
    })?;

    Ok(())
}
