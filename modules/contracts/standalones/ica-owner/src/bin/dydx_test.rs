use std::future::IntoFuture;
use abstract_client::{AbstractClient, Application, Publisher};
use abstract_standalone::{objects::namespace::Namespace, std::standalone};
use cosmwasm_std::{AnyMsg, coin, coins, StakingMsg};
use cw_ica_controller::types::msg::options::ChannelOpenInitOptions;
use ica_owner::{
    ica_controller::ICAController,
    msg::{MyStandaloneExecuteMsgFns, MyStandaloneInstantiateMsg, MyStandaloneQueryMsgFns},
    MyStandaloneInterface, MY_NAMESPACE,
};
use std::time::Duration;
use anybuf::Anybuf;
use cosmos_sdk_proto::cosmos::base::query::v1beta1::PageRequest;
// Use prelude to get all the necessary imports
use cw_orch::{anyhow, daemon::RUNTIME, prelude::*};
use cw_orch::daemon::networks::{CONSTANTINE_3, OSMO_5, parse_network};
use cw_orch::daemon::queriers::Ibc;
use cw_orch::daemon::TxSender;
use cw_orch::environment::{ChainKind, NetworkInfo};
use cw_orch::mock::cw_multi_test::IntoAddr;
use cw_orch_interchain::prelude::*;

#[allow(unused)]
struct TestEnv<Env: CwEnv> {
    abs_src: AbstractClient<Env>,
    // abs_dst: AbstractClient<Env>,
    standalone: Application<Env, MyStandaloneInterface<Env>>,
    ica_controller: ICAController<Env>,
}

const TEST_ACCOUNT_NAMESPACE: &'static str = "dydxtest2";

impl<Env: CwEnv> TestEnv<Env> {
    fn load(src_env: Env, dst_env: Env) -> anyhow::Result<TestEnv<Env>> {
        let ica_controller = ICAController::new(src_env.clone());
        let ica_controller_code_id = ica_controller.code_id()?;

        let abs_src = AbstractClient::new(src_env)?;

        let dydx_namespace = Namespace::new(TEST_ACCOUNT_NAMESPACE)?;
        let test_account = abs_src.fetch_or_build_account(dydx_namespace.clone(), |builder| builder.namespace(dydx_namespace))?;

        let standalone = test_account.install_standalone::<MyStandaloneInterface<_>>(
            &MyStandaloneInstantiateMsg {
                base: standalone::StandaloneInstantiateMsg {},
                ica_controller_code_id,
            },
            &[],
        )?;

        Ok(TestEnv {
            abs_src,
            standalone,
            ica_controller,
        })
    }

    /// Set up the test environment with an Account that has the Standalone installed
    fn setup(src_env: Env, dst_env: Env) -> anyhow::Result<TestEnv<Env>> {
        let mut ica_controller = ICAController::new(src_env.clone());
        // ica_controller.set_code_id(11825);
        if ica_controller.upload_if_needed()?.is_none() {
            // If it's uploaded already just load
            // return Self::load(src_env, dst_env);
        };
        let ica_controller_code_id = ica_controller.code_id()?;

        let namespace = Namespace::new(MY_NAMESPACE)?;

        let abs_src = AbstractClient::new(src_env)?;
        // let abs_src = AbstractClient::builder(src_env).build()?;

        println!("Namespace: {:?}", namespace);

        // Publish the standalone
        let publisher = abs_src.fetch_or_build_account(namespace.clone(), |builder| builder.namespace(namespace))?;
        let publisher = Publisher::new(&publisher)?;

        println!("Publisher: {:?}", publisher.account().id()?);

        publisher.publish_standalone::<MyStandaloneInterface<_>>()?;

        println!("Standalone published");

        let dydx_namespace = Namespace::new("dydxtest2")?;
        let test_account = abs_src.fetch_or_build_account(dydx_namespace.clone(), |builder| builder.namespace(dydx_namespace))?;

        println!("Test account: {:?}", test_account.id()?);

        let standalone = test_account.install_standalone::<MyStandaloneInterface<_>>(
            &MyStandaloneInstantiateMsg {
                base: standalone::StandaloneInstantiateMsg {},
                ica_controller_code_id,
            },
            &[],
        )?;

        Ok(TestEnv {
            abs_src,
            // abs_dst,
            standalone,
            ica_controller,
        })
    }
}


const DYDX_TESTNET_4: ChainInfo = ChainInfo {
    kind: ChainKind::Testnet,
    chain_id: "dydx-testnet-4",
    gas_denom: "udydx",
    gas_price: 0.025,
    grpc_urls: &["http://dydx-testnet-grpc.polkachu.com:23890"],
    network_info: NetworkInfo {
        chain_name: "dydx",
        pub_address_prefix: "dydx",
        coin_type: 118u32,
    },
    lcd_url: None,
    fcd_url: None,
};

pub async fn wait_for_channel(src_ibc: &Ibc, target_port_id: String) -> anyhow::Result<()> {
    let target_state = 3; // STATE_OPEN

    println!(
        "Waiting for channel with target port ID: {} and state 3",
        target_port_id
    );

    loop {
        let channels = src_ibc
            ._channels(Some(PageRequest {
                key: vec![],
                limit: 300,
                count_total: true,
                offset: 0,
                reverse: true,
            }))
            .await?;

        // println!("Channels: {:?}", channels);

        for channel in channels {
            if channel.port_id == target_port_id {
                println!(
                    "Found channel with target port ID and state: {:?}",
                    channel.state
                );
                if channel.state == target_state {
                    println!(
                        "Found channel with target port ID and state 3: {:?}",
                        channel
                    );
                    return Ok(());
                }
            }
        }

        println!("Channel not found, retrying in 10 seconds...");
        cw_orch::tokio::time::sleep(Duration::from_secs(10)).await;
    }
}

