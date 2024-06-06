use abstract_interface::Abstract;
use abstract_std::objects::gov_type::GovernanceDetails;
use cw_orch::{contract::Deploy, environment::ChainInfoOwned};
// ANCHOR: full_counter_example
use cw_orch_daemon::DaemonBuilder;
use tokio::runtime::Runtime;

// From https://github.com/CosmosContracts/juno/blob/32568dba828ff7783aea8cb5bb4b8b5832888255/docker/test-user.env#L2
const LOCAL_MNEMONIC: &str = "gesture spoil matrix shadow drift fluid canal frown define display awesome equal explain theme reject immune little lottery violin notice add start swift mechanic";

use cw_orch_core::environment::{ChainInfo, ChainKind, NetworkInfo};

pub const BITSONG_NETWORK: NetworkInfo = NetworkInfo {
    chain_name: "bitsong",
    pub_address_prefix: "bitsong",
    coin_type: 118u32,
};

pub const BITSONG_1: ChainInfo = ChainInfo {
    kind: ChainKind::Mainnet,
    chain_id: "bitsong-1",
    gas_denom: "ubtsg",
    gas_price: 0.025,
    grpc_urls: &["https://grpc.:443"],
    network_info: BITSONG_NETWORK,
    lcd_url: None,
    fcd_url: None,
};

pub const BOBNET: ChainInfo = ChainInfo {
    kind: ChainKind::Testnet,
    chain_id: "osmo-test-5",
    gas_denom: "uosmo",
    gas_price: 0.025,
    grpc_urls: &["https://grpc.osmotest5.osmosis.zone:443"],
    network_info: BITSONG_NETWORK,
    lcd_url: None,
    fcd_url: None,
};

pub const LOCAL_BITSONG: ChainInfo = ChainInfo {
    kind: ChainKind::Local,
    chain_id: "120u-1",
    gas_denom: "ubtsg",
    gas_price: 0.0026,
    grpc_urls: &["http://127.0.0.1:9090"],
    network_info: BITSONG_NETWORK,
    lcd_url: None,
    fcd_url: None,
};

fn manual_deploy(network: ChainInfoOwned) -> anyhow::Result<()> {
    let _rt = Runtime::new()?;
    let daemon = DaemonBuilder::default().chain(network).build()?;
    let wallet = daemon.wallet().address()?;
    // rt.block_on(assert_wallet_balance(vec![network.clone()]));

    let abs = Abstract::deploy_on(daemon.clone(), wallet.to_string())?;
    let account = abs.account_factory
        .create_default_account(GovernanceDetails::Monarchy {
            monarch: wallet.to_string(),
        })?;

        println!("{:?}", account.to_string());

    Ok(())
}

pub fn main() -> anyhow::Result<()> {
    env_logger::init();
    std::env::set_var("LOCAL_MNEMONIC", LOCAL_MNEMONIC);

    if let Err(ref err) = manual_deploy(LOCAL_BITSONG.into()) {
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

    Ok(())
}
// ANCHOR_END: full_counter_example
