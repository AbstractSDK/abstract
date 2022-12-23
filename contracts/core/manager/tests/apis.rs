mod common;

use ::manager::contract::CONTRACT_VERSION;
use abstract_boot::*;
use abstract_os::{
    api::BaseQueryMsgFns,
    tendermint_staking::{TendermintStakingExecuteMsg, TendermintStakingExecuteMsgFns},
    *,
};
use abstract_os::{manager::ManagerModuleInfo, TENDERMINT_STAKING};
use common::{create_default_os, init_abstract_env, init_staking_api, AResult, TEST_COIN};

use boot_core::{
    prelude::{instantiate_default_mock_env, CallAs, ContractInstance},
    Mock, TxHandler,
};
use cosmwasm_std::{Addr, Coin, Decimal, Empty, Validator};
use cw_multi_test::StakingInfo;
use speculoos::prelude::*;

const VALIDATOR: &str = "testvaloper1";
fn install_api(manager: &Manager<Mock>, api: &str) -> AResult {
    manager
        .install_module(api, Some(&Empty {}))
        .map_err(Into::into)
}

/// TODO
/// - Non-existent version
/// - Non-existent name
/// - Specific version
/// - Migration
/// - Migration with traders
/// - Uninstall
/// - Add one
/// - Add duplicate
///

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

#[test]
fn add_one_api() -> AResult {
    let sender = Addr::unchecked(common::ROOT_USER);
    let (_state, chain) = instantiate_default_mock_env(&sender)?;
    let (mut deployment, mut core) = init_abstract_env(&chain)?;
    deployment.deploy(&mut core)?;
    let os = create_default_os(&chain, &deployment.os_factory)?;
    let staking_api = init_staking_api(&chain, &deployment)?;

    install_api(&os.manager, TENDERMINT_STAKING)?;

    let modules = os.manager.module_infos(None, None)?.module_infos;
    // assert proxy module
    assert_that(&modules.len()).is_equal_to(2);
    assert_that(&modules[1]).is_equal_to(&ManagerModuleInfo {
        address: staking_api.addr_str()?,
        id: TENDERMINT_STAKING.to_string(),
        version: cw2::ContractVersion {
            contract: TENDERMINT_STAKING.into(),
            version: CONTRACT_VERSION.into(),
        },
    });
    assert_that!(os.proxy.config()?).is_equal_to(proxy::ConfigResponse {
        modules: vec![os.manager.address()?.into_string(), staking_api.addr_str()?],
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
fn not_trader_exec() -> AResult {
    let sender = Addr::unchecked(common::ROOT_USER);
    let not_trader = Addr::unchecked("not_trader");
    let (_state, chain) = instantiate_default_mock_env(&sender)?;
    let (mut deployment, mut core) = init_abstract_env(&chain)?;
    deployment.deploy(&mut core)?;
    let os = create_default_os(&chain, &deployment.os_factory)?;
    let staking_api = init_staking_api(&chain, &deployment)?;
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
fn manager_api_exec() -> AResult {
    let sender = Addr::unchecked(common::ROOT_USER);
    let (_state, chain) = instantiate_default_mock_env(&sender)?;
    let (mut deployment, mut core) = init_abstract_env(&chain)?;
    setup_staking(&chain)?;

    deployment.deploy(&mut core)?;
    let os = create_default_os(&chain, &deployment.os_factory)?;
    let _staking_api = init_staking_api(&chain, &deployment)?;
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
