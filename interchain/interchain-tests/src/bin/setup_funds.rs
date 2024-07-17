// This script is used for testing a connection between 2 chains
// This script sets up tokens and channels between transfer ports to transfer those tokens
// This also mints tokens to the chain sender for future interactions

use std::time::{SystemTime, UNIX_EPOCH};

use abstract_interchain_tests::{
    interchain_accounts::{create_test_remote_account, set_env},
    JUNO, STARGAZE,
};
use abstract_interface::{Abstract, AbstractAccount, ProxyQueryFns};
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

pub fn test_send_funds() -> AnyResult<()> {
    env_logger::init();

    set_env();

    let starship = Starship::new(None).unwrap();
    let interchain = starship.interchain_env();

    let juno = interchain.get_chain(JUNO).unwrap();
    let stargaze = interchain.get_chain(STARGAZE).unwrap();

    let abstr_stargaze = Abstract::deploy_on(stargaze.clone(), stargaze.sender_addr().to_string())?;
    let abstr_juno = Abstract::deploy_on(juno.clone(), juno.sender_addr().to_string())?;
    abstr_juno.connect_to(&abstr_stargaze, &interchain)?;
    // let abstr_stargaze: Abstract<Daemon> = Abstract::load_from(stargaze.clone())?;
    // let abstr_juno: Abstract<Daemon> = Abstract::load_from(juno.clone())?;

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

    // Create a channel between the 2 chains for the transfer ports
    let interchain_channel = interchain
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
    abstr_juno.ans_host.update_channels(
        vec![(
            UncheckedChannelEntry {
                connected_chain: "stargaze".to_string(),
                protocol: ICS20.to_string(),
            },
            interchain_channel
                .get_chain(JUNO)?
                .channel
                .unwrap()
                .to_string(),
        )],
        vec![],
    )?;

    // Create a test account + Remote account

    let (origin_account, remote_account_id) =
        create_test_remote_account(&abstr_juno, JUNO, STARGAZE, &interchain, None)?;
    // let account_config = osmo_abstr.account.manager.config()?;
    // let account_id = AccountId::new(
    //     account_config.account_id.seq(),
    //     AccountTrace::Remote(vec![TruncatedChainId::from("osmosis")]),
    // )?;

    // Get the ibc client address
    let remote_account = AbstractAccount::new(&abstr_stargaze, remote_account_id.clone());
    let client = remote_account.proxy.config()?;

    log::info!("client adddress {:?}", client);

    // Send funds to the remote account
    RUNTIME.block_on(juno.sender().bank_send(
        &origin_account.proxy.addr_str()?,
        vec![coin(test_amount, get_denom(&juno, token_subdenom.as_str()))],
    ))?;
    let send_funds_tx = origin_account.manager.execute_on_module(
        PROXY,
        abstract_std::proxy::ExecuteMsg::IbcAction {
            msg: abstract_std::ibc_client::ExecuteMsg::SendFunds {
                host_chain: TruncatedChainId::from_chain_id(STARGAZE),
                funds: coins(test_amount, get_denom(&juno, token_subdenom.as_str())),
                memo: Some("sent_some_tokens".to_owned()),
            },
        },
    )?;

    let response = interchain.await_packets(JUNO, send_funds_tx)?;
    response.into_result()?;
    let memo = response.event_attr_value("fungible_token_packet", "memo")?;
    log::info!("Got memo: {memo}");

    // Verify the funds have been received
    let remote_account_config = abstr_stargaze
        .version_control
        .get_account(remote_account_id.clone())?;

    let balance = stargaze
        .bank_querier()
        .balance(remote_account_config.proxy, None)?;

    log::info!("juno balance, {:?}", balance);

    Ok(())
}

pub fn main() {
    test_send_funds().unwrap();
}
