use abstract_interface::{Abstract, AccountI};
use abstract_std::objects::gov_type::GovernanceDetails;
use std::{
    fs::{self, File},
    io::BufReader,
    net::TcpStream,
};

use abstract_scripts::{assert_wallet_balance, DeploymentStatus, SUPPORTED_CHAINS};

use clap::Parser;
use cw_orch::{daemon::networks::parse_network, prelude::*};
use reqwest::Url;
use tokio::runtime::Runtime;

pub const ABSTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// Run "cargo run --example download_wasms" in the `abstract-interfaces` package before deploying!
fn full_deploy(mut networks: Vec<ChainInfoOwned>) -> anyhow::Result<()> {
    let rt = Runtime::new()?;

    if networks.is_empty() {
        networks = SUPPORTED_CHAINS.iter().map(|x| x.clone().into()).collect();
    }

    let networks = rt.block_on(assert_wallet_balance(networks));

    for network in networks {
        let chain = DaemonBuilder::new(network.clone())
            .handle(rt.handle())
            .build()?;

        let monarch = chain.sender_addr();

        let deployment = match Abstract::deploy_on(chain, ()) {
            Ok(deployment) => {
                // write_deployment(&deployment_status)?;
                deployment
            }
            Err(e) => {
                // write_deployment(&deployment_status)?;
                return Err(e.into());
            }
        };

        // Create the Abstract Account because it's needed for the fees for the dex module
        AccountI::create_default_account(
            &deployment,
            GovernanceDetails::Monarchy {
                monarch: monarch.to_string(),
            },
        )?;
    }

    // fs::copy(Path::new("~/.cw-orchestrator/state.json"), to)
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

    // let networks = vec![abstract_scripts::ROLLKIT_TESTNET];

    let networks = args
        .network_ids
        .iter()
        .map(|n| parse_network(n).unwrap().into())
        .collect::<Vec<_>>();

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
