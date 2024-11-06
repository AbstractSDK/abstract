use std::sync::Arc;

use abstract_std::native_addrs;
use cw_orch::{
    anyhow,
    daemon::{senders::CosmosSender, CosmosOptions},
    prelude::*,
};
pub const ABSTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
use dotenv::dotenv;
use networks::{LOCAL_JUNO, PION_1};

const CHAIN: ChainInfo = PION_1;

/// Run this script to get bytes or Address of the creator
fn main() -> anyhow::Result<()> {
    dotenv().ok();
    env_logger::init();

    // let cosmos_sender = cw_orch::daemon::RUNTIME.block_on(CosmosSender::new(
    //     &Arc::new(CHAIN.into()),
    //     CosmosOptions::default().mnemonic("clip hire initial neck maid actor venue client foam budget lock catalog sweet steak waste crater broccoli pipe steak sister coyote moment obvious choose"),
    // ))?;
    // let public_key = cosmos_sender
    //     .private_key
    //     .get_signer_public_key(&cosmos_sender.secp)
    //     .unwrap();
    // let single_key = public_key.single().unwrap();
    // println!("Signer public bytes: {:?}", single_key.to_bytes());
    // let signer = native_addrs::creator_address(CHAIN.network_info.pub_address_prefix)?;
    // println!("Signer Address: {signer}");

    Ok(())
}
