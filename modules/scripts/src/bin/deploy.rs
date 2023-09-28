use abstract_cw_staking::{interface::CwStakingAdapter, CW_STAKING};
use abstract_dex_adapter::{interface::DexAdapter, msg::DexInstantiateMsg, EXCHANGE};
use abstract_interface::{Abstract, AdapterDeployer, AppDeployer};
use challenge_app::{contract::CHALLENGE_APP_ID, ChallengeApp};
use cosmwasm_std::Decimal;
use dca_app::{contract::DCA_APP_ID, DCAApp};
use etf_app::{contract::interface::EtfApp, ETF_ID};
use reqwest::Url;
use semver::Version;
use std::net::TcpStream;

use clap::Parser;
use cw_orch::{
    deploy::Deploy,
    prelude::{
        networks::{parse_network, ChainInfo, JUNO_1},
        *,
    },
};
use tokio::runtime::Runtime;

pub const ABSTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// Run "cargo run --example download_wasms" in the `abstract-interfaces` package before deploying!
fn full_deploy() -> anyhow::Result<()> {
    let rt = Runtime::new()?;

    let version: Version = ABSTRACT_VERSION.parse().unwrap();

    let chain = DaemonBuilder::default()
        .handle(rt.handle())
        .chain(JUNO_1.clone())
        .build()?;

    let deployment = Abstract::load_from(chain)
        .unwrap()
        .get_all_deployed_chains();
    let networks: Vec<ChainInfo> = deployment.iter().map(|n| parse_network(n)).collect();

    for network in networks {
        let chain = DaemonBuilder::default()
            .handle(rt.handle())
            .chain(network.clone())
            .build()?;
        let _deployment = Abstract::load_from(chain.clone())?;

        let _staking = CwStakingAdapter::new(CW_STAKING, chain.clone())
            .maybe_deploy(version.clone(), Empty {})?;
        let _dex = DexAdapter::new(EXCHANGE, chain.clone()).maybe_deploy(
            version.clone(),
            DexInstantiateMsg {
                recipient_account: 0,
                swap_fee: Decimal::permille(3),
            },
        )?;
        let _etf = EtfApp::new(ETF_ID, chain.clone()).maybe_deploy(version.clone())?;
        let _dca = DCAApp::new(DCA_APP_ID, chain.clone()).maybe_deploy(version.clone())?;
        let _challenge =
            ChallengeApp::new(CHALLENGE_APP_ID, chain.clone()).maybe_deploy(version.clone())?;
    }
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
