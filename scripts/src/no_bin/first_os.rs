use abstract_core::objects::gov_type::GovernanceDetails;
use abstract_interface::Abstract;
use cw_orch::{
    networks::{ChainInfo, ChainKind, NetworkInfo},
    *,
};

use cw_orch::networks::kujira::KUJIRA_NETWORK;
use std::sync::Arc;
use tokio::runtime::Runtime;

pub const HARPOON_4: ChainInfo = ChainInfo {
    kind: ChainKind::Testnet,
    chain_id: "harpoon-4",
    gas_denom: "ukuji",
    gas_price: 0.025,
    grpc_urls: &["https://kujira-testnet-grpc.polkachu.com:11890"],
    chain_info: KUJIRA_NETWORK,
    lcd_url: None,
    fcd_url: None,
};

pub const ABSTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Script that registers the first Account in abstract (our Account)
pub fn first_os(network: ChainInfo) -> anyhow::Result<()> {
    // let network = LOCAL_JUNO;
    let rt = Arc::new(Runtime::new()?);
    let chain = DaemonBuilder::default()
        .handle(rt.handle())
        .chain(network)
        .build()?;
    let deployment = Abstract::new(chain.clone());

    // NOTE: this assumes that the deployment has been deployed

    deployment
        .account_factory
        .create_default_account(GovernanceDetails::Monarchy {
            monarch: chain.clone().sender().to_string(),
        })?;

    deployment.ans_host.update_all()?;

    Ok(())
}

fn main() {
    dotenv().ok();
    env_logger::init();

    use dotenv::dotenv;

    if let Err(ref err) = first_os(HARPOON_4) {
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
