// This script is used for testing a connection between 2 chains
// This script sets up tokens and channels between transfer ports to transfer those tokens
// This also mints tokens to the chain sender for future interactions

use std::time::{SystemTime, UNIX_EPOCH};

use abstract_interchain_tests::{
    interchain_accounts::{create_test_remote_account, set_env},
    JUNO, OSMOSIS, STARGAZE,
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

pub fn test_pfm() -> AnyResult<()> {
    dotenv::dotenv().ok();
    set_env();
    env_logger::init();

    let starship = Starship::new(None).unwrap();
    let interchain = starship.interchain_env();

    let juno = interchain.get_chain(JUNO).unwrap();
    let osmosis = interchain.get_chain(OSMOSIS).unwrap();
    let stargaze = interchain.get_chain(STARGAZE).unwrap();

    let abstr_juno = Abstract::deploy_on(juno.clone(), juno.sender_addr().to_string())?;
    let abstr_osmosis = Abstract::deploy_on(osmosis.clone(), osmosis.sender_addr().to_string())?;
    // let abstr_stargaze = Abstract::deploy_on(stargaze.clone(), stargaze.sender_addr().to_string())?;

    // let abstr_juno = Abstract::load_from(juno.clone())?;
    // let abstr_osmosis = Abstract::load_from(osmosis.clone())?;
    // let abstr_stargaze = Abstract::load_from(stargaze.clone())?;

    // OSMOSIS>JUNO>STARGAZE
    // for forwarding we only need connection between 2 chains, rest does pfm
    abstr_osmosis.connect_to(&abstr_juno, &interchain)?;
    // abstr_juno.connect_to(&abstr_stargaze, &interchain)?;

    let sender = osmosis.sender_addr().to_string();

    let test_amount: u128 = 100_000_000_000;
    let token_subdenom = format!(
        "testtoken{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );

    // Create Denom
    create_denom(&osmosis, token_subdenom.as_str())?;

    // Mint Denom
    mint(
        &osmosis,
        sender.as_str(),
        token_subdenom.as_str(),
        test_amount,
    )?;

    // Create a channel between the 3 chains for the transfer ports
    let osmosis_juno_channel = interchain
        .create_channel(
            OSMOSIS,
            JUNO,
            &PortId::transfer(),
            &PortId::transfer(),
            "ics20-1",
            Some(cosmwasm_std::IbcOrder::Unordered),
        )?
        .interchain_channel;

    let juno_stargaze_channel = interchain
        .create_channel(
            JUNO,
            STARGAZE,
            &PortId::transfer(),
            &PortId::transfer(),
            "ics20-1",
            Some(cosmwasm_std::IbcOrder::Unordered),
        )?
        .interchain_channel;

    // Register this channel with the abstract ibc implementation for sending tokens
    abstr_osmosis.ans_host.update_channels(
        vec![(
            UncheckedChannelEntry {
                connected_chain: "juno".to_string(),
                protocol: ICS20.to_string(),
            },
            osmosis_juno_channel
                .get_chain(OSMOSIS)?
                .channel
                .unwrap()
                .to_string(),
        )],
        vec![],
    )?;

    // Create a test account + Remote account

    let (origin_account, remote_account_id) =
        create_test_remote_account(&abstr_osmosis, OSMOSIS, JUNO, &interchain, None)?;

    // Get the ibc client address
    let remote_account = AbstractAccount::new(&abstr_juno, remote_account_id.clone());
    let client = remote_account.proxy.config()?;

    log::info!("client adddress {:?}", client);

    // Send funds to the remote account
    RUNTIME.block_on(osmosis.sender().bank_send(
        &origin_account.proxy.addr_str()?,
        vec![coin(
            test_amount,
            get_denom(&osmosis, token_subdenom.as_str()),
        )],
    ))?;
    let juno_stargaze_channel_port_juno = juno_stargaze_channel
        .get_chain(JUNO)
        .unwrap()
        .channel
        .unwrap()
        .to_string();

    let memo = PacketForwardMiddlewareBuilder::new(juno_stargaze_channel_port_juno)
        .receiver(stargaze.sender_addr())
        .build()?;
    // We send from osmosis to juno funds with pfm memo that includes juno-stargaze channel
    let send_funds_tx = origin_account.manager.execute_on_module(
        PROXY,
        abstract_std::proxy::ExecuteMsg::IbcAction {
            msg: abstract_std::ibc_client::ExecuteMsg::SendFunds {
                host_chain: TruncatedChainId::from_chain_id(JUNO),
                funds: coins(
                    100_000_000_000,
                    get_denom(&osmosis, token_subdenom.as_str()),
                ),
                memo: Some(memo),
            },
        },
    )?;
    interchain.await_and_check_packets(JUNO, send_funds_tx)?;

    // Verify the funds have been received
    let balance = stargaze
        .bank_querier()
        .balance(stargaze.sender_addr(), None)?;

    log::info!("stargaze balance, {:?}", balance);

    Ok(())
}

pub fn main() {
    test_pfm().unwrap();
}
