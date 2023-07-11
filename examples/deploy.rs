//! Deploys the module to the Abstract platform by uploading it and registering it on the version control contract.
//!
//! This should be used for mainnet/testnet deployments in combination with our front-end at https://app.abstract.money
//!
//! **Requires you to have an account and namespace registered**
//!
//! The mnemonic used to register the module must be the same as the owner of the account that claimed the namespace.
//!
//! Read our docs to learn how: https://docs.abstract.money/4_get_started/5_account_creation.html
//!
//! ## Example
//!
//! ```bash
//! $ just deploy uni-6 osmo-test-5
//! ```

use abstract_interface::AppDeployer;
use app::{
    contract::{APP_ID, APP_VERSION},
    AppInterface,
};
use clap::Parser;
use cw_orch::{
    anyhow,
    daemon::ChainInfo,
    prelude::{networks::parse_network, DaemonBuilder},
    tokio::runtime::Runtime,
};
use semver::Version;

fn deploy(networks: Vec<ChainInfo>) -> anyhow::Result<()> {
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

        // Create an account on our front-end to install the module!
        // https://app.abstract.money
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
    deploy(networks).unwrap();
}
