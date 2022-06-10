use std::collections::HashMap;

use cosmwasm_std::Addr;

use abstract_os::native::version_control::state::Core;
use abstract_os::registery::PROXY;

use cw_multi_test::App;

use crate::tests::common::TEST_CREATOR;
use crate::tests::integration_tests::common_integration::mock_app;

use crate::tests::integration_tests::upload::upload_contracts;

use abstract_os::core::*;

use abstract_os::native::*;
use cw_multi_test::Executor;

use super::common_integration::{NativeContracts, OsInstance};
const MILLION: u64 = 1_000_000u64;

fn init_os(app: &mut App, sender: Addr, native_contracts: &NativeContracts) -> OsInstance {
    let _resp = app
        .execute_contract(
            sender.clone(),
            native_contracts.os_factory.clone(),
            &abstract_os::native::os_factory::msg::ExecuteMsg::CreateOs {
                governance: abstract_os::governance::gov_type::GovernanceDetails::Monarchy {
                    monarch: sender.into_string(),
                },
            },
            &[],
        )
        .unwrap();

    // Check OS
    let resp: Core = app
        .wrap()
        .query_wasm_smart(
            &native_contracts.version_control,
            &version_control::msg::QueryMsg::QueryOsAddress { os_id: 0u32 },
        )
        .unwrap();

    let manager_addr = Addr::unchecked(resp.manager);
    let resp: manager::msg::ModuleQueryResponse = app
        .wrap()
        .query_wasm_smart(
            &manager_addr,
            &manager::msg::QueryMsg::QueryModules {
                names: vec![PROXY.to_string()],
            },
        )
        .unwrap();
    let proxy_addr = Addr::unchecked(resp.modules.last().unwrap().clone().1);
    OsInstance {
        manager: manager_addr,
        proxy: proxy_addr,
        modules: HashMap::new(),
    }
}

