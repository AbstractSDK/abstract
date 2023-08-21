// This script is used for testing a connection between 2 chains
// This script sets up tokens and channels between transfer ports to transfer those tokens
// This also mints tokens to the chain sender for future interactions

use std::time::{SystemTime, UNIX_EPOCH};

use abstract_core::{
    ans_host::ExecuteMsgFns,
    ibc_client::QueryMsgFns,
    objects::{
        account::AccountTrace, chain_name::ChainName, gov_type::GovernanceDetails, AccountId,
        UncheckedChannelEntry,
    },
    IBC_CLIENT, ICS20, PROXY,
};
use abstract_interface::{
    Abstract, AccountDetails, IbcClient, ManagerExecFns, ManagerQueryFns, ProxyExecFns,
    ProxyQueryFns,
};
use abstract_interface_integration_tests::{
    ibc::{create_test_remote_account, set_env},
    tokenfactory::{create_denom, create_transfer_channel, get_denom, mint},
    JUNO, STARGAZE,
};
use anyhow::Result as AnyResult;
use cosmwasm_std::{coins, to_binary};
use cw_orch::{
    deploy::Deploy,
    prelude::{queriers::Bank, *},
    starship::Starship,
};

pub fn test_send_funds() -> AnyResult<()> {
    env_logger::init();

    set_env();

    let rt: tokio::runtime::Runtime = tokio::runtime::Runtime::new().unwrap();

    let starship = Starship::new(rt.handle().to_owned(), None).unwrap();
    let interchain: InterchainEnv = starship.interchain_env();

    let juno = interchain.daemon(JUNO).unwrap();
    let osmosis = interchain.daemon(STARGAZE).unwrap();

    let osmo_abstr: Abstract<Daemon> = Abstract::load_from(osmosis.clone())?;
    let juno_abstr: Abstract<Daemon> = Abstract::load_from(juno.clone())?;

    let sender = juno.sender().to_string();

    let test_amount: u128 = 100_000_000_000;
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
        .block_on(create_transfer_channel(JUNO, STARGAZE, &starship))
        .unwrap();

    // Register this channel with the abstract ibc implementation for sending tokens
    osmo_abstr.ans_host.update_channels(
        vec![(
            UncheckedChannelEntry {
                connected_chain: "juno".to_string(),
                protocol: ICS20.to_string(),
            },
            interchain_channel
                .get_chain(STARGAZE.to_string())?
                .channel
                .unwrap()
                .to_string(),
        )],
        vec![],
    )?;

    // Create a test account + Remote account

    let account_id = create_test_remote_account(&rt, &osmosis, "osmosis", "juno", &interchain)?;
    // let account_config = osmo_abstr.account.manager.config()?;
    // let account_id = AccountId::new(
    //     account_config.account_id.seq(),
    //     AccountTrace::Remote(vec![ChainName::from("osmosis")]),
    // )?;

    // Get the ibc client address
    let client = osmo_abstr.account.proxy.config()?;

    log::info!("client adddress {:?}", client);

    // Send funds to the remote account
    let send_funds_tx = osmo_abstr.account.manager.execute_on_module(
        PROXY,
        abstract_core::proxy::ExecuteMsg::IbcAction {
            msgs: vec![abstract_core::ibc_client::ExecuteMsg::SendFunds {
                host_chain: ChainName::from("juno"),
                funds: coins(test_amount, get_denom(&osmosis, token_subdenom.as_str())),
            }],
        },
    )?;

    rt.block_on(interchain.await_ibc_execution(STARGAZE.to_owned(), send_funds_tx.txhash))?;

    // Verify the funds have been received
    let distant_account_config = juno_abstr.version_control.get_account(account_id.clone())?;

    let balance = rt.block_on(
        juno.query_client::<Bank>()
            .coin_balances(distant_account_config.proxy),
    )?;

    log::info!("juno balance, {:?}", balance);

    Ok(())
}

pub fn main() {
    test_send_funds().unwrap();
}
