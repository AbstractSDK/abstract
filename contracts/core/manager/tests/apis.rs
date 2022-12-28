mod common;

use ::manager::contract::CONTRACT_VERSION;

use abstract_boot::*;
use abstract_os::objects::module::{ModuleInfo, ModuleVersion};
use abstract_os::{
    api::BaseQueryMsgFns,
    tendermint_staking::{TendermintStakingExecuteMsg, TendermintStakingExecuteMsgFns},
    *,
};
use abstract_os::{manager::ManagerModuleInfo, TENDERMINT_STAKING};
use boot_core::{
    prelude::{instantiate_default_mock_env, CallAs, ContractInstance},
    BootError, Mock, TxHandler,
};
use common::{create_default_os, init_abstract_env, init_staking_api, AResult, TEST_COIN};
use cosmwasm_std::{Addr, Coin, Decimal, Empty, Validator};
use cw_multi_test::StakingInfo;

use speculoos::prelude::*;

const VALIDATOR: &str = "testvaloper1";

fn install_api(manager: &Manager<Mock>, api: &str) -> AResult {
    manager
        .install_module(api, Some(&Empty {}))
        .map_err(Into::into)
}

pub(crate) fn uninstall_module(manager: &Manager<Mock>, api: &str) -> AResult {
    manager
        .remove_module(api.to_string())
        .map_err(Into::<BootError>::into)?;
    Ok(())
}

fn setup_staking(mock: &Mock) -> AResult {
    let block_info = mock.block_info()?;

    mock.app.borrow_mut().init_modules(|router, api, store| {
        router
            .staking
            .setup(
                store,
                StakingInfo {
                    bonded_denom: TEST_COIN.to_string(),
                    unbonding_time: 60,
                    apr: Decimal::percent(50),
                },
            )
            .unwrap();

        // add validator
        let valoper1 = Validator {
            address: VALIDATOR.to_string(),
            commission: Decimal::percent(10),
            max_commission: Decimal::percent(100),
            max_change_rate: Decimal::percent(1),
        };
        router
            .staking
            .add_validator(api, store, &block_info, valoper1)
            .unwrap();
    });

    Ok(())
}

/// TODO
/// - Migration
/// - Migration with traders
/// - Uninstall
/// - Dependency checks
#[test]
fn installing_one_api_should_succeed() -> AResult {
    let sender = Addr::unchecked(common::ROOT_USER);
    let (_state, chain) = instantiate_default_mock_env(&sender)?;
    let (mut deployment, mut core) = init_abstract_env(&chain)?;
    deployment.deploy(&mut core)?;
    let os = create_default_os(&chain, &deployment.os_factory)?;
    let staking_api = init_staking_api(&chain, &deployment, None)?;

    install_api(&os.manager, TENDERMINT_STAKING)?;

    let modules = os.expect_modules(vec![staking_api.address()?.to_string()])?;

    assert_that(&modules[1]).is_equal_to(&ManagerModuleInfo {
        address: staking_api.addr_str()?,
        id: TENDERMINT_STAKING.to_string(),
        version: cw2::ContractVersion {
            contract: TENDERMINT_STAKING.into(),
            version: CONTRACT_VERSION.into(),
        },
    });

    // Configuration is correct
    let api_config = staking_api.config()?;
    assert_that!(api_config).is_equal_to(api::ApiConfigResponse {
        ans_host_address: deployment.ans_host.address()?,
        dependencies: vec![],
        version_control_address: deployment.version_control.address()?,
    });

    // no traders registered
    let traders = staking_api.traders(os.proxy.addr_str()?)?;
    assert_that!(traders).is_equal_to(api::TradersResponse { traders: vec![] });

    Ok(())
}

#[test]
fn install_non_existent_apiname_should_fail() -> AResult {
    let sender = Addr::unchecked(common::ROOT_USER);
    let (_state, chain) = instantiate_default_mock_env(&sender)?;
    let (mut deployment, mut core) = init_abstract_env(&chain)?;
    deployment.deploy(&mut core)?;
    let os = create_default_os(&chain, &deployment.os_factory)?;

    let res = install_api(&os.manager, "lol:no_chance");

    assert_that!(res).is_err();
    // testtodo: check error
    Ok(())
}

