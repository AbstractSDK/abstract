mod common;

use abstract_api::mock::MockExecMsg;
use abstract_boot::*;
use abstract_core::manager::ManagerModuleInfo;
use abstract_core::objects::module::{ModuleInfo, ModuleVersion};
use abstract_core::{api::BaseQueryMsgFns, *};
use abstract_testing::prelude::{OWNER, TEST_MODULE_ID, TEST_VERSION};
use boot_core::{
    BootError, Mock, TxHandler, {instantiate_default_mock_env, CallAs, ContractInstance},
};
use boot_core::{BootExecute, Deploy};
use common::{create_default_account, init_mock_api, AResult, TEST_COIN};
use cosmwasm_std::{Addr, Coin, Empty};
// use cw_multi_test::StakingInfo;
use speculoos::{assert_that, result::ResultAssertions, string::StrAssertions};

fn install_api(manager: &Manager<Mock>, api: &str) -> AResult {
    manager.install_module(api, &Empty {}).map_err(Into::into)
}

pub(crate) fn uninstall_module(manager: &Manager<Mock>, api: &str) -> AResult {
    manager
        .uninstall_module(api.to_string())
        .map_err(Into::<BootError>::into)?;
    Ok(())
}

#[test]
fn installing_one_api_should_succeed() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let (_state, chain) = instantiate_default_mock_env(&sender)?;
    let deployment = Abstract::deploy_on(chain.clone(), TEST_VERSION.parse().unwrap())?;
    let account = create_default_account(&deployment.account_factory)?;
    let staking_api = init_mock_api(chain, &deployment, None)?;
    install_api(&account.manager, TEST_MODULE_ID)?;

    let modules = account.expect_modules(vec![staking_api.address()?.to_string()])?;

    assert_that(&modules[1]).is_equal_to(&ManagerModuleInfo {
        address: staking_api.address()?,
        id: TEST_MODULE_ID.to_string(),
        version: cw2::ContractVersion {
            contract: TEST_MODULE_ID.into(),
            version: TEST_VERSION.into(),
        },
    });

    // Configuration is correct
    let api_config = staking_api.config()?;
    assert_that!(api_config).is_equal_to(api::ApiConfigResponse {
        ans_host_address: deployment.ans_host.address()?,
        dependencies: vec![],
        version_control_address: deployment.version_control.address()?,
    });

    // no authorized addresses registered
    let authorized = staking_api.authorized_addresses(account.proxy.addr_str()?)?;
    assert_that!(authorized).is_equal_to(api::AuthorizedAddressesResponse { addresses: vec![] });

    Ok(())
}

#[test]
fn install_non_existent_apiname_should_fail() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let (_state, chain) = instantiate_default_mock_env(&sender)?;
    let deployment = Abstract::deploy_on(chain, TEST_VERSION.parse().unwrap())?;
    let account = create_default_account(&deployment.account_factory)?;

    let res = install_api(&account.manager, "lol:no_chance");

    assert_that!(res).is_err();
    Ok(())
}

#[test]
fn install_non_existent_version_should_fail() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let (_state, chain) = instantiate_default_mock_env(&sender)?;
    let deployment = Abstract::deploy_on(chain.clone(), TEST_VERSION.parse().unwrap())?;
    let account = create_default_account(&deployment.account_factory)?;
    init_mock_api(chain, &deployment, None)?;

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
fn installation_of_duplicate_api_should_fail() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let (_state, chain) = instantiate_default_mock_env(&sender)?;
    let deployment = Abstract::deploy_on(chain.clone(), TEST_VERSION.parse().unwrap())?;
    let account = create_default_account(&deployment.account_factory)?;
    let staking_api = init_mock_api(chain, &deployment, None)?;

    install_api(&account.manager, TEST_MODULE_ID)?;

    let modules = account.expect_modules(vec![staking_api.address()?.to_string()])?;

    // assert proxy module
    // check staking api
    assert_that(&modules[1]).is_equal_to(&ManagerModuleInfo {
        address: staking_api.address()?,
        id: TEST_MODULE_ID.to_string(),
        version: cw2::ContractVersion {
            contract: TEST_MODULE_ID.into(),
            version: TEST_VERSION.into(),
        },
    });

    // install again
    let second_install_res = install_api(&account.manager, TEST_MODULE_ID);
    assert_that!(second_install_res)
        .is_err()
        .matches(|e| e.to_string().contains("test-module-id"));

    account.expect_modules(vec![staking_api.address()?.to_string()])?;

    Ok(())
}

