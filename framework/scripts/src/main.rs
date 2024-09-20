use std::{env::set_var, sync::Arc};

use abstract_client::AbstractClient;
use cw_orch::{
    daemon::{
        networks::{xion::XION_NETWORK, XION_TESTNET_1},
        senders::CosmosSender,
        CosmosOptions, Daemon, TxSender, RUNTIME,
    },
    prelude::*,
};
use networks::ChainKind;

const LOCAL_MNEMONIC: &str = "clinic tube choose fade collect fish original recipe pumpkin fantasy enrich sunny pattern regret blouse organ april carpet guitar skin work moon fatigue hurdle";

pub const LOCAL_XION: ChainInfo = ChainInfo {
    kind: ChainKind::Local,
    chain_id: "xion-devnet-1",
    gas_denom: "uxion",
    gas_price: 0.03,
    grpc_urls: &["http://localhost:9090"],
    network_info: XION_NETWORK,
    lcd_url: None,
    fcd_url: None,
};

fn main() -> anyhow::Result<()> {
    set_var("RUST_LOG", "info");
    env_logger::init();

    let xiond = Daemon::builder(LOCAL_XION)
        .build_sender(CosmosOptions::default().mnemonic(LOCAL_MNEMONIC))?;

    let wallet = xiond.sender();
    
    let abstr = AbstractClient::builder(xiond).build()?;

    let account = abstr.account_builder().build()?;



    Ok(())
}
