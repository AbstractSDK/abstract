// This scripts helps create a test environment for sending native tokens over an IBC connection between 2 chains
// This is only used with starhip, so this could be intergated into starthip in the future
// What needs to be done here is :
// 1. Create a token on chain 1 using token factory
// Create a channel between the 2 transfer ports of the 2 blockchains
// Test transfering a token back and forth on the 2 chains

use std::time::{SystemTime, UNIX_EPOCH};

use abstract_tests::interchain::{JUNO, STARGAZE};
use anyhow::Result as AnyResult;
use cosmwasm_std::{coin, Uint128};
use cw_orch::prelude::queriers::Ibc;
use cw_orch::prelude::*;
use cw_orch_interchain::prelude::*;
use cw_orch_proto::tokenfactory::{
    create_denom, create_transfer_channel, get_denom, mint, transfer_tokens,
};
use ibc_relayer_types::core::ics24_host::identifier::PortId;

pub fn token_bridge() -> AnyResult<()> {
    env_logger::init();
    let rt: tokio::runtime::Runtime = tokio::runtime::Runtime::new().unwrap();

    let interchain = Starship::new(None)?.interchain_env();

    let juno = interchain.get_chain(JUNO).unwrap();
    let stargaze = interchain.get_chain(STARGAZE).unwrap();

    let sender = juno.sender_addr().to_string();
    let receiver = stargaze.sender_addr().to_string();

    let test_amount: u128 = 100_000;
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
    let interchain_channel = create_transfer_channel(JUNO, STARGAZE, &interchain)?;

    // Transfer to the address on the remote chain
    transfer_tokens(
        &juno,
        receiver.as_str(),
        &coin(test_amount, get_denom(&juno, token_subdenom.as_str())),
        &interchain,
        &interchain_channel,
        None,
        None,
    )
    .unwrap()
    .assert()
    .unwrap();

    // Get the denom from the trace on the receiving chain
    let trace = format!(
        "{}/{}/{}",
        PortId::transfer(),
        interchain_channel
            .get_chain(STARGAZE)
            .unwrap()
            .channel
            .unwrap(),
        get_denom(&juno, token_subdenom.as_str())
    );
    let ibc: Ibc = stargaze.querier();
    let hash = rt.block_on(ibc._denom_hash(trace)).unwrap();
    let denom = format!("ibc/{}", hash);

    // Get balance on the remote chain
    let balance = stargaze
        .bank_querier()
        .balance(&stargaze.sender_addr(), Some(denom.clone()))
        .unwrap();

    assert_eq!(balance[0].amount, Uint128::from(test_amount));

    // Send all back
    transfer_tokens(
        &stargaze,
        sender.as_str(),
        &coin(test_amount, denom.clone()),
        &interchain,
        &interchain_channel,
        None,
        None,
    )
    .unwrap()
    .assert()
    .unwrap();

    let balance = stargaze
        .bank_querier()
        .balance(&stargaze.sender_addr(), Some(denom.clone()))
        .unwrap();

    assert_eq!(balance[0].amount, Uint128::zero());

    Ok(())
}

pub fn main() {
    token_bridge().unwrap()
}
