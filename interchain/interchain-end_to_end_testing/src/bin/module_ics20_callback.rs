// This script is used for testing a connection between 4 chains
// This script checks ibc-hook memo implementation on ibc-client

use std::{
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use abstract_interchain_tests::{abstract_starship_interfaces, set_starship_env, JUNO, JUNO2};
use abstract_interface::{Abstract, AccountDetails, AccountI, AppDeployer};
use abstract_sdk::HookMemoBuilder;
use abstract_std::{
    ans_host::ExecuteMsgFns,
    ibc::Callback,
    objects::{TruncatedChainId, UncheckedChannelEntry, ABSTRACT_ACCOUNT_ID},
    IBC_CLIENT, ICS20,
};
use anyhow::Result as AnyResult;
use cosmwasm_std::{coin, coins};
use counter_contract::CounterContract;
use cw_orch::{
    daemon::{senders::CosmosSender, CosmosOptions, RUNTIME},
    prelude::*,
};
use cw_orch_interchain::prelude::*;
use cw_orch_proto::tokenfactory::{create_denom, get_denom, mint};
use networks::ChainKind;
use ping_pong::{AppExecuteMsgFns, AppQueryMsgFns};

pub fn test_ics20_callback() -> AnyResult<()> {
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
    let root_account_juno = AccountI::load_from(&abstr_juno, ABSTRACT_ACCOUNT_ID)?;
    root_account_juno.set_ibc_status(true)?;
    root_account_juno.create_remote_account(
        AccountDetails::default(),
        TruncatedChainId::from_chain_id(JUNO2),
    )?;
    // let root_account_juno2 = AccountI::load_from(&abstr_juno2, ABSTRACT_ACCOUNT_ID)?;

    let ping_pong_juno = init_ping_pong(&root_account_juno)?;
    // let ping_pong_juno2 = init_ping_pong(&root_account_juno2)?;

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
    mint(
        &juno,
        root_account_juno.addr_str()?.as_str(),
        token_subdenom.as_str(),
        test_amount,
    )?;

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

    let tx_response = ping_pong_juno.fund_opponent(
        Callback { msg: b"foo".into() },
        Coin::new(
            10_000_000_000_u128,
            get_denom(&juno, token_subdenom.as_str()),
        ),
        TruncatedChainId::from_chain_id(JUNO2),
    )?;

    let ibc_result = interchain.await_and_check_packets(&juno.chain_id(), tx_response)?;

    println!(
        "Ibc Result of sending funds + hook : {:?}",
        ibc_result.packets[0]
    );

    let callbacks = ping_pong_juno.ics_20_callbacks()?;
    dbg!(callbacks);

    log::info!("waiting for ibc_hook to finish tx");
    std::thread::sleep(Duration::from_secs(15));

    Ok(())
}

pub fn init_ping_pong<Chain: CwEnv>(
    abstract_account: &AccountI<Chain>,
) -> anyhow::Result<ping_pong::AppInterface<Chain>> {
    let ping_pong = ping_pong::AppInterface::new(
        ping_pong::contract::APP_ID,
        abstract_account.environment().clone(),
    );
    ping_pong.deploy(
        ping_pong::contract::APP_VERSION.parse().unwrap(),
        abstract_interface::DeployStrategy::Try,
    )?;
    abstract_account.environment();
    abstract_account.install_app(&ping_pong, &ping_pong::msg::AppInstantiateMsg {}, &[])?;
    Ok(ping_pong)
}

pub fn main() {
    test_ics20_callback().unwrap();
}