#[test]
fn reinstalling_api_should_be_allowed() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let (_state, chain) = instantiate_default_mock_env(&sender)?;
    let deployment = Abstract::deploy_on(chain.clone(), TEST_VERSION.parse().unwrap())?;
    let account = create_default_account(&deployment.account_factory)?;
    let staking_api = init_mock_api(chain, &deployment, None)?;

    install_api(&account.manager, TEST_MODULE_ID)?;

    let modules = account.expect_modules(vec![staking_api.address()?.to_string()])?;

    // check staking api
    assert_that(&modules[1]).is_equal_to(&ManagerModuleInfo {
        address: staking_api.address()?,
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
    install_api(&account.manager, TEST_MODULE_ID)?;

    account.expect_modules(vec![staking_api.address()?.to_string()])?;

    Ok(())
}

/// Reinstalling the API should install the latest version
#[test]
fn reinstalling_new_version_should_install_latest() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let (_state, chain) = instantiate_default_mock_env(&sender)?;
    let deployment = Abstract::deploy_on(chain.clone(), TEST_VERSION.parse().unwrap())?;
    let account = create_default_account(&deployment.account_factory)?;
    let staking_api = init_mock_api(chain.clone(), &deployment, Some("1.0.0".to_string()))?;

    install_api(&account.manager, TEST_MODULE_ID)?;

    let modules = account.expect_modules(vec![staking_api.address()?.to_string()])?;

    // check staking api
    assert_that(&modules[1]).is_equal_to(&ManagerModuleInfo {
        address: staking_api.address()?,
        id: TEST_MODULE_ID.to_string(),
        version: cw2::ContractVersion {
            contract: TEST_MODULE_ID.into(),
            version: TEST_VERSION.into(),
        },
    });

    // uninstall tendermint staking
    uninstall_module(&account.manager, TEST_MODULE_ID)?;

    account.expect_modules(vec![])?;

    // Register the new version
    let new_version_num = "100.0.0";
    let old_api_addr = staking_api.address()?;

    // We init the staking api with a new version to ensure that we get a new address
    let new_staking_api = init_mock_api(chain, &deployment, Some(new_version_num.to_string()))?;

    // check that the latest staking version is the new one
    let latest_staking = deployment
        .version_control
        .module(ModuleInfo::from_id_latest(TEST_MODULE_ID)?)?;
    assert_that!(latest_staking.info.version)
        .is_equal_to(ModuleVersion::Version(new_version_num.to_string()));

    // reinstall
    install_api(&account.manager, TEST_MODULE_ID)?;

    let modules = account.expect_modules(vec![new_staking_api.address()?.to_string()])?;

    assert_that!(modules[1]).is_equal_to(&ManagerModuleInfo {
        // the address stored for BootMockApi was updated when we instantiated the new version, so this is the new address
        address: new_staking_api.address()?,
        id: TEST_MODULE_ID.to_string(),
        version: cw2::ContractVersion {
            contract: TEST_MODULE_ID.into(),
            // IMPORTANT: The version of the contract did not change although the version of the module in version control did.
            // Beware of this distinction. The version of the contract is the version that's imbedded into the contract's wasm on compilation.
            version: TEST_VERSION.to_string(),
        },
    });
    // assert that the new staking api has a different address
    assert_ne!(old_api_addr, new_staking_api.address()?);

    assert_that!(modules[1].address).is_equal_to(new_staking_api.as_instance().address()?);

    Ok(())
}

// struct TestModule = AppContract

#[test]
fn unauthorized_exec() -> AResult {
    let sender = Addr::unchecked(OWNER);
    let unauthorized = Addr::unchecked("unauthorized");
    let (_state, chain) = instantiate_default_mock_env(&sender)?;
    let deployment = Abstract::deploy_on(chain.clone(), TEST_VERSION.parse().unwrap())?;
    let account = create_default_account(&deployment.account_factory)?;
    let staking_api = init_mock_api(chain, &deployment, None)?;
    install_api(&account.manager, TEST_MODULE_ID)?;
    // non-authorized address cannot execute
    let res = staking_api
        .call_as(&unauthorized)
        .execute(&MockExecMsg.into(), None)
        .unwrap_err();
    assert_that!(res.root().to_string()).contains(
        "Sender: unauthorized of request to tester:test-module-id is not a Manager or Authorized Address",
    );
    // neither can the ROOT directly
    let res = staking_api.execute(&MockExecMsg.into(), None).unwrap_err();
    assert_that!(&res.root().to_string()).contains(
        "Sender: owner of request to tester:test-module-id is not a Manager or Authorized Address",
    );
    Ok(())
}

#[test]
fn manager_api_exec_staking_delegation() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let (_state, chain) = instantiate_default_mock_env(&sender)?;
    let deployment = Abstract::deploy_on(chain.clone(), TEST_VERSION.parse().unwrap())?;
    let account = create_default_account(&deployment.account_factory)?;
    let _staking_api_one = init_mock_api(chain.clone(), &deployment, Some("1.2.3".to_string()))?;

    install_api(&account.manager, TEST_MODULE_ID)?;

    chain.set_balance(
        &account.proxy.address()?,
        vec![Coin::new(100_000, TEST_COIN)],
    )?;

    account.manager.execute_on_module(
        TEST_MODULE_ID,
        Into::<abstract_core::api::ExecuteMsg<MockExecMsg>>::into(MockExecMsg),
    )?;

    Ok(())
}

#[test]
fn installing_specific_version_should_install_expected() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let (_state, chain) = instantiate_default_mock_env(&sender)?;
    let deployment = Abstract::deploy_on(chain.clone(), TEST_VERSION.parse().unwrap())?;
    let account = create_default_account(&deployment.account_factory)?;
    let _staking_api_one = init_mock_api(chain.clone(), &deployment, Some("1.2.3".to_string()))?;
    let expected_version = "2.3.4".to_string();
    let expected_staking_api =
        init_mock_api(chain.clone(), &deployment, Some(expected_version.clone()))?;
    let expected_staking_api_addr = expected_staking_api.address()?.to_string();

    let _staking_api_three = init_mock_api(chain, &deployment, Some("3.4.5".to_string()))?;

    // install specific version
    account.manager.install_module_version(
        TEST_MODULE_ID,
        ModuleVersion::Version(expected_version),
        &Empty {},
    )?;

    let modules = account.expect_modules(vec![expected_staking_api_addr])?;
    let installed_module: ManagerModuleInfo = modules[1].clone();
    assert_that!(installed_module.id).is_equal_to(TEST_MODULE_ID.to_string());

    Ok(())
}
