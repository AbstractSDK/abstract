mod common;
use abstract_adapter::mock::MockExecMsg;
use abstract_core::adapter::AdapterRequestMsg;
use abstract_core::objects::module::{ModuleInfo, ModuleVersion};
use abstract_core::objects::module_reference::ModuleReference;
use abstract_core::objects::namespace::Namespace;
use abstract_core::{manager::ManagerModuleInfo, PROXY};
use abstract_interface::*;
use abstract_manager::contract::CONTRACT_VERSION;
use abstract_testing::prelude::{TEST_ACCOUNT_ID, TEST_MODULE_ID};
use common::{
    create_default_account, init_mock_adapter, install_adapter, mock_modules, AResult, TEST_COIN,
};
use cosmwasm_std::{wasm_execute, Addr, Coin, CosmosMsg};
use cw_orch::deploy::Deploy;
use cw_orch::prelude::*;
use module_factory::error::ModuleFactoryError;
use speculoos::prelude::*;

#[test]
fn instantiate() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain, sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;

    let modules = account.manager.module_infos(None, None)?.module_infos;

    // assert proxy module
    assert_that!(&modules).has_length(1);
    assert_that(&modules[0]).is_equal_to(&ManagerModuleInfo {
        address: account.proxy.address()?,
        id: PROXY.to_string(),
        version: cw2::ContractVersion {
            contract: PROXY.into(),
            version: CONTRACT_VERSION.into(),
        },
    });

    // assert manager config
    assert_that!(account.manager.config()?).is_equal_to(abstract_core::manager::ConfigResponse {
        version_control_address: deployment.version_control.address()?,
        module_factory_address: deployment.module_factory.address()?,
        account_id: TEST_ACCOUNT_ID.into(),
        is_suspended: false,
    });
    Ok(())
}

#[test]
fn exec_through_manager() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;

    // mint coins to proxy address
    chain.set_balance(
        &account.proxy.address()?,
        vec![Coin::new(100_000, TEST_COIN)],
    )?;

    // burn coins from proxy
    let proxy_balance = chain
        .app
        .borrow()
        .wrap()
        .query_all_balances(account.proxy.address()?)?;
    assert_that!(proxy_balance).is_equal_to(vec![Coin::new(100_000, TEST_COIN)]);

    let burn_amount: Vec<Coin> = vec![Coin::new(10_000, TEST_COIN)];

    account.manager.exec_on_module(
        cosmwasm_std::to_binary(&abstract_core::proxy::ExecuteMsg::ModuleAction {
            msgs: vec![CosmosMsg::Bank(cosmwasm_std::BankMsg::Burn {
                amount: burn_amount,
            })],
        })?,
        PROXY.to_string(),
    )?;

    let proxy_balance = chain
        .app
        .borrow()
        .wrap()
        .query_all_balances(account.proxy.address()?)?;
    assert_that!(proxy_balance).is_equal_to(vec![Coin::new(100_000 - 10_000, TEST_COIN)]);

    Ok(())
}

#[test]
fn default_without_response_data() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;
    let _staking_adapter_one = init_mock_adapter(chain.clone(), &deployment, None)?;

    install_adapter(&account.manager, TEST_MODULE_ID)?;

    chain.set_balance(
        &account.proxy.address()?,
        vec![Coin::new(100_000, TEST_COIN)],
    )?;

    let resp = account.manager.execute_on_module(
        TEST_MODULE_ID,
        Into::<abstract_core::adapter::ExecuteMsg<MockExecMsg>>::into(MockExecMsg),
    )?;
    assert_that!(resp.data).is_none();

    Ok(())
}