#[test]
fn proper_initialization() {
    let mut app = mock_app();
    let sender = Addr::unchecked(TEST_CREATOR);
    let (_code_ids, native_contracts) = upload_contracts(&mut app);

    init_os(&mut app, sender, &native_contracts);

    // // Add whale and whale_ust token to the memory assets
    // // Is tested on unit-test level
    // app.execute_contract(
    //     sender.clone(),
    //     base_contracts.memory.clone(),
    //     &MemoryMsg::ExecuteMsg::UpdateAssetAddresses {
    //         to_add: vec![
    //             (
    //                 "whale".to_string(),
    //                 AssetInfoUnchecked::Cw20(base_contracts.whale.to_string()),
    //             ),
    //             (
    //                 "whale_ust".to_string(),
    //                 AssetInfoUnchecked::Cw20(base_contracts.whale_ust.to_string()),
    //             ),
    //             (
    //                 "ust".to_string(),
    //                 AssetInfoUnchecked::Native("uusd".to_string()),
    //             ),
    //         ],
    //         to_remove: vec![],
    //     },
    //     &[],
    // )
    // .unwrap();

    // // Check Memory
    // let resp: MemoryMsg::AssetQueryResponse = app
    //     .wrap()
    //     .query_wasm_smart(
    //         &base_contracts.memory,
    //         &MemoryMsg::QueryMsg::QueryAssets {
    //             names: vec![
    //                 "whale".to_string(),
    //                 "whale_ust".to_string(),
    //                 "ust".to_string(),
    //             ],
    //         },
    //     )
    //     .unwrap();

    // // Detailed check handled in unit-tests
    // assert_eq!("ust".to_string(), resp.assets[0].0);
    // assert_eq!("whale".to_string(), resp.assets[1].0);
    // assert_eq!("whale_ust".to_string(), resp.assets[2].0);

    // // Add whale_ust pair to the memory contracts
    // // Is tested on unit-test level
    // app.execute_contract(
    //     sender.clone(),
    //     base_contracts.memory.clone(),
    //     &MemoryMsg::ExecuteMsg::UpdateContractAddresses {
    //         to_add: vec![(
    //             "whale_ust_pair".to_string(),
    //             base_contracts.whale_ust_pair.to_string(),
    //         )],
    //         to_remove: vec![],
    //     },
    //     &[],
    // )
    // .unwrap();

    // // Check Memory
    // let resp: MemoryMsg::ContractQueryResponse = app
    //     .wrap()
    //     .query_wasm_smart(
    //         &base_contracts.memory,
    //         &MemoryMsg::QueryMsg::QueryContracts {
    //             names: vec!["whale_ust_pair".to_string()],
    //         },
    //     )
    //     .unwrap();

    // // Detailed check handled in unit-tests
    // assert_eq!("whale_ust_pair".to_string(), resp.contracts[0].0);

    // give proxy some uusd
    //     app.init_bank_balance(
    //         &base_contracts.proxy,
    //         vec![Coin {
    //             denom: "uusd".to_string(),
    //             amount: Uint128::from(100u64 * MILLION),
    //         }],
    //     )
    //     .unwrap();

    //     // give proxy some whale
    //     mint_some_whale(
    //         &mut app,
    //         sender.clone(),
    //         base_contracts.whale,
    //         Uint128::from(100u64 * MILLION),
    //         base_contracts.proxy.to_string(),
    //     );

    //     // Add liquidity to pair from proxy, through terraswap-dapp
    //     app.execute_contract(
    //         sender.clone(),
    //         tswap_dapp.clone(),
    //         &ExecuteMsg::DetailedProvideLiquidity {
    //             pool_id: "whale_ust_pair".to_string(),
    //             assets: vec![
    //                 ("ust".into(), Uint128::from(1u64 * MILLION)),
    //                 (("whale".into(), Uint128::from(1u64 * MILLION))),
    //             ],
    //             slippage_tolerance: None,
    //         },
    //         &[],
    //     )
    //     .unwrap();

    //     //
    //     let pool_res: PoolResponse = app
    //         .wrap()
    //         .query_wasm_smart(
    //             base_contracts.whale_ust_pair.clone(),
    //             &terraswap::pair::QueryMsg::Pool {},
    //         )
    //         .unwrap();

    //     let lp = Cw20Contract(base_contracts.whale_ust.clone());

    //     // Get proxy lp token balance
    //     let proxy_bal = lp.balance(&app, base_contracts.proxy.clone()).unwrap();

    //     // 1 WHALE and UST in pool
    //     assert_eq!(Uint128::from(1u64 * MILLION), pool_res.assets[0].amount);
    //     assert_eq!(Uint128::from(1u64 * MILLION), pool_res.assets[1].amount);
    //     // All LP tokens owned by proxy
    //     assert_eq!(proxy_bal, pool_res.total_share);

    //     // Failed swap UST for WHALE due to max spread
    //     app.execute_contract(
    //         sender.clone(),
    //         tswap_dapp.clone(),
    //         &ExecuteMsg::SwapAsset {
    //             pool_id: "whale_ust_pair".to_string(),
    //             offer_id: "ust".into(),
    //             amount: Uint128::from(100u64),
    //             max_spread: Some(Decimal::zero()),
    //             belief_price: Some(Decimal::one()),
    //         },
    //         &[],
    //     )
    //     .unwrap_err();

    //     // Successful swap UST for WHALE
    //     app.execute_contract(
    //         sender.clone(),
    //         tswap_dapp.clone(),
    //         &ExecuteMsg::SwapAsset {
    //             pool_id: "whale_ust_pair".to_string(),
    //             offer_id: "ust".into(),
    //             amount: Uint128::from(100u64),
    //             max_spread: None,
    //             belief_price: None,
    //         },
    //         &[],
    //     )
    //     .unwrap();
    //     //
    //     let pool_res: PoolResponse = app
    //         .wrap()
    //         .query_wasm_smart(
    //             base_contracts.whale_ust_pair.clone(),
    //             &terraswap::pair::QueryMsg::Pool {},
    //         )
    //         .unwrap();

    //     // 1 WHALE and UST in pool
    //     assert_eq!(
    //         Uint128::from(1u64 * MILLION + 100u64),
    //         pool_res.assets[0].amount
    //     );
    //     assert_eq!(
    //         Uint128::from(1u64 * MILLION - 99u64),
    //         pool_res.assets[1].amount
    //     );

    //     // try withdrawing zero, this will fail with an error
    //     app.execute_contract(
    //         sender.clone(),
    //         tswap_dapp.clone(),
    //         &ExecuteMsg::WithdrawLiquidity {
    //             lp_token_id: "whale_ust".to_string(),
    //             amount: Uint128::zero(),
    //         },
    //         &[],
    //     )
    //     .unwrap_err();

    //     // Withdraw half of the liquidity from the pool
    //     app.execute_contract(
    //         sender.clone(),
    //         tswap_dapp.clone(),
    //         &ExecuteMsg::WithdrawLiquidity {
    //             lp_token_id: "whale_ust".to_string(),
    //             amount: Uint128::from(MILLION / 2u64),
    //         },
    //         &[],
    //     )
    //     .unwrap();

    //     //
    //     let pool_res: PoolResponse = app
    //         .wrap()
    //         .query_wasm_smart(
    //             base_contracts.whale_ust_pair.clone(),
    //             &terraswap::pair::QueryMsg::Pool {},
    //         )
    //         .unwrap();

    //     // Get proxy lp token balance
    //     let proxy_bal = lp.balance(&app, base_contracts.proxy.clone()).unwrap();

    //     // 1 WHALE and UST in pool
    //     assert_eq!(
    //         Uint128::from((1u64 * MILLION + 100u64) / 2u64),
    //         pool_res.assets[0].amount
    //     );
    //     // small rounding error
    //     assert_eq!(
    //         Uint128::from((1u64 * MILLION - 98u64) / 2u64),
    //         pool_res.assets[1].amount
    //     );
    //     // Half of the LP tokens left
    //     assert_eq!(proxy_bal, Uint128::from(MILLION / 2u64));

    //     let price = Decimal::from_ratio(pool_res.assets[0].amount, pool_res.assets[1].amount);

    //     // Provide undetailed liquidity
    //     // Should add token at same price as pool
    //     app.execute_contract(
    //         sender.clone(),
    //         tswap_dapp.clone(),
    //         &ExecuteMsg::ProvideLiquidity {
    //             pool_id: "whale_ust_pair".to_string(),
    //             main_asset_id: "whale".to_string(),
    //             amount: Uint128::from(MILLION),
    //         },
    //         &[],
    //     )
    //     .unwrap();

    //     // Provide zero liquidity
    //     // this should fail with an error
    //     app.execute_contract(
    //         sender.clone(),
    //         tswap_dapp.clone(),
    //         &ExecuteMsg::ProvideLiquidity {
    //             pool_id: "whale_ust_pair".to_string(),
    //             main_asset_id: "whale".to_string(),
    //             amount: Uint128::zero(),
    //         },
    //         &[],
    //     )
    //     .unwrap_err();

    //     //
    //     let pool_res_after: PoolResponse = app
    //         .wrap()
    //         .query_wasm_smart(
    //             base_contracts.whale_ust_pair.clone(),
    //             &terraswap::pair::QueryMsg::Pool {},
    //         )
    //         .unwrap();

    //     // 1 WHALE added to pool
    //     assert_eq!(
    //         Uint128::from(MILLION) * price,
    //         pool_res_after.assets[0].amount - pool_res.assets[0].amount
    //     );
    //     // 1.00.. UST added to pool, equating to same price as before
    //     assert_eq!(
    //         Uint128::from(MILLION),
    //         pool_res_after.assets[1].amount - pool_res.assets[1].amount
    //     );
}