fn test_bank_send() -> anyhow::Result<()> {
    let _ = env_logger::builder().is_test(true).try_init();

    let src_chain = ChainInfo {
        gas_price: 1500000000000.0,
        ..CONSTANTINE_3
    };
    let dst_chain = DYDX_TESTNET_4;
    let daemon_interchain = DaemonInterchain::new(vec![src_chain.clone(), dst_chain.clone()], &ChannelCreationValidator)?;

    let src: Daemon = daemon_interchain.clone().get_chain(src_chain.chain_id)?;
    let dst = daemon_interchain.clone().get_chain(dst_chain.chain_id)?;

    let src_ibc: Ibc = src.querier();

    let test_env = TestEnv::setup(src.clone(), dst.clone())?;

    let ica_count = test_env.standalone.ica_count()?.count;

    if ica_count == 0 {
        let creation = daemon_interchain.await_and_check_packets(
            src_chain.chain_id,
            test_env.standalone.create_ica_contract(
                ChannelOpenInitOptions {
                    // Constantine-3
                    connection_id: "connection-165".to_string(),
                    counterparty_connection_id: "connection-45".to_string(),
                    counterparty_port_id: None,
                    channel_ordering: Some(cosmwasm_std::IbcOrder::Ordered),
                },
                None,
            )?,
        )?;

        println!("ICA creation: {:?}", creation);

        let port_id = creation.event_attr_value("channel_open_init", "port_id")?;

        println!("Waiting for channel with port {} to open", port_id);

        // RUNTIME.block_on(wait_for_channel(&src_ibc, port_id))?;
        std::thread::sleep(Duration::from_secs(30));
    }

    // First ica account id is 0
    let mut current_ica_account_id = ica_count - 1;

    // // Waiting for channel to open, cw-orch not capable of waiting channel open

    let state = test_env
        .standalone
        .ica_contract_state(current_ica_account_id)?;
    println!("State: {:?}", state);
    let ica_addr = state.ica_state.unwrap().ica_addr;

    println!("ICA Address: {:?}", ica_addr);

    // https://www.mintscan.io/dydx-testnet/address/dydx1q002zawgk98jwr2up08s0qcg9u06ede6vltd45svqpertqlfn57q2s9v9e
    //
    // // Send 10_000 uosmo from ICA to some address
    // let receiving_addr = test_env.abs_dst.registry().addr_str()?;
    // let amount = coins(10_000, "uosmo");
    // RUNTIME.block_on(osmosis.wallet().bank_send(&ica_addr, amount.clone()))?;

    println!("Sending 5 adv4tnt from {} to {}", dst.sender().address(), ica_addr);
    // TODO: for some reason this don't work because the sender seems to take the wrong bech32 prefix (codespace sdk code 35: internal logic error: hrp does not match bech32 prefix: expected 'dydx' got 'cosmwasm')
    // RUNTIME.block_on(dst.sender().bank_send(&ica_addr.into_addr(), coins(5, "adv4tnt")))?;

    // TODO: bank send from wallet to ICA if not already there to test delegate

    let staking_test = dbg!(daemon_interchain.await_and_check_packets(
        src_chain.chain_id,
        test_env.standalone.ica_execute(
            current_ica_account_id,
            vec![cosmwasm_std::CosmosMsg::Staking(StakingMsg::Delegate {
                amount: coin(5, "adv4tnt"),
                validator: "dydxvaloper1ldal0sqjf80lcepacdmgtgycunaxn9axt6l87w".to_string(),
            })],
        )?,
    )?);

    println!("Test: {:?}", staking_test);


    // nEeed to deposit to sub-account, though don't have testnet USDC
    // MsgDepositToSubaccount


    /*
    {
  "@type": "/dydxprotocol.vault.MsgDepositToMegavault",
  "subaccount_id": {
    "owner": "dydx1cz7y59k4jfdudjnpm7jynp3dkxqtejkfzmpc4f",
    "number": 0
  },
  "quote_quantums": "5000000"
}

export function calculateVaultQuantums(size: number): bigint {
  return BigInt(BigNumber(size).times(1_000_000).toFixed(0, BigNumber.ROUND_FLOOR));
}
bigIntToBytes(calculateVaultQuantums(amountUsdc)),
     */
    let test = daemon_interchain.await_and_check_packets(
        src_chain.chain_id,
        test_env.standalone.ica_execute(
            current_ica_account_id,
            vec![cosmwasm_std::CosmosMsg::Stargate {
                type_url: "/dydxprotocol.vault.MsgDepositToMegavault".to_string(),
                value: cosmwasm_std::Binary::new(Anybuf::new()
                    .append_bytes(1, Anybuf::new()
                        .append_string(1, ica_addr) // owner
                        .append_uint32(2, 0) // number
                        .into_vec()
                    )
                    // https://github.com/dydxprotocol/v4-clients/blob/aa0d95d298b2b202c4fe8a8160dd8f1b63b8f009/dydxjs/packages/dydxjs/src/dydxprotocol/vault/tx.ts#L180C39-L180C52
                    .append_bytes(2, (0u128 * 1_000_000).to_be_bytes()).into_vec()) //
            }],
        )?,
    )?;



    // let dst_account_balance = osmosis.balance(receiving_addr.clone(), None)?;
    // assert_eq!(dst_account_balance, amount);
    //
    // // It's possible to use control multiple ica with same ica controller
    //
    // let _ = daemon_interchain.check_ibc(
    //     src_chain.chain_id,
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
    //     src_chain.chain_id,
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


fn main() {
    dotenv::dotenv().ok();
    env_logger::init();
    dbg!(test_bank_send()).unwrap();
}
