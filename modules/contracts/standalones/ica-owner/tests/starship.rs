use cw_controllers::AdminError;
use cw_ica_controller::types::msg::options::ChannelOpenInitOptions;
use cw_orch_starship::Starship;
use my_standalone::{
    ica_controller::ICAController,
    msg::{
        ConfigResponse, ICACountResponse, MyStandaloneExecuteMsgFns, MyStandaloneInstantiateMsg,
        MyStandaloneQueryMsgFns,
    },
    MyStandaloneError, MyStandaloneInterface, MY_NAMESPACE,
};

use abstract_client::{AbstractClient, Application, Environment};
use abstract_standalone::{
    objects::namespace::Namespace,
    std::{osmosis, standalone},
};
use cosmwasm_std::coins;
// Use prelude to get all the necessary imports
use cw_orch::{
    anyhow,
    daemon::{json_lock::JsonLockedState, DaemonState, DaemonStateFile},
    prelude::*,
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
    /// Set up the test environment with an Account that has the Standalone installed
    fn setup(src_env: Env, dst_env: Env) -> anyhow::Result<TestEnv<Env>> {
        let ica_controller = ICAController::new(src_env.clone());
        let resp = ica_controller.upload()?;
        let ica_controller_code_id = resp.uploaded_code_id()?;

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
    let mut json_locked_state = JsonLockedState::new(&DaemonState::state_file_path().unwrap());

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
    std::env::set_var("STATE_FILE", "starship-state.json");
    env_logger::init();
    prepare_state_for_both_chains();
    let runtime = &cw_orch::daemon::RUNTIME;

    let starship = Starship::new(runtime.handle(), None)?;
    let juno = starship.daemon("juno-1")?.clone();
    let osmosis = starship.daemon("osmosis-1")?.clone();
    let daemon_interchain = cw_orch_interchain::DaemonInterchainEnv::from_daemons(
        runtime.handle(),
        vec![juno.clone(), osmosis.clone()],
        &starship,
    );
    // let bal = juno.balance(juno.sender(), None)?;
    runtime.block_on(starship.client().create_channel(
        "juno-1",
        "osmosis-1",
        "a",
        "b",
        "gg",
        Some(cosmwasm_std::IbcOrder::Unordered),
    ))?;

    let ibc_path = runtime.block_on(async {
        starship
            .client()
            .registry()
            .await
            .ibc_path("juno-1", "osmosis-1")
            .await
    })?;

    let test_env = TestEnv::setup(juno.clone(), osmosis.clone())?;
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
                channel_ordering: None,
            },
            None,
        )?,
    )?;
    let state = dbg!(test_env.standalone.ica_contract_state(0)?);
    let ica_addr = state.ica_state.unwrap().ica_addr;

    let amount = coins(100, "uosmo");
    runtime.block_on(osmosis.wallet().bank_send(&ica_addr, amount.clone()))?;
    let _ = daemon_interchain.check_ibc(
        "juno-1",
        test_env.standalone.send_action(
            0,
            cosmwasm_std::CosmosMsg::Bank(cosmwasm_std::BankMsg::Send {
                to_address: dst_proxy.to_string(),
                amount: amount.clone(),
            }),
        )?,
    );
    let dst_proxy_balance = osmosis.balance(dst_proxy, None)?;
    assert_eq!(dst_proxy_balance, amount);
    Ok(())
}

// #[test]
// fn test_mock_install() -> anyhow::Result<()> {
//     let mock_interchain =
//         MockBech32InterchainEnv::new(vec![("juno-1", "juno"), ("osmosis-1", "osmo")]);
//     let juno = mock_interchain.chain("juno-1")?;
//     let osmosis = mock_interchain.chain("osmosis-1")?;

//     let test_env = TestEnv::setup(juno.clone(), osmosis.clone())?;
//     Ok(())
// }