#[test]
fn install_non_existent_version_should_fail() -> AResult {
    let sender = Addr::unchecked(common::ROOT_USER);
    let (_state, chain) = instantiate_default_mock_env(&sender)?;
    let (mut deployment, mut core) = init_abstract_env(&chain)?;
    deployment.deploy(&mut core)?;
    let os = create_default_os(&chain, &deployment.os_factory)?;
    init_staking_api(&chain, &deployment, None)?;

    let res = os.manager.install_module_version::<Empty>(
        TENDERMINT_STAKING,
        ModuleVersion::Version("1.2.3".to_string()),
        None,
    );

    // testtodo: check error
    assert_that!(res).is_err();

    Ok(())
}

#[test]
fn installation_of_duplicate_api_should_fail() -> AResult {
    let sender = Addr::unchecked(common::ROOT_USER);
    let (_state, chain) = instantiate_default_mock_env(&sender)?;
    let (mut deployment, mut core) = init_abstract_env(&chain)?;
    deployment.deploy(&mut core)?;
    let os = create_default_os(&chain, &deployment.os_factory)?;
    let staking_api = init_staking_api(&chain, &deployment, None)?;

    install_api(&os.manager, TENDERMINT_STAKING)?;

    let modules = os.expect_modules(vec![staking_api.address()?.to_string()])?;

    // assert proxy module
    // check staking api
    assert_that(&modules[1]).is_equal_to(&ManagerModuleInfo {
        address: staking_api.addr_str()?,
        id: TENDERMINT_STAKING.to_string(),
        version: cw2::ContractVersion {
            contract: TENDERMINT_STAKING.into(),
            version: CONTRACT_VERSION.into(),
        },
    });

    // install again
    let second_install_res = install_api(&os.manager, TENDERMINT_STAKING);
    assert_that!(second_install_res)
        .is_err()
        .matches(|e| e.to_string().contains("tendermint_staking"));

    os.expect_modules(vec![staking_api.address()?.to_string()])?;

    Ok(())
}

#[test]
fn reinstalling_api_should_be_allowed() -> AResult {
    let sender = Addr::unchecked(common::ROOT_USER);
    let (_state, chain) = instantiate_default_mock_env(&sender)?;
    let (mut deployment, mut core) = init_abstract_env(&chain)?;
    deployment.deploy(&mut core)?;
    let os = create_default_os(&chain, &deployment.os_factory)?;
    let staking_api = init_staking_api(&chain, &deployment, None)?;

    install_api(&os.manager, TENDERMINT_STAKING)?;

    let modules = os.expect_modules(vec![staking_api.address()?.to_string()])?;

    // check staking api
    assert_that(&modules[1]).is_equal_to(&ManagerModuleInfo {
        address: staking_api.addr_str()?,
        id: TENDERMINT_STAKING.to_string(),
        version: cw2::ContractVersion {
            contract: TENDERMINT_STAKING.into(),
            version: CONTRACT_VERSION.into(),
        },
    });

    // uninstall
    uninstall_module(&os.manager, TENDERMINT_STAKING)?;

    // None expected
    os.expect_modules(vec![])?;

    // reinstall
    install_api(&os.manager, TENDERMINT_STAKING)?;

    os.expect_modules(vec![staking_api.address()?.to_string()])?;

    Ok(())
}

/// Reinstalling the API should install the latest version
#[test]
fn reinstalling_new_version_should_install_latest() -> AResult {
    let sender = Addr::unchecked(common::ROOT_USER);
    let (_state, chain) = instantiate_default_mock_env(&sender)?;
    let (mut deployment, mut core) = init_abstract_env(&chain)?;
    deployment.deploy(&mut core)?;
    let os = create_default_os(&chain, &deployment.os_factory)?;
    let staking_api = init_staking_api(&chain, &deployment, None)?;

    install_api(&os.manager, TENDERMINT_STAKING)?;

    let modules = os.expect_modules(vec![staking_api.address()?.to_string()])?;

    // check staking api
    assert_that(&modules[1]).is_equal_to(&ManagerModuleInfo {
        address: staking_api.addr_str()?,
        id: TENDERMINT_STAKING.to_string(),
        version: cw2::ContractVersion {
            contract: TENDERMINT_STAKING.into(),
            version: CONTRACT_VERSION.into(),
        },
    });

    // uninstall tendermint staking
    uninstall_module(&os.manager, TENDERMINT_STAKING)?;

    os.expect_modules(vec![])?;

    /// Register the new version
    let new_version_num = "100.0.0";

    // We init the staking api with a new version to ensure that we get a new address
    let new_staking_api = init_staking_api(&chain, &deployment, Some(new_version_num.to_string()))?;

    // check that the latest staking version is the new one
    let latest_staking = deployment
        .version_control
        .module(ModuleInfo::from_id_latest(TENDERMINT_STAKING)?)?
        .module;
    assert_that!(latest_staking.info.version)
        .is_equal_to(ModuleVersion::Version(new_version_num.to_string()));

    // reinstall
    install_api(&os.manager, TENDERMINT_STAKING)?;

    let modules = os.expect_modules(vec![new_staking_api.address()?.to_string()])?;

    // assert_that!(modules[1]).is_equal_to(&ManagerModuleInfo {
    //     address: staking_api.addr_str()?,
    //     id: TENDERMINT_STAKING.to_string(),
    //     version: cw2::ContractVersion {
    //         contract: TENDERMINT_STAKING.into(),
    //         version: new_version_num.to_string(),
    //     },
    // });
    // we should check that the address registered in the manager is the new one as opposed to the version, which is incorrectly returned as the ContractVerison (right now)
    // TODO uncomment when the manager actually queries version control (if desired)

    assert_that!(modules[1].address)
        .is_equal_to(new_staking_api.as_instance().address()?.to_string());

    Ok(())
}

