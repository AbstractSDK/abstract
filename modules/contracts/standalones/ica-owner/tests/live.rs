use abstract_client::{AbstractClient, Application, Publisher};
use abstract_standalone::{objects::namespace::Namespace, std::standalone};
use cosmwasm_std::{coins, PageRequest};
use cw_ica_controller::types::msg::options::ChannelOpenInitOptions;
use cw_orch_starship::Starship;
use my_standalone::{
    ica_controller::ICAController,
    msg::{MyStandaloneExecuteMsgFns, MyStandaloneInstantiateMsg, MyStandaloneQueryMsgFns},
    MyStandaloneInterface, MY_NAMESPACE,
};
use std::time::Duration;
// Use prelude to get all the necessary imports
use cw_orch::{anyhow, daemon::RUNTIME, prelude::*};
use cw_orch::daemon::networks::{OSMO_5, parse_network};
use cw_orch::daemon::queriers::Ibc;
use cw_orch::environment::{ChainKind, NetworkInfo};
use cw_orch_interchain::prelude::*;

#[allow(unused)]
struct TestEnv<Env: CwEnv> {
    abs_src: AbstractClient<Env>,
    abs_dst: AbstractClient<Env>,
    standalone: Application<Env, MyStandaloneInterface<Env>>,
    ica_controller: ICAController<Env>,
}

impl<Env: CwEnv> TestEnv<Env> {
    fn load(src_env: Env, dst_env: Env) -> anyhow::Result<TestEnv<Env>> {
        let ica_controller = ICAController::new(src_env.clone());
        let ica_controller_code_id = ica_controller.code_id()?;

        let abs_src = AbstractClient::new(src_env)?;
        let abs_dst = AbstractClient::new(dst_env)?;

        let namespace = Namespace::new(MY_NAMESPACE)?;
        let publisher = abs_src.account_builder().namespace(namespace).build()?;
        let publisher = Publisher::new(&publisher)?;

        let sub_account = abs_src
            .account_builder()
            .sub_account(publisher.account())
            .build()?;
        let standalone = sub_account.install_standalone::<MyStandaloneInterface<_>>(
            &MyStandaloneInstantiateMsg {
                base: standalone::StandaloneInstantiateMsg {},
                ica_controller_code_id,
            },
            &[],
        )?;

        Ok(TestEnv {
            abs_src,
            abs_dst,
            standalone,
            ica_controller,
        })
    }

    /// Set up the test environment with an Account that has the Standalone installed
    fn setup(src_env: Env, dst_env: Env) -> anyhow::Result<TestEnv<Env>> {
        let ica_controller = ICAController::new(src_env.clone());
        if ica_controller.upload_if_needed()?.is_none() {
            // If it's uploaded already just load
            return Self::load(src_env, dst_env);
        };
        let ica_controller_code_id = ica_controller.code_id()?;

        let namespace = Namespace::new(MY_NAMESPACE)?;

        let abs_src = AbstractClient::builder(src_env).build()?;
        let abs_dst = AbstractClient::builder(dst_env).build()?;

        // Publish the standalone
        let publisher = abs_src.account_builder().namespace(namespace).build()?;
        let publisher = Publisher::new(&publisher)?;
        publisher.publish_standalone::<MyStandaloneInterface<_>>()?;

        let sub_account = abs_src
            .account_builder()
            .sub_account(publisher.account())
            .build()?;
        let standalone = sub_account.install_standalone::<MyStandaloneInterface<_>>(
            &MyStandaloneInstantiateMsg {
                base: standalone::StandaloneInstantiateMsg {},
                ica_controller_code_id,
            },
            &[],
        )?;

        Ok(TestEnv {
            abs_src,
            abs_dst,
            standalone,
            ica_controller,
        })
    }
}

pub async fn wait_for_channel(ibc: Ibc) -> anyhow::Result<()> {
    let target_state = 3; // STATE_OPEN

    let channels = ibc
        ._channels(None)
        .await?;

    println!("Channels: {:?}", channels);
    Ok(())

    // loop {
    //     let channels = ibc
    //         ._channels(None)
    //         .await?;
    //
    //     println!("Channels: {:?}", channels);
    //
    //     for channel in channels {
    //         if let Some(ref counterparty) = channel.counterparty {
    //             if counterparty.port_id == target_port_id {
    //                 println!(
    //                     "Found channel with target port ID and state: {:?}",
    //                     channel.state
    //                 );
    //                 if channel.state == target_state {
    //                     println!(
    //                         "Found channel with target port ID and state 3: {:?}",
    //                         channel
    //                     );
    //                     return Ok(());
    //                 }
    //             }
    //         }
    //     }
    //
    //     println!("Channel not found, retrying in 10 seconds...");
    // }
}

