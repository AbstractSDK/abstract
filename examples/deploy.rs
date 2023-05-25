use cw_orch::{anyhow, tokio};

use abstract_interface::{
    cw_orch::daemon::networks::parse_network, cw_orch::prelude::*, AppDeployer,
};
use cw_orch::prelude::networks::ChainInfo;

use template_app::{interface::Template, TEMPLATE_ID};

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

fn main(chain: ChainInfo) -> anyhow::Result<()> {
    dotenv().ok();
    env_logger::init();

    use dotenv::dotenv;
    let version: Version = CONTRACT_VERSION.parse().unwrap();
    let rt = Arc::new(Runtime::new()?);
    let chain = DaemonBuilder::default()
        .chain(chain)
        .handle(rt.handle())
        .build()?;
    let mut app = Template::new(TEMPLATE_ID, chain);

    app.deploy(version)?;
    Ok(())
}
