//! Deploys the module to the Abstract platform by uploading it and registering it on the version control contract.
//!
//! **Requires you to have the namespace registered**
//!
//! ## Example
//!
//! ```bash
//! $ just deploy uni-6 osmo-test-5
//! ```

use clap::Parser;
use cw_orch::{
    anyhow,
    daemon::ChainInfo,
    prelude::{networks::parse_network, DaemonBuilder},
    tokio::runtime::Runtime,
};
use abstract_interface::AppDeployer;
use app::{
    contract::{APP_ID, APP_VERSION},
    AppInterface,
};
use semver::Version;

fn full_deploy(networks: Vec<ChainInfo>) -> anyhow::Result<()> {
    // run for each requested network
    for network in networks {
        let version: Version = APP_VERSION.parse().unwrap();
        let rt = Runtime::new()?;
        let chain = DaemonBuilder::default()
            .handle(rt.handle())
            .chain(network)
            .build()?;

        let app = AppInterface::new(APP_ID, chain);
        app.deploy(version)?;
    }
    Ok(())
}

#[derive(Parser, Default, Debug)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    /// Network Id to deploy on
    #[arg(short, long, value_delimiter = ' ', num_args = 1..)]
    network_ids: Vec<String>,
}

fn main() {
    dotenv::dotenv().ok();
    env_logger::init();
    let args = Arguments::parse();
    let networks = args.network_ids.iter().map(|n| parse_network(n)).collect();
    full_deploy(networks).unwrap();
}
