use std::sync::Arc;

use abstract_interface::{
    cw_orch::daemon::networks::parse_network, cw_orch::prelude::*, AppDeployer,
};
use cw_orch::prelude::networks::ChainInfo;
use semver::Version;

use clap::Parser;
use template_app::{interface::Template, TEMPLATE_ID};
use tokio::runtime::Runtime;

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

fn deploy_app(chain: ChainInfo) -> anyhow::Result<()> {
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

    let chain = parse_network(&args.network_id);

    deploy_app(chain)
}
