use abstract_interface::{Abstract, AccountI};
use abstract_std::objects::gov_type::GovernanceDetails;
use cosmrs::AccountId;
use cosmwasm_std::{CanonicalAddr, HexBinary};
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

// Juno default mnemonics (used for deployment of mock abstract)
const JUNO_MNEMONIC: &str = "clip hire initial neck maid actor venue client foam budget lock catalog sweet steak waste crater broccoli pipe steak sister coyote moment obvious choose";

// Run "cargo run --example download_wasms" in the `abstract-interfaces` package before deploying!
fn full_deploy(mut networks: Vec<ChainInfoOwned>) -> anyhow::Result<()> {
    let rt = Runtime::new()?;

    if networks.is_empty() {
        networks = SUPPORTED_CHAINS.iter().map(|x| x.clone().into()).collect();
    }

    // let deployment_status = read_deployment()?;
    // if deployment_status.success {
    //     log::info!("Do you want to re-deploy to {:?}?", networks);
    //     let mut input = String::new();
    //     std::io::stdin().read_line(&mut input)?;
    //     if input.to_lowercase().contains('n') {
    //         return Ok(());
    //     }
    // }
    // let deployment_status = deployment_status.clone();

    // If some chains need to be deployed, deploy them
    // if !deployment_status.chain_ids.is_empty() {
    //     networks = deployment_status.chain_ids.into_iter().map(|n| parse_network(&n)).collect();
    // }

    // let networks = rt.block_on(assert_wallet_balance(networks));

    // write_deployment(&deployment_status)?;

    for network in networks {
        let chain = DaemonBuilder::new(network.clone())
            .handle(rt.handle())
            .mnemonic(JUNO_MNEMONIC)
            .build()?;
        let account_id: cosmrs::AccountId = chain.sender_addr().as_str().parse().unwrap();
        panic!("{}", account_id);

        let sender = chain.sender().clone();
        let monarch = chain.sender_addr();

        let deployment = match Abstract::deploy_on(chain, sender) {
            Ok(deployment) => deployment,
            Err(e) => {
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
    Ok(())
}

// Do we want to store it somewhere?
#[allow(dead_code)]
fn write_deployment(status: &DeploymentStatus) -> anyhow::Result<()> {
    let path = dirs::home_dir()
        .unwrap()
        .join(".cw-orchestrator")
        .join("chains.json");
    let status_str = serde_json::to_string_pretty(status)?;
    fs::write(path, status_str)?;
    Ok(())
}

fn read_deployment() -> anyhow::Result<DeploymentStatus> {
    let path = dirs::home_dir()
        .unwrap()
        .join(".cw-orchestrator")
        .join("chains.json");
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    // Read the JSON contents of the file as an instance of `DeploymentStatus`. If not present use default.
    Ok(serde_json::from_reader(reader).unwrap_or_default())
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
