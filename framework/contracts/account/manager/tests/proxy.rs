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
use common::{create_default_account, init_mock_adapter, install_adapter, AResult, TEST_COIN};
use cosmwasm_std::{wasm_execute, Addr, Coin, CosmosMsg};
use cw_orch::deploy::Deploy;
use cw_orch::prelude::*;
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
fn install_pre_wasmed_modules() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = AbstractAccount::new(&deployment, Some(0));
    let cw20_contract = Box::new(ContractWrapper::new(
        cw20_base::contract::execute,
        cw20_base::contract::instantiate,
        cw20_base::contract::query,
    ));
    let cw20_id = chain.app.borrow_mut().store_code(cw20_contract);

    // let account_base = deployment.version_control.get_account(0)?;
    // account.manager.set_address(&account_base.manager);
    // account.proxy.set_address(&account_base.proxy);

    deployment.version_control.propose_modules(vec![(
        ModuleInfo {
            namespace: Namespace::new("abstract")?,
            name: "cw20".to_owned(),
            version: ModuleVersion::Version("1.1.0".to_owned()),
        },
        ModuleReference::Standalone(cw20_id),
    )])?;

    account.install_module(
        "abstract:cw20",
        &cw20_base::msg::InstantiateMsg {
            name: "abstr".to_owned(),
            symbol: "abs".to_owned(),
            decimals: 6,
            initial_balances: vec![],
            mint: None,
            marketing: None,
        },
        None,
    )?;
    Ok(())
}