// struct TestModule = AppContract

#[test]
fn not_trader_exec() -> AResult {
    let sender = Addr::unchecked(common::ROOT_USER);
    let not_trader = Addr::unchecked("not_trader");
    let (_state, chain) = instantiate_default_mock_env(&sender)?;
    let (mut deployment, mut core) = init_abstract_env(&chain)?;
    deployment.deploy(&mut core)?;
    let os = create_default_os(&chain, &deployment.os_factory)?;
    let staking_api = init_staking_api(&chain, &deployment, None)?;
    install_api(&os.manager, TENDERMINT_STAKING)?;
    // non-trader cannot execute
    let res = staking_api
        .call_as(&not_trader)
        .delegate(100u128.into(), VALIDATOR.into())
        .unwrap_err();
    assert_that!(res.root().to_string()).contains("Sender of request is not a Manager or Trader");
    // neither can the ROOT directly
    let res = staking_api
        .delegate(100u128.into(), VALIDATOR.into())
        .unwrap_err();
    assert_that!(&res.root().to_string()).contains("Sender of request is not a Manager or Trader");
    Ok(())
}

#[test]
fn manager_api_exec_staking_delegation() -> AResult {
    let sender = Addr::unchecked(common::ROOT_USER);
    let (_state, chain) = instantiate_default_mock_env(&sender)?;
    let (mut deployment, mut core) = init_abstract_env(&chain)?;
    setup_staking(&chain)?;

    deployment.deploy(&mut core)?;
    let os = create_default_os(&chain, &deployment.os_factory)?;
    let _staking_api = init_staking_api(&chain, &deployment, None)?;
    install_api(&os.manager, TENDERMINT_STAKING)?;

    chain.init_balance(&os.proxy.address()?, vec![Coin::new(100_000, TEST_COIN)])?;

    os.manager.execute_on_module(
        TENDERMINT_STAKING,
        Into::<tendermint_staking::ExecuteMsg>::into(TendermintStakingExecuteMsg::Delegate {
            validator: VALIDATOR.into(),
            amount: 100u128.into(),
        }),
    )?;

    Ok(())
}

#[test]
fn installing_specific_version_should_install_expected() -> AResult {
    let sender = Addr::unchecked(common::ROOT_USER);
    let (_state, chain) = instantiate_default_mock_env(&sender)?;
    let (mut deployment, mut core) = init_abstract_env(&chain)?;
    deployment.deploy(&mut core)?;
    let os = create_default_os(&chain, &deployment.os_factory)?;
    let _staking_api_one = init_staking_api(&chain, &deployment, Some("1.2.3".to_string()))?;
    let expected_version = "2.3.4".to_string();
    let expected_staking_api =
        init_staking_api(&chain, &deployment, Some(expected_version.clone()))?;
    let expected_staking_api_addr = expected_staking_api.address()?.to_string();

    let _staking_api_three = init_staking_api(&chain, &deployment, Some("3.4.5".to_string()))?;

    // install specific version
    os.manager.install_module_version::<Empty>(
        TENDERMINT_STAKING,
        ModuleVersion::Version(expected_version),
        None,
    )?;

    let modules = os.expect_modules(vec![expected_staking_api_addr])?;
    let installed_module: ManagerModuleInfo = modules[1].clone();
    assert_that!(installed_module.id).is_equal_to(TENDERMINT_STAKING.to_string());

    Ok(())
}

// #[test]
// fn uninstalling_api_with_dependent_module_should_fail() -> AResult {
//     // TODO
//     Ok(())
// }
