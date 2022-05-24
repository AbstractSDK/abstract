use cosmwasm_std::{Addr, Coin, Decimal, Uint128};
use cw20::Cw20Contract;

use cw_asset::AssetInfoUnchecked;
use terra_multi_test::{App, ContractWrapper};

use crate::dapp_base::common::TEST_CREATOR;
use crate::tests::integration_tests::common_integration::{
    init_contracts, mint_some_whale, mock_app,
};
use abstract_os::core::proxy::msg as TreasuryMsg;
use abstract_os::modules::apis::terraswap::ExecuteMsg;
use abstract_os::native::memory::msg as MemoryMsg;
use terra_multi_test::Executor;
use terraswap::pair::PoolResponse;

use abstract_os::modules::dapp_base::msg::{BaseExecuteMsg, BaseInstantiateMsg as InstantiateMsg};

use super::common_integration::{whitelist_dapp, BaseContracts};
const MILLION: u64 = 1_000_000u64;

fn init_terraswap_dapp(app: &mut App, owner: Addr, base_contracts: &BaseContracts) -> Addr {
    // Upload Terraswap DApp Contract
    let tswap_dapp_contract = Box::new(ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    ));

    let tswap_dapp_code_id = app.store_code(tswap_dapp_contract);

    let tswap_dapp_instantiate_msg = InstantiateMsg {
        memory_addr: base_contracts.memory.to_string(),
    };

    // Init contract
    let tswap_dapp_instance = app
        .instantiate_contract(
            tswap_dapp_code_id,
            owner.clone(),
            &tswap_dapp_instantiate_msg,
            &[],
            "Tswap_dapp",
            None,
        )
        .unwrap();

    whitelist_dapp(app, &owner, &base_contracts.proxy, &tswap_dapp_instance);

    // Make admin trader
    app.execute_contract(
        owner.clone(),
        tswap_dapp_instance.clone(),
        &ExecuteMsg::Base(BaseExecuteMsg::UpdateTraders {
            to_add: Some(vec![owner.to_string()]),
            to_remove: None,
        }),
        &[],
    )
    .unwrap();

    // Make set proxy
    app.execute_contract(
        owner.clone(),
        tswap_dapp_instance.clone(),
        &ExecuteMsg::Base(BaseExecuteMsg::UpdateConfig {
            proxy_address: Some(base_contracts.proxy.to_string()),
        }),
        &[],
    )
    .unwrap();

    tswap_dapp_instance
}

