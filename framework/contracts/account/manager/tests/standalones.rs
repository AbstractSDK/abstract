use abstract_integration_tests::{create_default_account, AResult};
use abstract_interface::*;
use abstract_manager::error::ManagerError;
use abstract_standalone::{
    gen_standalone_mock,
    mock::{MockExecMsgFns, MockQueryMsgFns, MockQueryResponse},
};
use abstract_testing::prelude::*;
use cosmwasm_std::Binary;
use cw_orch::prelude::*;

const STANDALONE_ID: &str = "tester:standalone";
const STANDALONE_VERSION: &str = "1.0.0";
gen_standalone_mock!(MockStandalone, STANDALONE_ID, STANDALONE_VERSION);

#[test]
fn account_install_standalone() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;

    deployment
        .version_control
        .claim_namespace(TEST_ACCOUNT_ID, "tester".to_owned())?;
    let standalone = MockStandalone::new(STANDALONE_ID, chain);
    standalone.deploy(STANDALONE_VERSION.parse().unwrap(), DeployStrategy::Try)?;
    account.install_standalone(
        &standalone,
        &MockInitMsg {
            base: standalone::StandaloneInstantiateMsg {
                ans_host_address: deployment.ans_host.addr_str()?,
                version_control_address: deployment.version_control.addr_str()?,
            },
            random_field: "LMAO".to_owned(),
        },
        &[],
    )?;
    // Check some actions
    let r = standalone.do_something()?;
    assert_eq!(r.data, Some(Binary::from(b"mock_exec")));
    let something = standalone.get_something()?;
    assert_eq!(something, MockQueryResponse {});
    Ok(())
}

#[test]
fn cant_reinstall_standalone_after_uninstall() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;

    deployment
        .version_control
        .claim_namespace(TEST_ACCOUNT_ID, "tester".to_owned())?;

    let standalone = MockStandalone::new_test(chain.clone());
    standalone.deploy(STANDALONE_VERSION.parse().unwrap(), DeployStrategy::Try)?;
    account.install_standalone(
        &standalone,
        &MockInitMsg {
            base: standalone::StandaloneInstantiateMsg {
                ans_host_address: deployment.ans_host.addr_str()?,
                version_control_address: deployment.version_control.addr_str()?,
            },
            random_field: "foo".to_owned(),
        },
        &[],
    )?;

    // Reinstall
    account.account.uninstall_module(STANDALONE_ID.to_owned())?;
    let Err(AbstractInterfaceError::Orch(err)) = account.install_standalone(
        &standalone,
        &MockInitMsg {
            base: standalone::StandaloneInstantiateMsg {
                ans_host_address: deployment.ans_host.addr_str()?,
                version_control_address: deployment.version_control.addr_str()?,
            },
            random_field: "foo".to_owned(),
        },
        &[],
    ) else {
        panic!("Expected error");
    };
    let manager_err: ManagerError = err.downcast().unwrap();
    assert_eq!(manager_err, ManagerError::ProhibitedReinstall {});
    Ok(())
}
