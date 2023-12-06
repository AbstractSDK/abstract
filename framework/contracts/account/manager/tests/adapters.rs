mod common;

use abstract_adapter::mock::{self, MockError};
use abstract_adapter::mock::{MockExecMsg, MockInitMsg};
use abstract_adapter::AdapterError;
use abstract_core::adapter::{AdapterBaseMsg, AdapterRequestMsg, BaseExecuteMsg};
use abstract_core::manager::{ManagerModuleInfo, ModuleInstallConfig};
use abstract_core::objects::fee::FixedFee;
use abstract_core::objects::module::{ModuleInfo, ModuleVersion, Monetization};
use abstract_core::objects::AccountId;
use abstract_core::{adapter::BaseQueryMsgFns, *};
use abstract_interface::*;
use abstract_testing::prelude::*;
use common::*;
use cosmwasm_std::{coin, coins};
use cosmwasm_std::{Addr, Coin, Empty};
use cw_orch::deploy::Deploy;
use cw_orch::prelude::*;
use speculoos::{assert_that, result::ResultAssertions, string::StrAssertions};

use crate::common::mock_modules::{adapter_1, BootMockAdapter1V1, BootMockAdapter1V2, V1, V2};

#[test]
fn installing_one_adapter_should_succeed() -> AResult {
    let sender = Addr::unchecked(OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;
    let staking_adapter = init_mock_adapter(chain.clone(), &deployment, None)?;
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
    let adapter_config = staking_adapter.base_config()?;
    assert_that!(adapter_config).is_equal_to(adapter::AdapterConfigResponse {
        ans_host_address: deployment.ans_host.address()?,
        dependencies: vec![],
        version_control_address: deployment.version_control.address()?,
    });

    // no authorized addresses registered
    let authorized = staking_adapter.authorized_addresses(account.proxy.addr_str()?)?;
    assert_that!(authorized)
        .is_equal_to(adapter::AuthorizedAddressesResponse { addresses: vec![] });

    take_storage_snapshot!(chain, "install_one_adapter");

    Ok(())
}

#[test]
fn installing_one_adapter_without_fee_should_fail() -> AResult {
    let sender = Addr::unchecked(OWNER);
    let chain = Mock::new(&sender);
    chain.set_balance(&sender, coins(12, "ujunox"))?;
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;
    init_mock_adapter(chain.clone(), &deployment, None)?;
    add_mock_adapter_install_fee(
        chain,
        &deployment,
        Monetization::InstallFee(FixedFee::new(&coin(45, "ujunox"))),
        None,
    )?;
    // TODO, match the exact error
    assert_that!(install_adapter(&account.manager, TEST_MODULE_ID)).is_err();

    // TODO, match the exact error
    assert_that!(install_adapter_with_funds(
        &account.manager,
        TEST_MODULE_ID,
        &coins(12, "ujunox")
    ))
    .is_err();

    Ok(())
}

#[test]
fn installing_one_adapter_with_fee_should_succeed() -> AResult {
    let sender = Addr::unchecked(OWNER);
    let chain = Mock::new(&sender);
    chain.set_balance(&sender, coins(45, "ujunox"))?;
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;
    init_mock_adapter(chain.clone(), &deployment, None)?;
    add_mock_adapter_install_fee(
        chain.clone(),
        &deployment,
        Monetization::InstallFee(FixedFee::new(&coin(45, "ujunox"))),
        None,
    )?;

    assert_that!(install_adapter_with_funds(
        &account.manager,
        TEST_MODULE_ID,
        &coins(45, "ujunox")
    ))
    .is_ok();

    take_storage_snapshot!(chain, "install_one_adapter_with_fee");

    Ok(())
}

#[test]
fn install_non_existent_adapterid_should_fail() -> AResult {
    let sender = Addr::unchecked(OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain, sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;

    let res = install_adapter(&account.manager, "lol:no_chance");

    assert_that!(res).is_err();
    Ok(())
}

#[test]
fn install_non_existent_version_should_fail() -> AResult {
    let sender = Addr::unchecked(OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;
    init_mock_adapter(chain, &deployment, None)?;

    let res = account.manager.install_module_version(
        TEST_MODULE_ID,
        ModuleVersion::Version("1.2.3".to_string()),
        Some(&Empty {}),
        None,
    );

    // testtodo: check error
    assert_that!(res).is_err();

    Ok(())
}

#[test]
fn installation_of_duplicate_adapter_should_fail() -> AResult {
    let sender = Addr::unchecked(OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
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
    let sender = Addr::unchecked(OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;
    let staking_adapter = init_mock_adapter(chain.clone(), &deployment, None)?;

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
    take_storage_snapshot!(chain, "reinstalling_adapter_should_be_allowed");

    Ok(())
}

/// Reinstalling the Adapter should install the latest version
#[test]
fn reinstalling_new_version_should_install_latest() -> AResult {
    let sender = Addr::unchecked(OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;
    deployment
        .version_control
        .claim_namespace(TEST_ACCOUNT_ID, "tester".to_string())?;

    let adapter1 = BootMockAdapter1V1::new_test(chain.clone());
    adapter1
        .deploy(V1.parse().unwrap(), MockInitMsg, DeployStrategy::Try)
        .unwrap();

    install_adapter(&account.manager, &adapter1.id())?;

    let modules = account.expect_modules(vec![adapter1.address()?.to_string()])?;

    // check staking adapter
    assert_that(&modules[1]).is_equal_to(&ManagerModuleInfo {
        address: adapter1.address()?,
        id: adapter1.id(),
        version: cw2::ContractVersion {
            contract: adapter1.id(),
            version: V1.into(),
        },
    });

    // uninstall tendermint staking
    uninstall_module(&account.manager, &adapter1.id())?;

    account.expect_modules(vec![])?;

    let old_adapter_addr = adapter1.address()?;

    let adapter2 = BootMockAdapter1V2::new_test(chain.clone());

    adapter2
        .deploy(V2.parse().unwrap(), MockInitMsg, DeployStrategy::Try)
        .unwrap();

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
    take_storage_snapshot!(chain, "reinstalling_new_version_should_install_latest");

    Ok(())
}

// struct TestModule = AppContract

#[test]
fn unauthorized_exec() -> AResult {
    let sender = Addr::unchecked(OWNER);
    let unauthorized = Addr::unchecked("unauthorized");
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
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
    let sender = Addr::unchecked(OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
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
    let sender = Addr::unchecked(OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;
    deployment
        .version_control
        .claim_namespace(TEST_ACCOUNT_ID, "tester".to_string())?;

    let adapter1 = BootMockAdapter1V1::new_test(chain.clone());
    adapter1
        .deploy(V1.parse().unwrap(), MockInitMsg, DeployStrategy::Try)
        .unwrap();

    let v1_adapter_addr = adapter1.address()?;

    let adapter2 = BootMockAdapter1V2::new_test(chain.clone());

    adapter2
        .deploy(V2.parse().unwrap(), MockInitMsg, DeployStrategy::Try)
        .unwrap();

    let expected_version = "1.0.0".to_string();

    // install specific version
    account.manager.install_module_version(
        &adapter1.id(),
        ModuleVersion::Version(expected_version),
        Some(&MockInitMsg {}),
        None,
    )?;

    let modules = account.expect_modules(vec![v1_adapter_addr.to_string()])?;
    let installed_module: ManagerModuleInfo = modules[1].clone();
    assert_that!(installed_module.id).is_equal_to(adapter1.id());
    take_storage_snapshot!(chain, "installing_specific_version_should_install_expected");

    Ok(())
}

#[test]
fn account_install_adapter() -> AResult {
    let sender = Addr::unchecked(OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;

    deployment
        .version_control
        .claim_namespace(TEST_ACCOUNT_ID, "tester".to_owned())?;

    let adapter = BootMockAdapter1V1::new_test(chain.clone());
    adapter.deploy(V1.parse().unwrap(), MockInitMsg, DeployStrategy::Try)?;
    let adapter_addr = account.install_adapter(&adapter, None)?;
    let module_addr = account
        .manager
        .module_info(adapter_1::MOCK_ADAPTER_ID)?
        .unwrap()
        .address;
    assert_that!(adapter_addr).is_equal_to(module_addr);
    take_storage_snapshot!(chain, "account_install_adapter");
    Ok(())
}

#[test]
fn account_adapter_ownership() -> AResult {
    let sender = Addr::unchecked(OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;

    deployment
        .version_control
        .claim_namespace(TEST_ACCOUNT_ID, "tester".to_owned())?;

    let adapter = BootMockAdapter1V1::new_test(chain.clone());
    adapter.deploy(V1.parse().unwrap(), MockInitMsg, DeployStrategy::Try)?;
    account.install_adapter(&adapter, None)?;

    let proxy_addr = account.proxy.address()?;

    // Checking module requests

    // Can call either by account owner or manager
    adapter.call_as(&sender).execute(
        &mock::ExecuteMsg::Module(AdapterRequestMsg {
            proxy_address: Some(proxy_addr.to_string()),
            request: MockExecMsg {},
        }),
        None,
    )?;
    adapter.call_as(&account.manager.address()?).execute(
        &mock::ExecuteMsg::Module(AdapterRequestMsg {
            proxy_address: Some(proxy_addr.to_string()),
            request: MockExecMsg {},
        }),
        None,
    )?;

    // Not admin or manager
    let err: MockError = adapter
        .call_as(&Addr::unchecked("who"))
        .execute(
            &mock::ExecuteMsg::Module(AdapterRequestMsg {
                proxy_address: Some(proxy_addr.to_string()),
                request: MockExecMsg {},
            }),
            None,
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(
        err,
        MockError::Adapter(AdapterError::UnauthorizedAddressAdapterRequest {
            adapter: adapter_1::MOCK_ADAPTER_ID.to_owned(),
            sender: "who".to_owned()
        })
    );

    // Checking base requests

    // Can call either by account owner or manager
    adapter.call_as(&sender).execute(
        &mock::ExecuteMsg::Base(BaseExecuteMsg {
            proxy_address: Some(proxy_addr.to_string()),
            msg: AdapterBaseMsg::UpdateAuthorizedAddresses {
                to_add: vec!["123".to_owned()],
                to_remove: vec![],
            },
        }),
        None,
    )?;
    adapter.call_as(&account.manager.address()?).execute(
        &mock::ExecuteMsg::Base(BaseExecuteMsg {
            proxy_address: Some(proxy_addr.to_string()),
            msg: AdapterBaseMsg::UpdateAuthorizedAddresses {
                to_add: vec!["234".to_owned()],
                to_remove: vec![],
            },
        }),
        None,
    )?;

    // Not admin or manager
    let err: MockError = adapter
        .call_as(&Addr::unchecked("who"))
        .execute(
            &mock::ExecuteMsg::Base(BaseExecuteMsg {
                proxy_address: Some(proxy_addr.to_string()),
                msg: AdapterBaseMsg::UpdateAuthorizedAddresses {
                    to_add: vec!["345".to_owned()],
                    to_remove: vec![],
                },
            }),
            None,
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(
        err,
        MockError::Adapter(AdapterError::UnauthorizedAdapterRequest {
            adapter: adapter_1::MOCK_ADAPTER_ID.to_owned(),
            sender: "who".to_owned()
        })
    );

    Ok(())
}

#[test]
fn subaccount_adapter_ownership() -> AResult {
    let sender = Addr::unchecked(OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;

    deployment
        .version_control
        .claim_namespace(TEST_ACCOUNT_ID, "tester".to_owned())?;

    let adapter = BootMockAdapter1V1::new_test(chain.clone());
    adapter.deploy(V1.parse().unwrap(), MockInitMsg, DeployStrategy::Try)?;

    account.manager.create_sub_account(
        vec![ModuleInstallConfig::new(
            ModuleInfo::from_id_latest(adapter_1::MOCK_ADAPTER_ID).unwrap(),
            None,
        )],
        "My subaccount".to_string(),
        None,
        None,
        None,
        None,
        &[],
    )?;

    let sub_account = AbstractAccount::new(&deployment, AccountId::local(2));

    let module = sub_account
        .manager
        .module_info(adapter_1::MOCK_ADAPTER_ID)?
        .unwrap();
    adapter.set_address(&module.address);

    let proxy_addr = sub_account.proxy.address()?;

    // Checking module requests

    // Can call either by account owner or manager
    adapter.call_as(&sender).execute(
        &mock::ExecuteMsg::Module(AdapterRequestMsg {
            proxy_address: Some(proxy_addr.to_string()),
            request: MockExecMsg {},
        }),
        None,
    )?;
    adapter.call_as(&sub_account.manager.address()?).execute(
        &mock::ExecuteMsg::Module(AdapterRequestMsg {
            proxy_address: Some(proxy_addr.to_string()),
            request: MockExecMsg {},
        }),
        None,
    )?;

    // Not admin or manager
    let err: MockError = adapter
        .call_as(&Addr::unchecked("who"))
        .execute(
            &mock::ExecuteMsg::Module(AdapterRequestMsg {
                proxy_address: Some(proxy_addr.to_string()),
                request: MockExecMsg {},
            }),
            None,
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(
        err,
        MockError::Adapter(AdapterError::UnauthorizedAddressAdapterRequest {
            adapter: adapter_1::MOCK_ADAPTER_ID.to_owned(),
            sender: "who".to_owned()
        })
    );

    // Checking base requests

    // Can call either by account owner or manager
    adapter.call_as(&sender).execute(
        &mock::ExecuteMsg::Base(BaseExecuteMsg {
            proxy_address: Some(proxy_addr.to_string()),
            msg: AdapterBaseMsg::UpdateAuthorizedAddresses {
                to_add: vec!["123".to_owned()],
                to_remove: vec![],
            },
        }),
        None,
    )?;
    adapter.call_as(&sub_account.manager.address()?).execute(
        &mock::ExecuteMsg::Base(BaseExecuteMsg {
            proxy_address: Some(proxy_addr.to_string()),
            msg: AdapterBaseMsg::UpdateAuthorizedAddresses {
                to_add: vec!["234".to_owned()],
                to_remove: vec![],
            },
        }),
        None,
    )?;

    // Not admin or manager
    let err: MockError = adapter
        .call_as(&Addr::unchecked("who"))
        .execute(
            &mock::ExecuteMsg::Base(BaseExecuteMsg {
                proxy_address: Some(proxy_addr.to_string()),
                msg: AdapterBaseMsg::UpdateAuthorizedAddresses {
                    to_add: vec!["345".to_owned()],
                    to_remove: vec![],
                },
            }),
            None,
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(
        err,
        MockError::Adapter(AdapterError::UnauthorizedAdapterRequest {
            adapter: adapter_1::MOCK_ADAPTER_ID.to_owned(),
            sender: "who".to_owned()
        })
    );
    Ok(())
}

mod old_mock {
    use super::*;
    use crate::common::mock_modules;
    use abstract_adapter::gen_adapter_old_mock;
    use mock_modules::adapter_1::MOCK_ADAPTER_ID;

    gen_adapter_old_mock!(OldMockAdapter1V1, MOCK_ADAPTER_ID, "1.0.0", &[]);

    #[test]
    fn old_adapters_migratable() -> AResult {
        let sender = Addr::unchecked(OWNER);
        let chain = Mock::new(&sender);
        let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
        let account = create_default_account(&deployment.account_factory)?;

        deployment
            .version_control
            .claim_namespace(TEST_ACCOUNT_ID, "tester".to_owned())?;

        let old = OldMockAdapter1V1::new_test(chain.clone());
        old.deploy(V1.parse().unwrap(), MockInitMsg, DeployStrategy::Try)?;

        account.install_adapter(&old, None)?;

        let new = BootMockAdapter1V2::new_test(chain.clone());
        new.deploy(V2.parse().unwrap(), MockInitMsg, DeployStrategy::Try)?;

        account.manager.upgrade_module(MOCK_ADAPTER_ID, &Empty {})?;
        Ok(())
    }
}
