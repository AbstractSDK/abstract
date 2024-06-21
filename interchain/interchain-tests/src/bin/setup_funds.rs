// This script is used for testing a connection between 2 chains
// This script sets up tokens and channels between transfer ports to transfer those tokens
// This also mints tokens to the chain sender for future interactions

use std::time::{SystemTime, UNIX_EPOCH};

use abstract_interchain_tests::{
    interchain_accounts::{create_test_remote_account, set_env},
    JUNO, STARGAZE,
};
use abstract_interface::{Abstract, AbstractAccount, ProxyQueryFns};
use abstract_std::{ans_host::ExecuteMsgFns, objects::UncheckedChannelEntry, ICS20, PROXY};
use anyhow::Result as AnyResult;
use cosmwasm_std::coins;
use cw_orch::prelude::*;
use cw_orch_interchain::prelude::*;
use cw_orch_proto::tokenfactory::{create_denom, create_transfer_channel, get_denom, mint};

pub fn test_send_funds() -> AnyResult<()> {
    env_logger::init();

    set_env();

    let rt: tokio::runtime::Runtime = tokio::runtime::Runtime::new().unwrap();

    let starship = Starship::new(rt.handle(), None).unwrap();
    let interchain = starship.interchain_env();

    let juno = interchain.chain(JUNO).unwrap();
    let stargaze = interchain.chain(STARGAZE).unwrap();

    let abstr_stargaze: Abstract<Daemon> = Abstract::load_from(stargaze.clone())?;
    let abstr_juno: Abstract<Daemon> = Abstract::load_from(juno.clone())?;

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
    create_denom(&juno, token_subdenom.as_str())?;

    // Mint Denom
    mint(&juno, sender.as_str(), token_subdenom.as_str(), test_amount)?;

    // Create a channel between the 2 chains for the transfer ports
    let interchain_channel = create_transfer_channel(JUNO, STARGAZE, &interchain)?;

    // Register this channel with the abstract ibc implementation for sending tokens
    abstr_stargaze.ans_host.update_channels(
        vec![(
            UncheckedChannelEntry {
                connected_chain: "juno".to_string(),
                protocol: ICS20.to_string(),
            },
            interchain_channel
                .get_chain(STARGAZE)?
                .channel
                .unwrap()
                .to_string(),
        )],
        vec![],
    )?;

    // Create a test account + Remote account

    let (origin_account, remote_account_id) =
        create_test_remote_account(&abstr_stargaze, STARGAZE, JUNO, &interchain, None)?;
    // let account_config = osmo_abstr.account.manager.config()?;
    // let account_id = AccountId::new(
    //     account_config.account_id.seq(),
    //     AccountTrace::Remote(vec![ChainName::from("osmosis")]),
    // )?;

    // Get the ibc client address
    let remote_account = AbstractAccount::new(&abstr_stargaze, remote_account_id.clone());
    let client = remote_account.proxy.config()?;

    log::info!("client adddress {:?}", client);

    // Send funds to the remote account
    let send_funds_tx = origin_account.manager.execute_on_module(
        PROXY,
        abstract_std::proxy::ExecuteMsg::IbcAction {
            msg: abstract_std::ibc_client::ExecuteMsg::SendFunds {
                host_chain: "juno".parse().unwrap(),
                funds: coins(test_amount, get_denom(&stargaze, token_subdenom.as_str())),
            },
        },
    )?;

    interchain
        .check_ibc(STARGAZE, send_funds_tx)?
        .into_result()?;

    // Verify the funds have been received
    let remote_account_config = abstr_juno
        .version_control
        .get_account(remote_account_id.clone())?;

    let balance = juno
        .bank_querier()
        .balance(remote_account_config.proxy, None)?;

    log::info!("juno balance, {:?}", balance);

    Ok(())
}

pub fn main() {
    test_send_funds().unwrap();
}