#[test]
fn proper_initialization() {
    let mut app = mock_app();
    let sender = Addr::unchecked(TEST_CREATOR);
    let base_contracts = init_contracts(&mut app);
    let tswap_dapp = init_terraswap_dapp(&mut app, sender.clone(), &base_contracts);

    let resp: TreasuryMsg::ConfigResponse = app
        .wrap()
        .query_wasm_smart(&base_contracts.proxy, &TreasuryMsg::QueryMsg::Config {})
        .unwrap();

    // Check config, tswap dapp is added
    assert_eq!(1, resp.dapps.len());

    // Add whale and whale_ust token to the memory assets
    // Is tested on unit-test level
    app.execute_contract(
        sender.clone(),
        base_contracts.memory.clone(),
        &MemoryMsg::ExecuteMsg::UpdateAssetAddresses {
            to_add: vec![
                (
                    "whale".to_string(),
                    AssetInfoUnchecked::Cw20(base_contracts.whale.to_string()),
                ),
                (
                    "whale_ust".to_string(),
                    AssetInfoUnchecked::Cw20(base_contracts.whale_ust.to_string()),
                ),
                (
                    "ust".to_string(),
                    AssetInfoUnchecked::Native("uusd".to_string()),
                ),
            ],
            to_remove: vec![],
        },
        &[],
    )
    .unwrap();

    // Check Memory
    let resp: MemoryMsg::AssetQueryResponse = app
        .wrap()
        .query_wasm_smart(
            &base_contracts.memory,
            &MemoryMsg::QueryMsg::QueryAssets {
                names: vec![
                    "whale".to_string(),
                    "whale_ust".to_string(),
                    "ust".to_string(),
                ],
            },
        )
        .unwrap();

    // Detailed check handled in unit-tests
    assert_eq!("ust".to_string(), resp.assets[0].0);
    assert_eq!("whale".to_string(), resp.assets[1].0);
    assert_eq!("whale_ust".to_string(), resp.assets[2].0);

    // Add whale_ust pair to the memory contracts
    // Is tested on unit-test level
    app.execute_contract(
        sender.clone(),
        base_contracts.memory.clone(),
        &MemoryMsg::ExecuteMsg::UpdateContractAddresses {
            to_add: vec![(
                "whale_ust_pair".to_string(),
                base_contracts.whale_ust_pair.to_string(),
            )],
            to_remove: vec![],
        },
        &[],
    )
    .unwrap();

    // Check Memory
    let resp: MemoryMsg::ContractQueryResponse = app
        .wrap()
        .query_wasm_smart(
            &base_contracts.memory,
            &MemoryMsg::QueryMsg::QueryContracts {
                names: vec!["whale_ust_pair".to_string()],
            },
        )
        .unwrap();

    // Detailed check handled in unit-tests
    assert_eq!("whale_ust_pair".to_string(), resp.contracts[0].0);

    // give proxy some uusd
    app.init_bank_balance(
        &base_contracts.proxy,
        vec![Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(100u64 * MILLION),
        }],
    )
    .unwrap();

    // give proxy some whale
    mint_some_whale(
        &mut app,
        sender.clone(),
        base_contracts.whale,
        Uint128::from(100u64 * MILLION),
        base_contracts.proxy.to_string(),
    );

    // Add liquidity to pair from proxy, through terraswap-dapp
    app.execute_contract(
        sender.clone(),
        tswap_dapp.clone(),
        &ExecuteMsg::DetailedProvideLiquidity {
            pool_id: "whale_ust_pair".to_string(),
            assets: vec![
                ("ust".into(), Uint128::from(1u64 * MILLION)),
                (("whale".into(), Uint128::from(1u64 * MILLION))),
            ],
            slippage_tolerance: None,
        },
        &[],
    )
    .unwrap();

    //
    let pool_res: PoolResponse = app
        .wrap()
        .query_wasm_smart(
            base_contracts.whale_ust_pair.clone(),
            &terraswap::pair::QueryMsg::Pool {},
        )
        .unwrap();

    let lp = Cw20Contract(base_contracts.whale_ust.clone());

    // Get proxy lp token balance
    let proxy_bal = lp.balance(&app, base_contracts.proxy.clone()).unwrap();

    // 1 WHALE and UST in pool
    assert_eq!(Uint128::from(1u64 * MILLION), pool_res.assets[0].amount);
    assert_eq!(Uint128::from(1u64 * MILLION), pool_res.assets[1].amount);
    // All LP tokens owned by proxy
    assert_eq!(proxy_bal, pool_res.total_share);

    // Failed swap UST for WHALE due to max spread
    app.execute_contract(
        sender.clone(),
        tswap_dapp.clone(),
        &ExecuteMsg::SwapAsset {
            pool_id: "whale_ust_pair".to_string(),
            offer_id: "ust".into(),
            amount: Uint128::from(100u64),
            max_spread: Some(Decimal::zero()),
            belief_price: Some(Decimal::one()),
        },
        &[],
    )
    .unwrap_err();

    // Successful swap UST for WHALE
    app.execute_contract(
        sender.clone(),
        tswap_dapp.clone(),
        &ExecuteMsg::SwapAsset {
            pool_id: "whale_ust_pair".to_string(),
            offer_id: "ust".into(),
            amount: Uint128::from(100u64),
            max_spread: None,
            belief_price: None,
        },
        &[],
    )
    .unwrap();
    //
    let pool_res: PoolResponse = app
        .wrap()
        .query_wasm_smart(
            base_contracts.whale_ust_pair.clone(),
            &terraswap::pair::QueryMsg::Pool {},
        )
        .unwrap();

    // 1 WHALE and UST in pool
    assert_eq!(
        Uint128::from(1u64 * MILLION + 100u64),
        pool_res.assets[0].amount
    );
    assert_eq!(
        Uint128::from(1u64 * MILLION - 99u64),
        pool_res.assets[1].amount
    );

    // try withdrawing zero, this will fail with an error
    app.execute_contract(
        sender.clone(),
        tswap_dapp.clone(),
        &ExecuteMsg::WithdrawLiquidity {
            lp_token_id: "whale_ust".to_string(),
            amount: Uint128::zero(),
        },
        &[],
    )
    .unwrap_err();

    // Withdraw half of the liquidity from the pool
    app.execute_contract(
        sender.clone(),
        tswap_dapp.clone(),
        &ExecuteMsg::WithdrawLiquidity {
            lp_token_id: "whale_ust".to_string(),
            amount: Uint128::from(MILLION / 2u64),
        },
        &[],
    )
    .unwrap();

    //
    let pool_res: PoolResponse = app
        .wrap()
        .query_wasm_smart(
            base_contracts.whale_ust_pair.clone(),
            &terraswap::pair::QueryMsg::Pool {},
        )
        .unwrap();

    // Get proxy lp token balance
    let proxy_bal = lp.balance(&app, base_contracts.proxy.clone()).unwrap();

    // 1 WHALE and UST in pool
    assert_eq!(
        Uint128::from((1u64 * MILLION + 100u64) / 2u64),
        pool_res.assets[0].amount
    );
    // small rounding error
    assert_eq!(
        Uint128::from((1u64 * MILLION - 98u64) / 2u64),
        pool_res.assets[1].amount
    );
    // Half of the LP tokens left
    assert_eq!(proxy_bal, Uint128::from(MILLION / 2u64));

    let price = Decimal::from_ratio(pool_res.assets[0].amount, pool_res.assets[1].amount);

    // Provide undetailed liquidity
    // Should add token at same price as pool
    app.execute_contract(
        sender.clone(),
        tswap_dapp.clone(),
        &ExecuteMsg::ProvideLiquidity {
            pool_id: "whale_ust_pair".to_string(),
            main_asset_id: "whale".to_string(),
            amount: Uint128::from(MILLION),
        },
        &[],
    )
    .unwrap();

    // Provide zero liquidity
    // this should fail with an error
    app.execute_contract(
        sender.clone(),
        tswap_dapp.clone(),
        &ExecuteMsg::ProvideLiquidity {
            pool_id: "whale_ust_pair".to_string(),
            main_asset_id: "whale".to_string(),
            amount: Uint128::zero(),
        },
        &[],
    )
    .unwrap_err();

    //
    let pool_res_after: PoolResponse = app
        .wrap()
        .query_wasm_smart(
            base_contracts.whale_ust_pair.clone(),
            &terraswap::pair::QueryMsg::Pool {},
        )
        .unwrap();

    // 1 WHALE added to pool
    assert_eq!(
        Uint128::from(MILLION) * price,
        pool_res_after.assets[0].amount - pool_res.assets[0].amount
    );
    // 1.00.. UST added to pool, equating to same price as before
    assert_eq!(
        Uint128::from(MILLION),
        pool_res_after.assets[1].amount - pool_res.assets[1].amount
    );
}
