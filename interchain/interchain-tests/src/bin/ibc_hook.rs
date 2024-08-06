// This script is used for testing a connection between 2 chains
// This script sets up tokens and channels between transfer ports to transfer those tokens
// This also mints tokens to the chain sender for future interactions

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use abstract_interchain_tests::{
    interchain_accounts::create_test_remote_account, setup::set_starship_env, JUNO,
};
use abstract_interface::{
    connection::connect_one_way_to, Abstract, AbstractAccount, ProxyQueryFns,
};
use abstract_sdk::{IbcHooksBuilder, IbcMemoBuilder, PacketForwardMiddlewareBuilder};
use abstract_std::{
    ans_host::ExecuteMsgFns,
    objects::{TruncatedChainId, UncheckedChannelEntry},
    ICS20, PROXY,
};
use anyhow::Result as AnyResult;
use cosmwasm_std::{coin, coins};
use counter_contract::CounterQueryMsgFns;
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
    set_starship_env();
    env_logger::init();

    let starship = Starship::new(None).unwrap();
    let interchain = starship.interchain_env();

    let juno = interchain.get_chain(JUNO).unwrap();
    let juno2 = interchain.get_chain(JUNO2).unwrap();
    let juno3 = interchain.get_chain(JUNO3).unwrap();
    let juno4 = interchain.get_chain(JUNO4).unwrap();

    // Create a channel between the 4 chains for the transfer ports
    // JUNO>JUNO2>JUNO3>JUNO4
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

    // let abstr_juno = Abstract::deploy_on(juno.clone(), juno.sender_addr().to_string())?;
    // let abstr_juno2 = Abstract::deploy_on(juno2.clone(), juno2.sender_addr().to_string())?;
    // connect_one_way_to(&abstr_juno, &abstr_juno2, &interchain)?;

    let abstr_juno = Abstract::load_from(juno.clone())?;
    let abstr_juno2 = Abstract::load_from(juno2.clone())?;

    let counter_juno2 = init_counter(juno2.clone())?;
    let counter_juno3 = init_counter(juno3.clone())?;
    let counter_juno4 = init_counter(juno4.clone())?;

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
        .receiver(counter_juno3.address()?) // "receiver" on juno3
        .next(IbcHooksBuilder::new(
            counter_juno2.address()?,
            &counter_contract::msg::ExecuteMsg::Increment {},
        ))
        .build()?;
    // We send from osmosis to juno funds with pfm memo that includes juno-stargaze channel
    origin_account.manager.execute_on_module(
        PROXY,
        abstract_std::proxy::ExecuteMsg::IbcAction {
            msg: abstract_std::ibc_client::ExecuteMsg::SendFunds {
                host_chain: TruncatedChainId::from_chain_id(JUNO2),
                funds: coins(100_000_000_000, get_denom(&juno, token_subdenom.as_str())),
                memo: Some(memo),
                receiver: None,
            },
        },
    )?;
    log::info!("waiting for ibc_hook to finish tx");
    std::thread::sleep(Duration::from_secs(15));

    // Verify the funds have been received
    let balance = juno4.bank_querier().balance(juno4.sender_addr(), None)?;

    log::info!("juno4 balance, {:?}", balance);

    let count_juno2 = counter_juno2.get_count()?;
    log::info!("count juno2: {count_juno2:?}");
    let count_juno3 = counter_juno3.get_count()?;
    log::info!("count juno3: {count_juno3:?}");
    let count_juno4 = counter_juno4.get_count()?;
    log::info!("count juno4: {count_juno4:?}");

    Ok(())
}

pub fn init_counter<Chain: CwEnv>(
    chain: Chain,
) -> AnyResult<counter_contract::CounterContract<Chain>> {
    let counter = counter_contract::CounterContract::new(chain);
    // counter.upload()?;
    // counter.instantiate(
    //     &counter_contract::msg::InstantiateMsg { count: 0 },
    //     None,
    //     None,
    // )?;
    Ok(counter)
}

pub fn main() {
    test_pfm().unwrap();
}
