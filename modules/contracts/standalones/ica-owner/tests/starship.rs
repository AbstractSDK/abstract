use std::{io::Write, time::Duration};

use cw_ica_controller::types::msg::options::ChannelOpenInitOptions;
use cw_orch_starship::Starship;
use my_standalone::{
    ica_controller::ICAController,
    msg::{MyStandaloneExecuteMsgFns, MyStandaloneInstantiateMsg, MyStandaloneQueryMsgFns},
    MyStandaloneInterface, MY_NAMESPACE,
};

use abstract_client::{AbstractClient, Application};
use abstract_standalone::{objects::namespace::Namespace, std::standalone};
use cosmwasm_std::coins;
// Use prelude to get all the necessary imports
use cw_orch::{
    anyhow,
    daemon::{json_lock::JsonLockedState, DaemonState},
    prelude::*,
    tokio::runtime::Runtime,
};
use cw_orch_interchain::prelude::*;
use networks::{JUNO_1, OSMOSIS_1};

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
        let publisher = abs_src.publisher_builder(namespace).build()?;

        let sub_account = abs_src
            .account_builder()
            .sub_account(publisher.account())
            .build()?;
        let standalone = sub_account.install_standalone::<MyStandaloneInterface<_>>(
            &MyStandaloneInstantiateMsg {
                base: standalone::StandaloneInstantiateMsg {
                    ans_host_address: abs_src.name_service().addr_str()?,
                    version_control_address: abs_src.version_control().addr_str()?,
                },
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
        let publisher = abs_src.publisher_builder(namespace).build()?;
        publisher.publish_standalone::<MyStandaloneInterface<_>>()?;

        let sub_account = abs_src
            .account_builder()
            .sub_account(publisher.account())
            .build()?;
        let standalone = sub_account.install_standalone::<MyStandaloneInterface<_>>(
            &MyStandaloneInstantiateMsg {
                base: standalone::StandaloneInstantiateMsg {
                    ans_host_address: abs_src.name_service().addr_str()?,
                    version_control_address: abs_src.version_control().addr_str()?,
                },
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

// See https://github.com/AbstractSDK/cw-orchestrator/pull/424
fn prepare_state_for_both_chains() {
    let state_file = DaemonState::state_file_path().unwrap();
    let mut json_locked_state = JsonLockedState::new(&state_file);

    json_locked_state.prepare(&JUNO_1.chain_id, &JUNO_1.network_info.chain_name, "default");
    json_locked_state.prepare(
        &OSMOSIS_1.chain_id,
        &OSMOSIS_1.network_info.chain_name,
        "default",
    );
}

#[test]
fn test_install() -> anyhow::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    // Make sure we don't corrupt actual state
    // Don't forget to remove this file if it's fresh starship
    std::env::set_var(
        cw_orch::daemon::env::STATE_FILE_ENV_NAME,
        "starship-state.json",
    );
    env_logger::init();
    prepare_state_for_both_chains();
    let runtime = Runtime::new().unwrap();

    let starship = Starship::new(runtime.handle(), None)?;
    let daemon_interchain = starship.interchain_env();
    let juno = daemon_interchain.chain("juno-1")?;
    let stargaze = daemon_interchain.chain("stargaze-1")?;

    let ibc_path = runtime.block_on(async {
        starship
            .client()
            .registry()
            .await
            .ibc_path("juno-1", "stargaze-1")
            .await
    })?;

    let test_env = TestEnv::setup(juno.clone(), stargaze.clone())?;
    let dst_account = test_env.abs_dst.account_builder().build()?;
    let dst_proxy = dst_account.proxy()?;

    let _ = daemon_interchain.check_ibc(
        "juno-1",
        test_env.standalone.create_ica_contract(
            ChannelOpenInitOptions {
                connection_id: ibc_path.chain_1.connection_id.to_string(),
                counterparty_connection_id: ibc_path.chain_2.connection_id.to_string(),
                counterparty_port_id: None,
                tx_encoding: None,
                channel_ordering: Some(cosmwasm_std::IbcOrder::Ordered),
            },
            None,
        )?,
    )?;

    // Waiting for channel to open
    {
        std::thread::sleep(Duration::from_secs(30));
        // let mut stdout_lock = std::io::stdout().lock();
        // writeln!(
        //     stdout_lock,
        //     "Waiting for channel to open, use this command to open it:"
        // )
        // .unwrap();
        // // kubectl exec -it hermes-osmo-juno-0 -- hermes tx chan-open-try --src-chain juno-1 --dst-chain stargaze-1 --dst-connection connection-0 --dst-port icahost --src-port wasm.juno1dxc3x3x8terrgls077h0hwqgsnec6fyqwtfak8mtg0shultqevdqsdvhcf --src-channel channel-5
        // writeln!(stdout_lock, "kubectl exec -it hermes-osmo-juno-0 -- hermes tx chan-open-try --src-chain juno-1 --dst-chain stargaze-1 --dst-connection connection-0 --dst-port icahost --src-port wasm.{ica_controller_addr} --src-channel channel-5").unwrap();
        // let mut line = String::new();
        // std::io::stdin().read_line(&mut line)?;
    }

    let last_ica_account = dbg!(test_env.standalone.ica_count()?.count) - 1;
    let state = dbg!(test_env.standalone.ica_contract_state(last_ica_account)?);
    test_env.ica_controller.set_address(&state.contract_addr);
    let ica_state: cw_ica_controller::types::state::ContractState = test_env
        .ica_controller
        .query(&cw_ica_controller::types::msg::QueryMsg::GetContractState {})?;
    let ica_channel_state: cw_ica_controller::types::state::ChannelState = test_env
        .ica_controller
        .query(&cw_ica_controller::types::msg::QueryMsg::GetChannel {})?;
    dbg!(ica_state, ica_channel_state);

    let ica_addr = state.ica_state.unwrap().ica_addr;

    let amount = coins(10_000, "ustars");
    runtime.block_on(stargaze.wallet().bank_send(&ica_addr, amount.clone()))?;

    test_env.standalone.send_action(
        last_ica_account,
        cosmwasm_std::CosmosMsg::Bank(cosmwasm_std::BankMsg::Send {
            to_address: dst_proxy.to_string(),
            amount: coins(100, "ustars"),
        }),
    )?;
    {
        std::thread::sleep(Duration::from_secs(30));
        // let mut stdout_lock = std::io::stdout().lock();
        // writeln!(
        //     stdout_lock,
        //     "Waiting for execution to go through IBC to open, use this commands to receive packet:"
        // )
        // .unwrap();
        // // kubectl exec -it hermes-osmo-juno-0 -- hermes tx chan-open-try --src-chain juno-1 --dst-chain stargaze-1 --dst-connection connection-0 --dst-port icahost --src-port wasm.juno1dxc3x3x8terrgls077h0hwqgsnec6fyqwtfak8mtg0shultqevdqsdvhcf --src-channel channel-5
        // writeln!(stdout_lock, "kubectl exec -it hermes-osmo-juno-0 -- hermes tx packet-recv --dst-chain stargaze-1 --src-chain juno-1 --src-port wasm.{ica_controller_addr} --src-channel channel-5").unwrap();
        // writeln!(stdout_lock, "kubectl exec -it hermes-osmo-juno-0 -- hermes tx packet-ack --dst-chain juno-1 --src-chain stargaze-1 --src-port icahost --src-channel channel-1").unwrap();
        // let mut line = String::new();
        // std::io::stdin().read_line(&mut line)?;
    }
    let dst_proxy_balance = stargaze.balance(dst_proxy, None)?;
    assert_eq!(dst_proxy_balance, coins(100, "ustars"));
    Ok(())
}
