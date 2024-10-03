use abstract_client::{AbstractClient, Application};
use abstract_standalone::{objects::namespace::Namespace, std::standalone};
use cosmwasm_std::coins;
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
        let publisher = abs_src.publisher_builder(namespace).build()?;

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

        let abs_src = AbstractClient::builder(src_env).build(src_env.sender().clone())?;
        let abs_dst = AbstractClient::builder(dst_env).build(dst_env.sender().clone())?;

        // Publish the standalone
        let publisher = abs_src.publisher_builder(namespace).build()?;
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

#[test]
fn test_bank_send() -> anyhow::Result<()> {
    let _ = env_logger::builder().is_test(true).try_init();
    // Make sure we don't corrupt actual state
    // Don't forget to remove this file if it's fresh starship
    std::env::set_var(
        cw_orch::daemon::env::STATE_FILE_ENV_NAME,
        "starship-state.json",
    );
    // Some txs don't succeed with default gas_buffer
    std::env::set_var(cw_orch::daemon::env::GAS_BUFFER_ENV_NAME, "1.8");

    let starship = Starship::new(None)?;
    let daemon_interchain = starship.interchain_env();
    let juno = daemon_interchain.chain("juno-1")?;
    let osmosis = daemon_interchain.chain("osmosis-1")?;

    let ibc_path = RUNTIME.block_on(async {
        starship
            .client()
            .registry()
            .await
            .ibc_path("juno-1", "osmosis-1")
            .await
    })?;

    let test_env = TestEnv::setup(juno.clone(), osmosis.clone())?;

    let _ = daemon_interchain.check_ibc(
        "juno-1",
        test_env.standalone.create_ica_contract(
            ChannelOpenInitOptions {
                connection_id: ibc_path.chain_1.connection_id.to_string(),
                counterparty_connection_id: ibc_path.chain_2.connection_id.to_string(),
                counterparty_port_id: None,
                channel_ordering: Some(cosmwasm_std::IbcOrder::Ordered),
            },
            None,
        )?,
    )?;

    // First ica account id is 0
    let mut current_ica_account_id = 0;

    // Waiting for channel to open, cw-orch not capable of waiting channel open
    std::thread::sleep(Duration::from_secs(15));

    let state = test_env
        .standalone
        .ica_contract_state(current_ica_account_id)?;
    let ica_addr = state.ica_state.unwrap().ica_addr;

    // Send 10_000 uosmo from ICA to some address
    let receiving_addr = test_env.abs_dst.registry().addr_str()?;
    let amount = coins(10_000, "uosmo");
    RUNTIME.block_on(osmosis.wallet().bank_send(&ica_addr, amount.clone()))?;

    let _ = daemon_interchain.check_ibc(
        "juno-1",
        test_env.standalone.send_action(
            current_ica_account_id,
            cosmwasm_std::CosmosMsg::Bank(cosmwasm_std::BankMsg::Send {
                to_address: receiving_addr.to_string(),
                amount: amount.clone(),
            }),
        )?,
    )?;

    let dst_proxy_balance = osmosis.balance(receiving_addr.clone(), None)?;
    assert_eq!(dst_proxy_balance, amount);

    // It's possible to use control multiple ica with same ica controller

    let _ = daemon_interchain.check_ibc(
        "juno-1",
        test_env.standalone.create_ica_contract(
            ChannelOpenInitOptions {
                connection_id: ibc_path.chain_1.connection_id.to_string(),
                counterparty_connection_id: ibc_path.chain_2.connection_id.to_string(),
                counterparty_port_id: None,
                channel_ordering: Some(cosmwasm_std::IbcOrder::Ordered),
            },
            None,
        )?,
    )?;

    current_ica_account_id += 1;

    // Waiting for channel to open, cw-orch not capable of waiting channel open
    std::thread::sleep(Duration::from_secs(15));

    let state = test_env
        .standalone
        .ica_contract_state(current_ica_account_id)?;
    let ica_addr = state.ica_state.unwrap().ica_addr;

    // Send 15_000 uosmo from ICA to some address
    let receiving_addr = test_env.abs_dst.name_service().addr_str()?;
    let amount = coins(15_000, "uosmo");
    RUNTIME.block_on(osmosis.wallet().bank_send(&ica_addr, amount.clone()))?;

    let _ = daemon_interchain.check_ibc(
        "juno-1",
        test_env.standalone.send_action(
            current_ica_account_id,
            cosmwasm_std::CosmosMsg::Bank(cosmwasm_std::BankMsg::Send {
                to_address: receiving_addr.to_string(),
                amount: amount.clone(),
            }),
        )?,
    )?;

    let dst_proxy_balance = osmosis.balance(receiving_addr, None)?;
    assert_eq!(dst_proxy_balance, amount);

    Ok(())
}
