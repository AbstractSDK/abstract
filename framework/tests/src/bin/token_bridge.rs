// This scripts helps create a test environment for sending native tokens over an IBC connection between 2 chains
// This is only used with starhip, so this could be intergated into starthip in the future
// What needs to be done here is :
// 1. Create a token on chain 1 using token factory
// Create a channel between the 2 transfer ports of the 2 blockchains
// Test transfering a token back and forth on the 2 chains

use std::time::{SystemTime, UNIX_EPOCH};

use abstract_interface_integration_tests::{
    tokenfactory::{create_denom, create_transfer_channel, get_denom, mint, transfer_tokens},
    JUNO, OSMOSIS,
};

use cosmwasm_std::coin;
use cw_orch::{
    prelude::{
        queriers::{Bank, Ibc},
        InterchainEnv, TxHandler,
    },
    starship::Starship,
};
use ibc_relayer_types::core::ics24_host::identifier::PortId;

pub fn main() {
    env_logger::init();
    let rt: tokio::runtime::Runtime = tokio::runtime::Runtime::new().unwrap();

    let starship = Starship::new(rt.handle().to_owned(), None).unwrap();
    let interchain: InterchainEnv = starship.interchain_env();

    let juno = interchain.daemon(JUNO).unwrap();
    let osmosis = interchain.daemon(OSMOSIS).unwrap();

    let sender = juno.sender().to_string();
    let receiver = osmosis.sender().to_string();

    let test_amount: u128 = 100_000;
    let token_subdenom = format!(
        "testtoken{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );

    // Create Denom
    rt.block_on(create_denom(&juno, token_subdenom.as_str()))
        .unwrap();

    // Mint Denom
    rt.block_on(mint(
        &juno,
        sender.as_str(),
        token_subdenom.as_str(),
        test_amount,
    ))
    .unwrap();

    // Create a channel between the 2 chains for the transfer ports
    let interchain_channel = rt
        .block_on(create_transfer_channel(JUNO, OSMOSIS, &starship))
        .unwrap();

    // Transfer to the address on the remote chain
    transfer_tokens(
        &rt,
        &juno,
        receiver.as_str(),
        &coin(test_amount, get_denom(&juno, token_subdenom.as_str())),
        &interchain_channel,
        None,
    )
    .unwrap();

    // Get the denom from the trace on the receiving chain
    let trace = format!(
        "{}/{}/{}",
        PortId::transfer(),
        interchain_channel
            .get_chain(OSMOSIS.to_string())
            .unwrap()
            .channel
            .unwrap(),
        get_denom(&juno, token_subdenom.as_str())
    );
    let hash = rt
        .block_on(osmosis.query_client::<Ibc>().denom_hash(trace))
        .unwrap();
    let denom = format!("ibc/{}", hash);

    // Get balance on the remote chain
    let balance = rt
        .block_on(
            osmosis
                .query_client::<Bank>()
                .balance(osmosis.sender().to_string(), denom.clone()),
        )
        .unwrap();

    assert_eq!(balance.amount, test_amount.to_string());

    // Send all back
    transfer_tokens(
        &rt,
        &osmosis,
        sender.as_str(),
        &coin(test_amount, denom.clone()),
        &interchain_channel,
        None,
    )
    .unwrap();

    let balance = rt
        .block_on(
            osmosis
                .query_client::<Bank>()
                .balance(osmosis.sender().to_string(), denom.clone()),
        )
        .unwrap();

    assert_eq!(balance.amount, "0");
}