#[test]
fn with_response_data() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;
    let staking_adapter = init_mock_adapter(chain.clone(), &deployment, None)?;

    install_adapter(&account.manager, TEST_MODULE_ID)?;

    staking_adapter
        .call_as(&account.manager.address()?)
        .execute(
            &abstract_core::adapter::ExecuteMsg::<MockExecMsg, Empty>::Base(
                abstract_core::adapter::BaseExecuteMsg::UpdateAuthorizedAddresses {
                    to_add: vec![account.proxy.addr_str()?],
                    to_remove: vec![],
                },
            ),
            None,
        )?;

    chain.set_balance(
        &account.proxy.address()?,
        vec![Coin::new(100_000, TEST_COIN)],
    )?;

    let adapter_addr = account
        .manager
        .module_info(TEST_MODULE_ID)?
        .expect("test module installed");
    // proxy should be final executor because of the reply
    let resp = account.manager.exec_on_module(
        cosmwasm_std::to_binary(&abstract_core::proxy::ExecuteMsg::ModuleActionWithData {
            // execute a message on the adapter, which sets some data in its response
            msg: wasm_execute(
                adapter_addr.address,
                &abstract_core::adapter::ExecuteMsg::<MockExecMsg, Empty>::Module(
                    AdapterRequestMsg {
                        proxy_address: Some(account.proxy.addr_str()?),
                        request: MockExecMsg,
                    },
                ),
                vec![],
            )?
            .into(),
        })?,
        PROXY.to_string(),
    )?;

    let response_data_attr_present = resp.event_attr_value("wasm-abstract", "response_data")?;
    assert_that!(response_data_attr_present).is_equal_to("true".to_string());

    Ok(())
}

#[test]
fn install_standalone_modules() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = AbstractAccount::new(&deployment, Some(0));

    let standalone1_contract = Box::new(ContractWrapper::new(
        mock_modules::standalone_cw2::mock_execute,
        mock_modules::standalone_cw2::mock_instantiate,
        mock_modules::standalone_cw2::mock_query,
    ));
    let standalone1_id = chain.app.borrow_mut().store_code(standalone1_contract);

    let standalone2_contract = Box::new(ContractWrapper::new(
        mock_modules::standalone_no_cw2::mock_execute,
        mock_modules::standalone_no_cw2::mock_instantiate,
        mock_modules::standalone_no_cw2::mock_query,
    ));
    let standalone2_id = chain.app.borrow_mut().store_code(standalone2_contract);

    // install first standalone
    deployment.version_control.propose_modules(vec![(
        ModuleInfo {
            namespace: Namespace::new("abstract")?,
            name: "standalone1".to_owned(),
            version: ModuleVersion::Version(mock_modules::V1.to_owned()),
        },
        ModuleReference::Standalone(standalone1_id),
    )])?;

    account.install_module(
        "abstract:standalone1",
        &mock_modules::standalone_cw2::MockMsg,
        None,
    )?;

    // install second standalone
    deployment.version_control.propose_modules(vec![(
        ModuleInfo {
            namespace: Namespace::new("abstract")?,
            name: "standalone2".to_owned(),
            version: ModuleVersion::Version(mock_modules::V1.to_owned()),
        },
        ModuleReference::Standalone(standalone2_id),
    )])?;

    account.install_module(
        "abstract:standalone2",
        &mock_modules::standalone_no_cw2::MockMsg,
        None,
    )?;
    Ok(())
}

#[test]
fn install_standalone_versions_not_met() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = AbstractAccount::new(&deployment, Some(0));

    let standalone1_contract = Box::new(ContractWrapper::new(
        mock_modules::standalone_cw2::mock_execute,
        mock_modules::standalone_cw2::mock_instantiate,
        mock_modules::standalone_cw2::mock_query,
    ));
    let standalone1_id = chain.app.borrow_mut().store_code(standalone1_contract);

    // install first standalone
    deployment.version_control.propose_modules(vec![(
        ModuleInfo {
            namespace: Namespace::new("abstract")?,
            name: "standalone1".to_owned(),
            version: ModuleVersion::Version(mock_modules::V2.to_owned()),
        },
        ModuleReference::Standalone(standalone1_id),
    )])?;

    let err = account
        .install_module(
            "abstract:standalone1",
            &mock_modules::standalone_cw2::MockMsg,
            None,
        )
        .unwrap_err();

    if let AbstractInterfaceError::Orch(err) = err {
        let err: ModuleFactoryError = err.downcast()?;
        assert_eq!(
            err,
            ModuleFactoryError::Abstract(abstract_core::AbstractError::UnequalModuleData {
                cw2: mock_modules::V1.to_owned(),
                module: mock_modules::V2.to_owned(),
            })
        );
    } else {
        panic!("wrong error type")
    };

    Ok(())
}
