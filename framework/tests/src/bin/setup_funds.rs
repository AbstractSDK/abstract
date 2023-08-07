// This script is used for testing a connection between 2 chains
// This script sets up tokens and channels between transfer ports to transfer those tokens
// This also mints tokens to the chain sender for future interactions

use abstract_interface_integration_tests::{
    tokenfactory::{create_denom, create_transfer_channel, mint},
    JUNO, OSMOSIS,
};
use cw_orch::{prelude::*, starship::Starship};

pub fn main() {
    env_logger::init();
    let rt: tokio::runtime::Runtime = tokio::runtime::Runtime::new().unwrap();

    let starship = Starship::new(rt.handle().to_owned(), None).unwrap();
    let interchain: InterchainEnv = starship.interchain_env();

    let juno = interchain.daemon(JUNO).unwrap();
    let osmosis = interchain.daemon(OSMOSIS).unwrap();

    let sender = juno.sender().to_string();

    let test_amount: u128 = 100_000_000_000;
    let token_subdenom = "abstracttesttoken";

    // Create Denom
    rt.block_on(create_denom(&juno, token_subdenom)).unwrap();

    // Mint Denom
    rt.block_on(mint(&juno, sender.as_str(), token_subdenom, test_amount))
        .unwrap();

    // Create a channel between the 2 chains for the transfer ports
    let interchain_channel = rt
        .block_on(create_transfer_channel(JUNO, OSMOSIS, &starship))
        .unwrap();

    // Register this channel with the ibc implementation for sending tokens ???
}
