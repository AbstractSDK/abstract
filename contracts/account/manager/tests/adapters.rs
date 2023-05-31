mod common;

use abstract_adapter::mock::{MockExecMsg, MockInitMsg};
use abstract_core::manager::ManagerModuleInfo;
use abstract_core::objects::module::{ModuleInfo, ModuleVersion};
use abstract_core::{adapter::BaseQueryMsgFns, *};
use abstract_interface::*;
use abstract_testing::prelude::{OWNER, TEST_ACCOUNT_ID, TEST_MODULE_ID, TEST_VERSION};
use common::{create_default_account, init_mock_adapter, AResult, TEST_COIN};
use cosmwasm_std::{Addr, Coin, Empty};
use cw_orch::deploy::Deploy;
use cw_orch::prelude::*;
// use cw_multi_test::StakingInfo;
use speculoos::{assert_that, result::ResultAssertions, string::StrAssertions};

use crate::common::mock_modules::{BootMockAdapter1V1, BootMockAdapter1V2, V1, V2};

fn install_adapter(manager: &Manager<Mock>, adapter_id: &str) -> AResult {
    manager
        .install_module(adapter_id, &Empty {})
        .map_err(Into::into)
}

pub(crate) fn uninstall_module(manager: &Manager<Mock>, module_id: &str) -> AResult {
    manager
        .uninstall_module(module_id.to_string())
        .map_err(Into::<CwOrchError>::into)?;
    Ok(())
}

#[test]
fn installing_one_adapter_should_succeed() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain.clone(), Empty {})?;
    let account = create_default_account(&deployment.account_factory)?;
    let staking_adapter = init_mock_adapter(chain, &deployment, None)?;
    install_adapter(&account.manager, TEST_MODULE_ID)?;

    let modules = account.expect_modules(vec![staking_adapter.address()?.to_string()])?;

    assert_that(&modules[1]).is_equal_to(&ManagerModuleInfo {
        address: staking_adapter.address()?,
        id: TEST_MODULE_ID.to_string(),
        version: cw2::ContractVersion {
            contract: TEST_MODULE_ID.into(),
            version: TEST_VERSION.into(),
        },
    });

    // Configuration is correct
    let adapter_config = staking_adapter.config()?;
    assert_that!(adapter_config).is_equal_to(adapter::AdapterConfigResponse {
        ans_host_address: deployment.ans_host.address()?,
        dependencies: vec![],
        version_control_address: deployment.version_control.address()?,
    });

    // no authorized addresses registered
    let authorized = staking_adapter.authorized_addresses(account.proxy.addr_str()?)?;
    assert_that!(authorized)
        .is_equal_to(adapter::AuthorizedAddressesResponse { addresses: vec![] });

    Ok(())
}

#[test]
fn install_non_existent_adapterid_should_fail() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain, Empty {})?;
    let account = create_default_account(&deployment.account_factory)?;

    let res = install_adapter(&account.manager, "lol:no_chance");

    assert_that!(res).is_err();
    Ok(())
}

#[test]
fn install_non_existent_version_should_fail() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain.clone(), Empty {})?;
    let account = create_default_account(&deployment.account_factory)?;
    init_mock_adapter(chain, &deployment, None)?;

    let res = account.manager.install_module_version(
        TEST_MODULE_ID,
        ModuleVersion::Version("1.2.3".to_string()),
        &Empty {},
    );

    // testtodo: check error
    assert_that!(res).is_err();

    Ok(())
}

#[test]
fn installation_of_duplicate_adapter_should_fail() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain.clone(), Empty {})?;
    let account = create_default_account(&deployment.account_factory)?;
    let staking_adapter = init_mock_adapter(chain, &deployment, None)?;

    install_adapter(&account.manager, TEST_MODULE_ID)?;

    let modules = account.expect_modules(vec![staking_adapter.address()?.to_string()])?;

    // assert proxy module
    // check staking adapter
    assert_that(&modules[1]).is_equal_to(&ManagerModuleInfo {
        address: staking_adapter.address()?,
        id: TEST_MODULE_ID.to_string(),
        version: cw2::ContractVersion {
            contract: TEST_MODULE_ID.into(),
            version: TEST_VERSION.into(),
        },
    });

    // install again
    let second_install_res = install_adapter(&account.manager, TEST_MODULE_ID);
    assert_that!(second_install_res)
        .is_err()
        .matches(|e| e.to_string().contains("test-module-id"));

    account.expect_modules(vec![staking_adapter.address()?.to_string()])?;

    Ok(())
}

#[test]
fn reinstalling_adapter_should_be_allowed() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain.clone(), Empty {})?;
    let account = create_default_account(&deployment.account_factory)?;
    let staking_adapter = init_mock_adapter(chain, &deployment, None)?;

    install_adapter(&account.manager, TEST_MODULE_ID)?;

    let modules = account.expect_modules(vec![staking_adapter.address()?.to_string()])?;

    // check staking adapter
    assert_that(&modules[1]).is_equal_to(&ManagerModuleInfo {
        address: staking_adapter.address()?,
        id: TEST_MODULE_ID.to_string(),
        version: cw2::ContractVersion {
            contract: TEST_MODULE_ID.into(),
            version: TEST_VERSION.into(),
        },
    });

    // uninstall
    uninstall_module(&account.manager, TEST_MODULE_ID)?;

    // None expected
    account.expect_modules(vec![])?;

    // reinstall
    install_adapter(&account.manager, TEST_MODULE_ID)?;

    account.expect_modules(vec![staking_adapter.address()?.to_string()])?;

    Ok(())
}

