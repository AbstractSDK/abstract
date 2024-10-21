use std::net::TcpStream;

use abstract_interface::{Abstract, AccountI};
use abstract_scripts::assert_wallet_balance;
use abstract_std::objects::gov_type::GovernanceDetails;
use clap::Parser;
use cw_orch::{environment::NetworkInfoOwned, prelude::*};
use reqwest::Url;
use tokio::runtime::Runtime;

use cw_orch::environment::ChainKind;
use cw_orch_polytone::Polytone;

pub const ABSTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Script to deploy Abstract & polytone to a new network provided by commmand line arguments
/// Run "cargo run --example download_wasms" in the `abstract-interfaces` package before deploying!
fn manual_deploy(network: ChainInfoOwned) -> anyhow::Result<()> {
    let rt = Runtime::new()?;

    rt.block_on(assert_wallet_balance(vec![network.clone()]));

    let urls = network.grpc_urls.to_vec();
    for url in urls {
        rt.block_on(ping_grpc(&url))?;
    }

    let chain = DaemonBuilder::new(network.clone())
        .handle(rt.handle())
        .build()?;

    let sender = chain.sender().clone();
    let monarch = chain.sender_addr();

    // Abstract
    let _abstr = match Abstract::load_from(chain.clone()) {
        Ok(deployed) => deployed,
        Err(_) => {
            let abs = Abstract::deploy_on(chain.clone(), sender)?;
            // Create the Abstract Account because it's needed for the fees for the dex module
            AccountI::create_default_account(
                &abs,
                GovernanceDetails::Monarchy {
                    monarch: monarch.to_string(),
                },
            )?;

            abs
        }
    };

    // Attempt to load or deploy Polytone based on condition check
    let _polytone = match Polytone::load_from(chain.clone()) {
        Ok(deployed) => {
            // Check if the address property of deployed Polytone indicates it's properly deployed
            match deployed.note.address() {
                Ok(_) => deployed, // Use the deployed instance if check is successful
                Err(CwOrchError::AddrNotInStore(_)) => {
                    // If the check fails, deploy a new instance instead of returning an error
                    Polytone::deploy_on(chain.clone(), Empty {})?
                }
                Err(e) => return Err(e.into()), // Return any other error
            }
        }
        // If Polytone is not loaded, deploy a new one
        Err(_) => Polytone::deploy_on(chain.clone(), Empty {})?,
    };

    Ok(())
}

async fn ping_grpc(url_str: &str) -> anyhow::Result<()> {
    let parsed_url = Url::parse(url_str)?;

    let host = parsed_url
        .host_str()
        .ok_or_else(|| anyhow::anyhow!("No host in url"))?;

    let port = parsed_url.port_or_known_default().ok_or_else(|| {
        anyhow::anyhow!(
            "No port in url, and no default for scheme {:?}",
            parsed_url.scheme()
        )
    })?;
    let socket_addr = format!("{}:{}", host, port);

    let _ = TcpStream::connect(socket_addr);
    Ok(())
}

#[derive(Parser, Default, Debug)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    /// Network Id to deploy on
    #[arg(long)]
    network_id: String,
    /// Chain Id to deploy on
    #[arg(long)]
    chain_id: String,
    /// Address prefix
    #[arg(long)]
    address_prefix: String,
    /// Coin type, optional default 118u32
    #[arg(long)]
    coin_type: Option<u32>,
    /// Gas price, optional default 0.025
    #[arg(long)]
    gas_price: Option<f64>,
    /// GRPC URL
    #[arg(long)]
    grpc_url: String,
    /// Gas denom
    #[arg(long)]
    gas_denom: String,
}

fn main() {
    dotenv().ok();
    env_logger::init();

    use dotenv::dotenv;

    let args = Arguments::parse();

    let network_info: NetworkInfoOwned = NetworkInfoOwned {
        chain_name: args.network_id,
        pub_address_prefix: args.address_prefix,
        coin_type: args.coin_type.unwrap_or(118u32),
    };

    let chain_info: ChainInfoOwned = ChainInfoOwned {
        kind: ChainKind::Testnet,
        chain_id: args.chain_id,
        gas_denom: args.gas_denom,
        gas_price: args.gas_price.unwrap_or(0.025),
        grpc_urls: vec![args.grpc_url],
        network_info,
        lcd_url: None,
        fcd_url: None,
    };

    if let Err(ref err) = manual_deploy(chain_info) {
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
