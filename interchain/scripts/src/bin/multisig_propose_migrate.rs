//! This script creates proposal to migrate contracts and register them in the registry

use abstract_interface::Abstract;
use clap::Parser;
use cw_orch::prelude::{
    networks::{parse_network, ChainInfo},
    *,
};
use tokio::runtime::Runtime;

pub const ABSTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

fn migrate(networks: Vec<ChainInfo>) -> anyhow::Result<()> {
    let rt = Runtime::new()?;
    for network in networks {
        let chain = DaemonBuilder::new(network).handle(rt.handle()).build()?;

        let deployment = Abstract::load_from(chain.clone())?;

        deployment.propose_migrate_if_version_changed()?;
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
    dotenv().ok();
    env_logger::init();
    use dotenv::dotenv;
    let args = Arguments::parse();

    let networks = args
        .network_ids
        .iter()
        .map(|n| parse_network(n).unwrap())
        .collect::<Vec<_>>();

    if let Err(ref err) = migrate(networks) {
        log::error!("{}", err);
        err.chain()
            .skip(1)
            .for_each(|cause| log::error!("because: {}", cause));

        // The backtrace is not always generated. Try to run this example
        // with `$env:RUST_BACKTRACE=1`.
        //    if let Some(backtrace) = e.backtrace() {
        //        log::debug!("backtrace: {:?}", backtrace);
        //    }

        ::std::process::exit(1);
    }
}
