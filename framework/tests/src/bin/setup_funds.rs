// This script is used for testing a connection between 2 chains
// This script sets up tokens and channels between transfer ports to transfer those tokens
// This also mints tokens to the chain sender for future interactions

use std::time::{SystemTime, UNIX_EPOCH};

use abstract_core::{ans_host::ExecuteMsgFns, objects::UncheckedChannelEntry, ICS20, PROXY};
use abstract_interface::{Abstract, ProxyQueryFns};
use abstract_interface_integration_tests::{
    interchain_accounts::{create_test_remote_account, set_env},
    JUNO, STARGAZE,
};

use anyhow::Result as AnyResult;
use cosmwasm_std::coins;
use cw_orch::{
    deploy::Deploy,
    prelude::{queriers::Bank, *},
};
use cw_orch_interchain::channel_creator::ChannelCreator;
use cw_orch_interchain_core::env::InterchainEnv;
use cw_orch_proto::tokenfactory::{create_denom, create_transfer_channel, get_denom, mint};
use cw_orch_starship::Starship;

pub fn test_send_funds() -> AnyResult<()> {
    env_logger::init();

    set_env();

    let rt: tokio::runtime::Runtime = tokio::runtime::Runtime::new().unwrap();

    let starship = Starship::new(rt.handle().to_owned(), None).unwrap();
    let interchain = starship.interchain_env();

    let juno = interchain.chain(JUNO).unwrap();
    let osmosis = interchain.chain(STARGAZE).unwrap();

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
    create_denom(&juno, token_subdenom.as_str())?;

    // Mint Denom
    mint(&juno, sender.as_str(), token_subdenom.as_str(), test_amount)?;

    // Create a channel between the 2 chains for the transfer ports
    let interchain_channel =
        rt.block_on(create_transfer_channel(JUNO, STARGAZE, None, &interchain))?;

    // Register this channel with the abstract ibc implementation for sending tokens
    osmo_abstr.ans_host.update_channels(
        vec![(
            UncheckedChannelEntry {
                connected_chain: "juno".to_string(),
                protocol: ICS20.to_string(),
            },
            interchain_channel
                .get_chain(&STARGAZE.to_string())?
                .channel
                .unwrap()
                .to_string(),
        )],
        vec![],
    )?;

    // Create a test account + Remote account

    let account_id = create_test_remote_account(&rt, &osmo_abstr, STARGAZE, JUNO, &interchain)?;
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
                host_chain: "juno".into(),
                funds: coins(test_amount, get_denom(&osmosis, token_subdenom.as_str())),
            }],
        },
    )?;

    rt.block_on(interchain.wait_ibc(&STARGAZE.to_string(), send_funds_tx))?;

    // Verify the funds have been received
    let remote_account_config = juno_abstr.version_control.get_account(account_id.clone())?;

    let balance = rt.block_on(
        juno.query_client::<Bank>()
            .balance(remote_account_config.proxy, None),
    )?;

    log::info!("juno balance, {:?}", balance);

    Ok(())
}

pub fn main() {
    test_send_funds().unwrap();
}
