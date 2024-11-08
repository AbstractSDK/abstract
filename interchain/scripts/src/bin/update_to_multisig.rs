use std::sync::Arc;

use abstract_interface::Abstract;
use cw_orch::prelude::*;
use cw_orch_daemon::{CosmosOptions, Wallet};
use networks::LOCAL_JUNO;

fn main() -> anyhow::Result<()> {
    dotenv().ok();
    env_logger::init();
    use dotenv::dotenv;

    let abstract_mnemonic =
        std::env::var("ABSTRACT_MNEMONIC").expect("Fill your abstract mnemonic");
    // Fill members
    let members = vec![cw4::Member {
        addr: "juno14cl2dthqamgucg9sfvv4relp3aa83e40rg8jrz".to_string(),
        weight: 1,
    }];
    assert!(!members.is_empty(), "Fill multisig members first");

    // Change network
    let network = LOCAL_JUNO;

    let chain = DaemonBuilder::new(network).build()?;
    let proposal_creator = chain.rt_handle.block_on(Wallet::new(
        &Arc::new(chain.chain_info().clone()),
        CosmosOptions::default().mnemonic(abstract_mnemonic),
    ))?;

    let mut deployment = Abstract::load_from(chain.clone())?;
    deployment.update_admin_to_multisig(
        chain.sender_addr().to_string(),
        members,
        &proposal_creator,
        [],
    )?;

    Ok(())
}
