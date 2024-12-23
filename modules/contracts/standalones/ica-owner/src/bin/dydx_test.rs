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
use dydx_proto::cosmos_sdk_proto::traits::{MessageExt, Message, Name};

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
    gas_denom: "adv4tnt",
    gas_price: 0.025,
    grpc_urls: &["https://test-dydx-grpc.kingnodes.com:443", "http://dydx-testnet-grpc.polkachu.com:23890"],
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

    if ica_count == 1 {
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

    println!("Sending 5 {} from {} to {}", dst_chain.gas_denom, dst.sender().address(), ica_addr);
    // TODO: for some reason this don't work because the sender seems to take the wrong bech32 prefix (codespace sdk code 35: internal logic error: hrp does not match bech32 prefix: expected 'dydx' got 'cosmwasm')
    // RUNTIME.block_on(dst.sender().bank_send(&ica_addr.into_addr(), coins(5, dst_chain.gas_denom)))?;

    // TODO: bank send from wallet to ICA if not already there to test delegate

    let staking_test = dbg!(daemon_interchain.await_and_check_packets(
        src_chain.chain_id,
        test_env.standalone.ica_execute(
            current_ica_account_id,
            vec![cosmwasm_std::CosmosMsg::Staking(StakingMsg::Delegate {
                amount: coin(10, dst_chain.gas_denom),
                // Arbitrary validator
                validator: "dydxvaloper1vhj8v39z46e0euew3ntqzftx48lca6y7kfl80g".to_string(),
            })],
        )?,
    )?);

    // Need to deposit to sub-account, though don't have testnet USDC
    // MsgDepositToSubaccount

    // We withdraw from subaccount (frontend) first
    // https://www.mintscan.io/dydx-testnet/tx/215EAE37ED4D73881A1C6054575A8FFF5686549A0315D4C7E662CA544C719B1A?height=27830625


    let dst_bank  = dst.bank_querier();
    let ica_balance = RUNTIME.block_on(dst_bank._spendable_balances(&Addr::unchecked(ica_addr.clone())))?;
    println!("ICA balance: {:?}", ica_balance);
    let sub_account_test = daemon_interchain.await_and_check_packets(
        src_chain.chain_id,
        test_env.standalone.ica_execute(
            current_ica_account_id,
            #[allow(deprecated)]
            vec![
                cosmwasm_std::CosmosMsg::Stargate {
                    type_url: dydx_proto::dydxprotocol::sending::MsgCreateTransfer::type_url(),
                    value: dydx_proto::dydxprotocol::sending::MsgCreateTransfer {
                        transfer: Some(dydx_proto::dydxprotocol::sending::Transfer {
                            sender: None,
                            recipient: Some(dydx_proto::dydxprotocol::subaccounts::SubaccountId {
                                owner: ica_addr.to_string(),
                                number: 0
                            }),
                            asset_id: 0,
                            amount: 69,
                        })
                }.encode_to_vec().into()
            },
                cosmwasm_std::CosmosMsg::Stargate {
                type_url: dydx_proto::dydxprotocol::sending::MsgDepositToSubaccount::type_url(),
                value: dydx_proto::dydxprotocol::sending::MsgDepositToSubaccount {
                    sender: ica_addr.to_string(),
                    recipient: Some(dydx_proto::dydxprotocol::subaccounts::SubaccountId {
                        owner: ica_addr.to_string(),
                        number: 0
                    }),
                    asset_id: 0,
                    quantums: 69,
                }.encode_to_vec().into()
            }
            ],
        )?,
    )?;


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
    let megavault_test = daemon_interchain.await_and_check_packets(
        src_chain.chain_id,
        test_env.standalone.ica_execute(
            current_ica_account_id,
            #[allow(deprecated)]
            vec![cosmwasm_std::CosmosMsg::Stargate {
                type_url: dydx_proto::dydxprotocol::vault::MsgDepositToMegavault::type_url(),
                value: dydx_proto::dydxprotocol::vault::MsgDepositToMegavault {
                    subaccount_id: Some(dydx_proto::dydxprotocol::subaccounts::SubaccountId {
                        owner: ica_addr.to_string(),
                        number: 0
                    }),
                    quote_quantums: 42_i32.to_be_bytes().to_vec(),
                }.encode_to_vec().into()
            }],
        )?,
    )?;

    // let dst_account_balance = osmosis.balance(receiving_addr.clone(), None)?;
    // assert_eq!(dst_account_balance, amount);

    Ok(())
}


fn main() {
    dotenv::dotenv().ok();
    env_logger::init();
    dbg!(test_bank_send()).unwrap();
}
