// This script is used for testing a connection between 2 chains
// This script sets up tokens and channels between transfer ports to transfer those tokens
// This also mints tokens to the chain sender for future interactions

use std::{
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use abstract_interface::{AccountI, AccountQueryFns};
use abstract_sdk::PfmMemoBuilder;
use abstract_std::{
    ans_host::ExecuteMsgFns,
    objects::{TruncatedChainId, UncheckedChannelEntry},
    IBC_CLIENT, ICS20,
};
use abstract_tests::interchain::{
    abstract_starship_interfaces, create_test_remote_account, set_starship_env, JUNO,
};
use anyhow::Result as AnyResult;
use cosmwasm_std::{coin, coins};
use cw_orch::{
    daemon::{networks::ChainKind, senders::CosmosSender, CosmosOptions, RUNTIME},
    prelude::*,
};
use cw_orch_interchain::prelude::*;
use cw_orch_proto::tokenfactory::{create_denom, get_denom, mint};

// Note: Truncated chain id have to be different
pub const JUNO2: &str = "junotwo-1";
pub const JUNO3: &str = "junothree-1";
pub const JUNO4: &str = "junofour-1";

pub fn test_pfm() -> AnyResult<()> {
    dotenv::dotenv().ok();
    set_starship_env();
    env_logger::init();

    let starship = Starship::new(None).unwrap();
    let interchain = starship.interchain_env();

    let juno = interchain.get_chain(JUNO).unwrap();
    let juno2 = interchain.get_chain(JUNO2).unwrap();
    let juno4 = interchain.get_chain(JUNO4).unwrap();

    // Using chainkind local so we can use mnemonic from env
    let juno_chain_info = ChainInfoOwned {
        kind: ChainKind::Local,
        ..juno.chain_info().clone()
    };
    let juno2_chain_info = ChainInfoOwned {
        kind: ChainKind::Local,
        ..juno2.chain_info().clone()
    };

    let juno_abstract_deployer = juno.rt_handle.block_on(CosmosSender::new(
        &Arc::new(juno_chain_info),
        CosmosOptions::default(),
    ))?;
    let juno2_abstract_deployer = juno2.rt_handle.block_on(CosmosSender::new(
        &Arc::new(juno2_chain_info),
        CosmosOptions::default(),
    ))?;

    // Create a channel between the 4 chains for the transfer ports
    // JUNO>JUNO2>JUNO3>JUNO4
    let juno_juno2_channel = interchain
        .create_channel(
            JUNO,
            JUNO2,
            &PortId::transfer(),
            &PortId::transfer(),
            "ics20-1",
            Some(cosmwasm_std::IbcOrder::Unordered),
        )?
        .interchain_channel;

    let juno2_juno3_channel = interchain
        .create_channel(
            JUNO2,
            JUNO3,
            &PortId::transfer(),
            &PortId::transfer(),
            "ics20-1",
            Some(cosmwasm_std::IbcOrder::Unordered),
        )?
        .interchain_channel;

    let juno3_juno4_channel = interchain
        .create_channel(
            JUNO3,
            JUNO4,
            &PortId::transfer(),
            &PortId::transfer(),
            "ics20-1",
            Some(cosmwasm_std::IbcOrder::Unordered),
        )?
        .interchain_channel;

    let (abstr_juno, abstr_juno2) = abstract_starship_interfaces(
        &interchain,
        &juno_abstract_deployer,
        &juno2_abstract_deployer,
    )?;

    let sender = juno.sender_addr().to_string();

    let test_amount: u128 = 100_000_000_000;
    let token_subdenom = format!(
        "testtoken{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );

    // Create Denom
    create_denom(&juno, token_subdenom.as_str())?;

    // Mint Denom
    mint(&juno, sender.as_str(), token_subdenom.as_str(), test_amount)?;

    // Register this channel with the abstract ibc implementation for sending tokens
    abstr_juno
        .ans_host
        .call_as(&juno_abstract_deployer)
        .update_channels(
            vec![(
                UncheckedChannelEntry {
                    connected_chain: "junotwo".to_string(),
                    protocol: ICS20.to_string(),
                },
                juno_juno2_channel
                    .get_chain(JUNO)?
                    .channel
                    .unwrap()
                    .to_string(),
            )],
            vec![],
        )?;

    // Create a test account + Remote account
    let (origin_account, remote_account_id) =
        create_test_remote_account(&abstr_juno, JUNO, JUNO2, &interchain, vec![])?;

    // Get the ibc client address
    let remote_account = AccountI::load_from(&abstr_juno2, remote_account_id.clone())?;
    let client = remote_account.config()?;

    log::info!("client adddress {:?}", client);

    // Send funds to the remote account
    RUNTIME.block_on(juno.sender().bank_send(
        &origin_account.address()?,
        vec![coin(test_amount, get_denom(&juno, token_subdenom.as_str()))],
    ))?;
    let juno2_juno3_channel_port_juno2 = juno2_juno3_channel
        .get_chain(JUNO2)
        .unwrap()
        .channel
        .unwrap()
        .to_string();
    let juno3_juno4_channel_port_juno3 = juno3_juno4_channel
        .get_chain(JUNO3)
        .unwrap()
        .channel
        .unwrap()
        .to_string();

    let memo = PfmMemoBuilder::new(juno2_juno3_channel_port_juno2)
        .hop(juno3_juno4_channel_port_juno3)
        .build(juno4.sender_addr())?;
    origin_account.execute_on_module(
        IBC_CLIENT,
        abstract_std::ibc_client::ExecuteMsg::SendFunds {
            host_chain: TruncatedChainId::from_chain_id(JUNO2),
            memo: Some(memo),
            receiver: None,
        },
        coins(100_000_000_000, get_denom(&juno, token_subdenom.as_str())),
    )?;
    log::info!("waiting for pfm bank send to finish");
    std::thread::sleep(Duration::from_secs(15));

    // Verify the funds have been received
    let balance = juno4.bank_querier().balance(&juno4.sender_addr(), None)?;

    log::info!("juno4 balance, {:?}", balance);

    Ok(())
}

pub fn main() {
    test_pfm().unwrap();
}
