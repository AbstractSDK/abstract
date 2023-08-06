use std::fs::remove_file;

use abstract_core::objects::gov_type::GovernanceDetails;
use abstract_interface::Abstract;

use abstract_interface_scripts::assert_wallet_balance;
use clap::Parser;
use cw_orch::{
    deploy::Deploy,
    prelude::{
        networks::{parse_network, ChainInfo},
        *,
    },
};
use tokio::runtime::Runtime;

pub const ABSTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// Run "cargo run --example download_wasms" in the `abstract-interfaces` package before deploying!
fn full_deploy(networks: Vec<ChainInfo>) -> anyhow::Result<()> {
    let rt = Runtime::new()?;
    // remove the state file
    remove_file("./daemon_state.json").unwrap_or_default();

    let networks = rt.block_on(assert_wallet_balance(&networks));

    for network in networks {
        let chain = DaemonBuilder::default()
            .handle(rt.handle())
            .chain(network.clone())
            .build()?;
        let sender = chain.sender();
        let deployment = Abstract::deploy_on(chain, sender.to_string())?;

        // Create the Abstract Account because it's needed for the fees for the dex module
        deployment
            .account_factory
            .create_default_account(GovernanceDetails::Monarchy {
                monarch: sender.to_string(),
            })?;
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
        .map(|n| parse_network(n))
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    if let Err(ref err) = full_deploy(networks) {
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
