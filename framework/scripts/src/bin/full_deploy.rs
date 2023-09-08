use std::fs::remove_file;

use abstract_core::objects::gov_type::GovernanceDetails;
use abstract_interface::Abstract;

use abstract_interface_scripts::assert_wallet_balance;
use clap::Parser;
use cw_orch::{
    deploy::Deploy,
    prelude::{
        networks::{parse_network, ChainInfo, NetworkInfo},
        *,
    },
};
use tokio::runtime::Runtime;
use crate::networks::ChainKind;

pub const ABSTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const SEI_NETWORK: NetworkInfo = NetworkInfo {
    id: "sei",
    pub_address_prefix: "sei",
    coin_type: 118u32,
};

pub const PACIFIC_1: ChainInfo = ChainInfo {
    kind: ChainKind::Mainnet,
    chain_id: "pacific-1",
    gas_denom: "usei",
    gas_price: 0.1,
    grpc_urls: &["http://sei-grpc.polkachu.com:11990"],
    network_info: SEI_NETWORK,
    lcd_url: None,
    fcd_url: None,
};

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
        let deployment = Abstract::deploy_on(chain, Empty {})?;

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


    if let Err(ref err) = full_deploy(vec![PACIFIC_1]) {
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
