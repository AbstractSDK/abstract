use std::net::TcpStream;

use abstract_cw_staking::{interface::CwStakingAdapter, CW_STAKING_ADAPTER_ID};
use abstract_dex_adapter::{interface::DexAdapter, msg::DexInstantiateMsg, DEX_ADAPTER_ID};
use abstract_interface::*;
use challenge_app::{contract::CHALLENGE_APP_ID, Challenge};
use clap::Parser;
use cosmwasm_std::Decimal;
use cw_orch::{daemon::networks::parse_network, prelude::*};
use dca_app::{contract::DCA_APP_ID, DCA};
use etf_app::{contract::interface::Etf, ETF_APP_ID};
use reqwest::Url;
use tokio::runtime::Runtime;

pub const ABSTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// Run "cargo run --example download_wasms" in the `abstract-interfaces` package before deploying!
fn full_deploy() -> anyhow::Result<()> {
    let rt = Runtime::new()?;

    let deployment = Abstract::<Daemon>::get_all_deployed_chains();
    let networks: Vec<ChainInfo> = deployment
        .iter()
        .map(|n| parse_network(n).unwrap())
        .collect();

    for network in networks {
        let chain = DaemonBuilder::new(network.clone())
            .handle(rt.handle())
            .build()?;

        // Deploy Adapters
        CwStakingAdapter::new(CW_STAKING_ADAPTER_ID, chain.clone()).deploy(
            abstract_cw_staking::contract::CONTRACT_VERSION.parse()?,
            Empty {},
            DeployStrategy::Try,
        )?;
        DexAdapter::new(DEX_ADAPTER_ID, chain.clone()).deploy(
            abstract_dex_adapter::contract::CONTRACT_VERSION.parse()?,
            DexInstantiateMsg {
                recipient_account: 0,
                swap_fee: Decimal::permille(3),
            },
            DeployStrategy::Try,
        )?;

        // Deploy apps
        Etf::new(ETF_APP_ID, chain.clone()).deploy(
            etf_app::contract::CONTRACT_VERSION.parse()?,
            DeployStrategy::Try,
        )?;

        DCA::new(DCA_APP_ID, chain.clone()).deploy(
            dca_app::contract::DCA_APP_VERSION.parse()?,
            DeployStrategy::Try,
        )?;
        Challenge::new(CHALLENGE_APP_ID, chain.clone()).deploy(
            challenge_app::contract::CHALLENGE_APP_VERSION.parse()?,
            DeployStrategy::Try,
        )?;
    }
    Ok(())
}

#[allow(unused)]
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
    #[arg(short, long, value_delimiter = ' ', num_args = 1..)]
    network_ids: Vec<String>,
}

fn main() {
    dotenv().ok();
    env_logger::init();

    use dotenv::dotenv;

    if let Err(ref err) = full_deploy() {
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
