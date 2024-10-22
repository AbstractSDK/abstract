// This script is used for testing a connection between 4 chains
// This script checks ibc-hook memo implementation on ibc-client

use std::{
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use abstract_interchain_tests::{abstract_starship_interfaces, set_starship_env, JUNO, JUNO2};
use abstract_interface::{connection::connect_one_way_to, Abstract, AccountI};
use abstract_sdk::HookMemoBuilder;
use abstract_std::{
    ans_host::ExecuteMsgFns,
    ibc_client::QueryMsgFns,
    objects::{TruncatedChainId, UncheckedChannelEntry},
    IBC_CLIENT, ICS20,
};
use anyhow::Result as AnyResult;
use cosmwasm_std::{coin, coins};
use counter_contract::CounterContract;
use cw_orch::{
    daemon::{senders::CosmosSender, CosmosOptions, TxSender, Wallet, RUNTIME},
    prelude::*,
};
use cw_orch_interchain::prelude::*;
use cw_orch_proto::tokenfactory::{create_denom, get_denom, mint};
use networks::ChainKind;

pub fn test_ibc_hook() -> AnyResult<()> {
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

    // Create a channel between the 2 chains for the transfer ports
    // JUNO>JUNO2
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

    let counter_juno2 = init_counter(juno2.clone())?;

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
            monarch: juno.sender_addr().to_string(),
        },
    )?;
    origin_account.set_ibc_status(true)?;

    // Send funds to the remote account
    RUNTIME.block_on(juno.sender().bank_send(
        &origin_account.address()?,
        vec![coin(test_amount, get_denom(&juno, token_subdenom.as_str()))],
    ))?;

    let memo = HookMemoBuilder::new(
        counter_juno2.address()?,
        &counter_contract::msg::ExecuteMsg::Increment {},
    )
    .build()?;
    // We send from osmosis to juno funds with pfm memo that includes juno-stargaze channel
    origin_account.execute_on_module(
        IBC_CLIENT,
        &abstract_std::ibc_client::ExecuteMsg::SendFunds {
            host_chain: TruncatedChainId::from_chain_id(JUNO2),
            memo: Some(memo.clone()),
            receiver: Some(counter_juno2.addr_str()?),
        },
        coins(10_000_000_000, get_denom(&juno, token_subdenom.as_str())),
    )?;

    origin_account.execute_on_module(
        IBC_CLIENT,
        &abstract_std::ibc_client::ExecuteMsg::SendFunds {
            host_chain: TruncatedChainId::from_chain_id(JUNO2),
            memo: Some(memo.clone()),
            receiver: Some(counter_juno2.addr_str()?),
        },
        coins(10_000_000_000, get_denom(&juno, token_subdenom.as_str())),
    )?;

    origin_account.execute_on_module(
        IBC_CLIENT,
        &abstract_std::ibc_client::ExecuteMsg::SendFunds {
            host_chain: TruncatedChainId::from_chain_id(JUNO2),
            memo: Some(memo.clone()),
            receiver: Some(counter_juno2.addr_str()?),
        },
        coins(10_000_000_000, get_denom(&juno, token_subdenom.as_str())),
    )?;

    log::info!("waiting for ibc_hook to finish tx");
    std::thread::sleep(Duration::from_secs(15));

    let counter_juno2 = CounterContract::new(juno2.clone());
    let count_juno2: counter_contract::msg::GetCountResponse =
        counter_juno2.query(&counter_contract::msg::QueryMsg::GetCount {})?;
    log::info!("count juno2: {count_juno2:?}");

    // Verify the funds have been received
    let count_juno2_balance = juno2
        .bank_querier()
        .balance(&counter_juno2.address()?, None)?;

    log::info!("count_juno2 balance, {:?}", count_juno2_balance);
    Ok(())
}

pub fn init_counter<Chain: CwEnv>(chain: Chain) -> anyhow::Result<CounterContract<Chain>> {
    let counter = CounterContract::new(chain);
    counter.upload()?;
    counter.instantiate(
        &counter_contract::msg::InstantiateMsg { count: 0 },
        None,
        &[],
    )?;
    Ok(counter)
}

pub fn main() {
    test_ibc_hook().unwrap();
}
