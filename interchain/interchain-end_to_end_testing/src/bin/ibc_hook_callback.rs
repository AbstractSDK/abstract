// This script is used for testing a connection between 4 chains
// This script checks ibc-hook memo implementation on ibc-client

use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use abstract_interchain_tests::{abstract_starship_interfaces, set_starship_env, JUNO, JUNO2};
use abstract_interface::{AccountDetails, AccountI};
use abstract_std::{
    account,
    ans_host::ExecuteMsgFns,
    ibc_client,
    objects::{TruncatedChainId, UncheckedChannelEntry},
    IBC_CLIENT, ICS20,
};
use anyhow::Result as AnyResult;
use cosmwasm_std::{to_json_binary, BankMsg};
use cw_orch::{
    daemon::{senders::CosmosSender, CosmosOptions},
    prelude::*,
};
use cw_orch_interchain::prelude::*;
use cw_orch_proto::tokenfactory::{create_denom, get_denom, mint};
use networks::ChainKind;

pub fn test_ibc_hook_callback() -> AnyResult<()> {
    dotenv::dotenv().ok();
    set_starship_env();
    env_logger::init();

    let starship = Starship::new(None).unwrap();
    let interchain = starship.interchain_env();

    let juno = interchain.get_chain(JUNO).unwrap();
    let juno2 = interchain.get_chain(JUNO2).unwrap();

    // // Using chainkind local so we can use mnemonic from env
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

    // // Create a channel between the 2 chains for the transfer ports
    // // JUNO>JUNO2
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

    let (abstr_juno, abstr_juno2) = abstract_starship_interfaces(
        &interchain,
        &juno_abstract_deployer,
        &juno2_abstract_deployer,
    )?;

    let juno_sender = juno.sender_addr().to_string();

    // Register this channel with the abstract ibc implementation for sending tokens
    abstr_juno
        .ans_host
        .call_as(&juno_abstract_deployer)
        .update_channels(
            vec![(
                UncheckedChannelEntry {
                    connected_chain: TruncatedChainId::from_chain_id(JUNO2).to_string(),
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
    let origin_account = AccountI::create_default_account(
        &abstr_juno,
        abstract_client::GovernanceDetails::Monarchy {
            monarch: juno_sender.clone(),
        },
    )?;
    origin_account.set_ibc_status(true)?;
    origin_account.create_remote_account(
        AccountDetails::default(),
        TruncatedChainId::from_chain_id(JUNO2),
    )?;
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
    let denom = get_denom(&juno, token_subdenom.as_str());
    // Mint Denom
    mint(
        &juno,
        &origin_account.addr_str()?,
        token_subdenom.as_str(),
        test_amount,
    )?;

    // Whitelist ibc-client to get callback
    let ibc_client_addr = origin_account.module_address(IBC_CLIENT)?;
    origin_account.update_whitelist(vec![ibc_client_addr.to_string()], vec![])?;
    let tx_response = origin_account.execute_on_module(
        IBC_CLIENT,
        ibc_client::ExecuteMsg::SendFundsWithActions {
            host_chain: TruncatedChainId::from_chain_id(JUNO2),
            actions: vec![to_json_binary(&account::ExecuteMsg::<Empty>::Execute {
                msgs: vec![BankMsg::Send {
                    to_address: juno_sender,
                    amount: vec![Coin::new(5_000_000_u128, denom.clone())],
                }
                .into()],
            })?],
        },
        vec![Coin::new(100_000_000_u128, denom.clone())],
    )?;
    interchain.await_and_check_packets(JUNO, tx_response)?;

    let balance = juno.balance(&juno.sender_addr(), Some(denom.clone()))?;
    assert_eq!(balance, vec![Coin::new(5_000_000_u128, denom)]);
    println!("We got a callback! Result: {balance:?}");
    Ok(())
}

pub fn main() {
    test_ibc_hook_callback().unwrap();
}
