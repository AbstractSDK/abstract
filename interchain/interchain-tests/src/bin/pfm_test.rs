// This script is used for testing a connection between 2 chains
// This script sets up tokens and channels between transfer ports to transfer those tokens
// This also mints tokens to the chain sender for future interactions

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use abstract_interchain_tests::{
    interchain_accounts::{create_test_remote_account, set_env},
    JUNO,
};
use abstract_interface::{Abstract, AbstractAccount, ProxyQueryFns};
use abstract_sdk::{IbcMemoBuilder, PacketForwardMiddlewareBuilder};
use abstract_std::{
    ans_host::ExecuteMsgFns,
    objects::{TruncatedChainId, UncheckedChannelEntry},
    ICS20, PROXY,
};
use anyhow::Result as AnyResult;
use cosmwasm_std::{coin, coins};
use cw_orch::{daemon::RUNTIME, prelude::*};
use cw_orch_interchain::prelude::*;
use cw_orch_proto::tokenfactory::{create_denom, get_denom, mint};
use ibc_relayer_types::core::ics24_host::identifier::PortId;

// Note: Truncated chain id have to be different
pub const JUNO2: &str = "junotwo-1";
pub const JUNO3: &str = "junothree-1";
pub const JUNO4: &str = "junofour-1";

pub fn test_pfm() -> AnyResult<()> {
    dotenv::dotenv().ok();
    set_env();
    env_logger::init();

    let starship = Starship::new(None).unwrap();
    let interchain = starship.interchain_env();

    let juno = interchain.get_chain(JUNO).unwrap();
    let juno2 = interchain.get_chain(JUNO2).unwrap();
    let juno3 = interchain.get_chain(JUNO3).unwrap();
    let juno4 = interchain.get_chain(JUNO4).unwrap();

    let abstr_juno = Abstract::deploy_on(juno.clone(), juno.sender_addr().to_string())?;
    let abstr_juno2 = Abstract::deploy_on(juno2.clone(), juno2.sender_addr().to_string())?;

    // let abstr_juno = Abstract::load_from(juno.clone())?;
    // let abstr_osmosis = Abstract::load_from(osmosis.clone())?;
    // let abstr_stargaze = Abstract::load_from(stargaze.clone())?;

    // JUNO>JUNO2>JUNO3>JUNO4
    // for forwarding we only need connection between 2 chains, rest does pfm
    abstr_juno.connect_to(&abstr_juno2, &interchain)?;

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

    // Create a channel between the 3 chains for the transfer ports
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

    // Register this channel with the abstract ibc implementation for sending tokens
    abstr_juno.ans_host.update_channels(
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
        create_test_remote_account(&abstr_juno, JUNO, JUNO2, &interchain, None)?;

    // Get the ibc client address
    let remote_account = AbstractAccount::new(&abstr_juno2, remote_account_id.clone());
    let client = remote_account.proxy.config()?;

    log::info!("client adddress {:?}", client);

    // Send funds to the remote account
    RUNTIME.block_on(juno.sender().bank_send(
        &origin_account.proxy.addr_str()?,
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

    let memo = PacketForwardMiddlewareBuilder::new(juno2_juno3_channel_port_juno2)
        .next(PacketForwardMiddlewareBuilder::new(
            juno3_juno4_channel_port_juno3,
        ))
        .build()?;
    // We send from osmosis to juno funds with pfm memo that includes juno-stargaze channel
    let send_funds_tx = origin_account.manager.execute_on_module(
        PROXY,
        abstract_std::proxy::ExecuteMsg::IbcAction {
            msg: abstract_std::ibc_client::ExecuteMsg::SendFunds {
                host_chain: TruncatedChainId::from_chain_id(JUNO2),
                funds: coins(100_000_000_000, get_denom(&juno, token_subdenom.as_str())),
                memo: Some(memo),
            },
        },
    )?;
    std::thread::sleep(Duration::from_secs(15));
    // interchain.await_and_check_packets(JUNO, send_funds_tx)?;

    // Verify the funds have been received
    let balance = juno4.bank_querier().balance(juno4.sender_addr(), None)?;

    log::info!("juno4 balance, {:?}", balance);

    Ok(())
}

pub fn main() {
    test_pfm().unwrap();
}
