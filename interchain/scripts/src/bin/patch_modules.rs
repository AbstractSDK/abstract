use abstract_core::objects::module::ModuleInfo;
use abstract_cw_staking::{interface::CwStakingAdapter, CW_STAKING_ADAPTER_ID};
use abstract_dex_adapter::{interface::DexAdapter, msg::DexInstantiateMsg, DEX_ADAPTER_ID};
use abstract_interface::*;
use challenge_app::{contract::CHALLENGE_APP_ID, Challenge};
use clap::Parser;
use cosmwasm_std::Decimal;
use cw_orch::daemon::networks::OSMOSIS_1;
use cw_orch::daemon::ChainInfo;
use cw_orch::{daemon::networks::parse_network, prelude::*};
use dca_app::{contract::DCA_APP_ID, DCA};
use etf_app::{contract::interface::Etf, ETF_APP_ID};
use reqwest::Url;
use std::net::TcpStream;
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

    for mut network in networks {
        if network.chain_id == "uni-6" {
            continue;
        } else if network.chain_id == "neutron-1" {
            network.gas_price = 0.075;
        }

        network = OSMOSIS_1;

        let chain = DaemonBuilder::default()
            .handle(rt.handle())
            .chain(network.clone())
            .build()?;

        DexAdapter::new(DEX_ADAPTER_ID, chain.clone()).deploy(
            abstract_dex_adapter::contract::CONTRACT_VERSION.parse()?,
            DexInstantiateMsg {
                recipient_account: 0,
                swap_fee: Decimal::permille(3),
            },
            DeployStrategy::Try,
        )?;

        let abstr = Abstract::load_from(chain.clone())?;
        // yank 0.20
        abstr
            .version_control
            .yank_module(ModuleInfo::from_id(DEX_ADAPTER_ID, "0.20.0".into())?);
        // approve 0.20.1
        abstr.version_control.approve_any_abstract_modules()?;
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
