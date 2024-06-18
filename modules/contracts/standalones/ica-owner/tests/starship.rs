use cw_controllers::AdminError;
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
use cw_orch::{anyhow, prelude::*};
use cw_orch_interchain::prelude::*;

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

        // let abs_src = AbstractClient::new(src_env)?;
        // let abs_dst = AbstractClient::new(dst_env)?;

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

#[test]
fn test_install() -> anyhow::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let starship = Starship::new(cw_orch::daemon::RUNTIME.handle(), None)?;
    let juno = starship.daemon("juno-1")?.clone();
    let osmosis = starship.daemon("osmosis-1")?.clone();

    let test_env = TestEnv::setup(juno.clone(), osmosis.clone())?;
    Ok(())
}

#[test]
fn test_mock_install() -> anyhow::Result<()> {
    let mock_interchain =
        MockBech32InterchainEnv::new(vec![("juno-1", "juno"), ("osmosis-1", "osmo")]);
    let juno = mock_interchain.chain("juno-1")?;
    let osmosis = mock_interchain.chain("osmosis-1")?;

    let test_env = TestEnv::setup(juno.clone(), osmosis.clone())?;
    Ok(())
}