/// Reinstalling the Adapter should install the latest version
#[test]
fn reinstalling_new_version_should_install_latest() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain.clone(), Empty {})?;
    let account = create_default_account(&deployment.account_factory)?;
    deployment
        .version_control
        .claim_namespaces(TEST_ACCOUNT_ID, vec!["tester".to_string()])?;

    let adapter1 = BootMockAdapter1V1::new_test(chain.clone());
    adapter1.deploy(V1.parse().unwrap(), MockInitMsg).unwrap();

    install_adapter(&account.manager, &adapter1.id())?;

    let modules = account.expect_modules(vec![adapter1.address()?.to_string()])?;

    // check staking adapter
    assert_that(&modules[1]).is_equal_to(&ManagerModuleInfo {
        address: adapter1.address()?,
        id: adapter1.id().to_string(),
        version: cw2::ContractVersion {
            contract: adapter1.id().into(),
            version: V1.into(),
        },
    });

    // uninstall tendermint staking
    uninstall_module(&account.manager, &adapter1.id())?;

    account.expect_modules(vec![])?;

    let old_adapter_addr = adapter1.address()?;

    let adapter2 = BootMockAdapter1V2::new_test(chain.clone());

    adapter2.deploy(V2.parse().unwrap(), MockInitMsg).unwrap();

    // check that the latest staking version is the new one
    let latest_staking = deployment
        .version_control
        .module(ModuleInfo::from_id_latest(&adapter1.id())?)?;
    assert_that!(latest_staking.info.version).is_equal_to(ModuleVersion::Version(V2.to_string()));

    // reinstall
    install_adapter(&account.manager, &adapter2.id())?;

    let modules = account.expect_modules(vec![adapter2.address()?.to_string()])?;

    assert_that!(modules[1]).is_equal_to(&ManagerModuleInfo {
        // the address stored for BootMockAdapter was updated when we instantiated the new version, so this is the new address
        address: adapter2.address()?,
        id: adapter2.id(),
        version: cw2::ContractVersion {
            contract: adapter2.id(),
            // IMPORTANT: The version of the contract did not change although the version of the module in version control did.
            // Beware of this distinction. The version of the contract is the version that's imbedded into the contract's wasm on compilation.
            version: V2.to_string(),
        },
    });
    // assert that the new staking adapter has a different address
    assert_ne!(old_adapter_addr, adapter2.address()?);

    assert_that!(modules[1].address).is_equal_to(adapter2.as_instance().address()?);

    Ok(())
}

// struct TestModule = AppContract

#[test]
fn unauthorized_exec() -> AResult {
    let sender = Addr::unchecked(OWNER);
    let unauthorized = Addr::unchecked("unauthorized");
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain.clone(), Empty {})?;
    let account = create_default_account(&deployment.account_factory)?;
    let staking_adapter = init_mock_adapter(chain, &deployment, None)?;
    install_adapter(&account.manager, TEST_MODULE_ID)?;
    // non-authorized address cannot execute
    let res = staking_adapter
        .call_as(&unauthorized)
        .execute(&MockExecMsg.into(), None)
        .unwrap_err();
    assert_that!(res.root().to_string()).contains(
        "Sender: unauthorized of request to tester:test-module-id is not a Manager or Authorized Address",
    );
    // neither can the ROOT directly
    let res = staking_adapter
        .execute(&MockExecMsg.into(), None)
        .unwrap_err();
    assert_that!(&res.root().to_string()).contains(
        "Sender: owner of request to tester:test-module-id is not a Manager or Authorized Address",
    );
    Ok(())
}

#[test]
fn manager_adapter_exec() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain.clone(), Empty {})?;
    let account = create_default_account(&deployment.account_factory)?;
    let _staking_adapter_one = init_mock_adapter(chain.clone(), &deployment, None)?;

    install_adapter(&account.manager, TEST_MODULE_ID)?;

    chain.set_balance(
        &account.proxy.address()?,
        vec![Coin::new(100_000, TEST_COIN)],
    )?;

    account.manager.execute_on_module(
        TEST_MODULE_ID,
        Into::<abstract_core::adapter::ExecuteMsg<MockExecMsg>>::into(MockExecMsg),
    )?;

    Ok(())
}

#[test]
fn installing_specific_version_should_install_expected() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain.clone(), Empty {})?;
    let account = create_default_account(&deployment.account_factory)?;
    deployment
        .version_control
        .claim_namespaces(TEST_ACCOUNT_ID, vec!["tester".to_string()])?;

    let adapter1 = BootMockAdapter1V1::new_test(chain.clone());
    adapter1.deploy(V1.parse().unwrap(), MockInitMsg).unwrap();

    let v1_adapter_addr = adapter1.address()?;

    let adapter2 = BootMockAdapter1V2::new_test(chain.clone());

    adapter2.deploy(V2.parse().unwrap(), MockInitMsg).unwrap();

    let expected_version = "1.0.0".to_string();

    // install specific version
    account.manager.install_module_version(
        &adapter1.id(),
        ModuleVersion::Version(expected_version),
        &MockInitMsg {},
    )?;

    let modules = account.expect_modules(vec![v1_adapter_addr.to_string()])?;
    let installed_module: ManagerModuleInfo = modules[1].clone();
    assert_that!(installed_module.id).is_equal_to(adapter1.id());

    Ok(())
}