#[test]
fn test_bank_send() -> anyhow::Result<()> {
    let _ = env_logger::builder().is_test(true).try_init();

    let daemon_interchain = DaemonInterchain::new(vec![OSMO_5, ChainInfo {
        kind: ChainKind::Testnet,
        chain_id: "dydx-testnet-4",
        gas_denom: "udydx",
        gas_price: 0.025,
        grpc_urls: &["https://dydx-testnet-grpc.polkachu.com:23890"],
        network_info: NetworkInfo {
            chain_name: "dydx",
            pub_address_prefix: "dydx",
            coin_type: 118u32,
        },
        lcd_url: None,
        fcd_url: None,
    }], &ChannelCreationValidator)?;

    let osmo_5: Daemon = daemon_interchain.clone().get_chain("osmo-test-5")?;
    let dydx = daemon_interchain.clone().get_chain("dydx-testnet-4")?;

    let ibc: Ibc = dydx.querier();

    RUNTIME.block_on(wait_for_channel(ibc))?;



    // let ibc_path = RUNTIME.block_on(async {
    //     starship
    //         .client()
    //         .registry()
    //         .await
    //         .ibc_path("juno-1", "osmosis-1")
    //         .await
    // })?;
    //
    // let test_env = TestEnv::setup(juno.clone(), osmosis.clone())?;
    //
    // let _ = daemon_interchain.check_ibc(
    //     "juno-1",
    //     test_env.standalone.create_ica_contract(
    //         ChannelOpenInitOptions {
    //             connection_id: ibc_path.chain_1.connection_id.to_string(),
    //             counterparty_connection_id: ibc_path.chain_2.connection_id.to_string(),
    //             counterparty_port_id: None,
    //             channel_ordering: Some(cosmwasm_std::IbcOrder::Ordered),
    //         },
    //         None,
    //     )?,
    // )?;
    //
    // // First ica account id is 0
    // let mut current_ica_account_id = 0;
    //
    // // Waiting for channel to open, cw-orch not capable of waiting channel open
    // std::thread::sleep(Duration::from_secs(15));
    //
    // let state = test_env
    //     .standalone
    //     .ica_contract_state(current_ica_account_id)?;
    // let ica_addr = state.ica_state.unwrap().ica_addr;
    //
    // // Send 10_000 uosmo from ICA to some address
    // let receiving_addr = test_env.abs_dst.registry().addr_str()?;
    // let amount = coins(10_000, "uosmo");
    // RUNTIME.block_on(osmosis.wallet().bank_send(&ica_addr, amount.clone()))?;
    //
    // let _ = daemon_interchain.check_ibc(
    //     "juno-1",
    //     test_env.standalone.ica_execute(
    //         current_ica_account_id,
    //         cosmwasm_std::CosmosMsg::Bank(cosmwasm_std::BankMsg::Send {
    //             to_address: receiving_addr.to_string(),
    //             amount: amount.clone(),
    //         }),
    //     )?,
    // )?;
    //
    // let dst_account_balance = osmosis.balance(receiving_addr.clone(), None)?;
    // assert_eq!(dst_account_balance, amount);
    //
    // // It's possible to use control multiple ica with same ica controller
    //
    // let _ = daemon_interchain.check_ibc(
    //     "juno-1",
    //     test_env.standalone.create_ica_contract(
    //         ChannelOpenInitOptions {
    //             connection_id: ibc_path.chain_1.connection_id.to_string(),
    //             counterparty_connection_id: ibc_path.chain_2.connection_id.to_string(),
    //             counterparty_port_id: None,
    //             channel_ordering: Some(cosmwasm_std::IbcOrder::Ordered),
    //         },
    //         None,
    //     )?,
    // )?;
    //
    // current_ica_account_id += 1;
    //
    // // Waiting for channel to open, cw-orch not capable of waiting channel open
    // std::thread::sleep(Duration::from_secs(15));
    //
    // let state = test_env
    //     .standalone
    //     .ica_contract_state(current_ica_account_id)?;
    // let ica_addr = state.ica_state.unwrap().ica_addr;
    //
    // // Send 15_000 uosmo from ICA to some address
    // let receiving_addr = test_env.abs_dst.name_service().addr_str()?;
    // let amount = coins(15_000, "uosmo");
    // RUNTIME.block_on(osmosis.wallet().bank_send(&ica_addr, amount.clone()))?;
    //
    // let _ = daemon_interchain.check_ibc(
    //     "juno-1",
    //     test_env.standalone.ica_execute(
    //         current_ica_account_id,
    //         cosmwasm_std::CosmosMsg::Bank(cosmwasm_std::BankMsg::Send {
    //             to_address: receiving_addr.to_string(),
    //             amount: amount.clone(),
    //         }),
    //     )?,
    // )?;
    //
    // let dst_account_balance = osmosis.balance(receiving_addr, None)?;
    // assert_eq!(dst_account_balance, amount);

    Ok(())
}
