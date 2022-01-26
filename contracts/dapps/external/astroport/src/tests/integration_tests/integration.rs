use cosmwasm_std::{Addr, Coin, Decimal, Uint128};
use cw20::Cw20Contract;

use terra_multi_test::{App, ContractWrapper};

use crate::dapp_base::common::TEST_CREATOR;
use crate::msg::ExecuteMsg;
use crate::tests::integration_tests::common_integration::{
    init_contracts, mint_some_whale, mock_app,
};
use astroport::pair::PoolResponse;
use pandora::memory::msg as MemoryMsg;
use pandora::treasury::msg as TreasuryMsg;
use terra_multi_test::Executor;

use pandora::treasury::dapp_base::msg::BaseInstantiateMsg as InstantiateMsg;

use super::common_integration::{whitelist_dapp, BaseContracts};
const MILLION: u64 = 1_000_000u64;

fn init_astroport_dapp(app: &mut App, owner: Addr, base_contracts: &BaseContracts) -> Addr {
    // Upload astroport DApp Contract
    let astro_dapp_contract = Box::new(ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    ));

    let astro_dapp_code_id = app.store_code(astro_dapp_contract);

    let astro_dapp_instantiate_msg = InstantiateMsg {
        trader: owner.to_string(),
        treasury_address: base_contracts.treasury.to_string(),
        memory_addr: base_contracts.memory.to_string(),
    };

    // Init contract
    let astro_dapp_instance = app
        .instantiate_contract(
            astro_dapp_code_id,
            owner.clone(),
            &astro_dapp_instantiate_msg,
            &[],
            "astro_dapp",
            None,
        )
        .unwrap();

    whitelist_dapp(app, &owner, &base_contracts.treasury, &astro_dapp_instance);
    astro_dapp_instance
}

#[test]
fn proper_initialization() {
    let mut app = mock_app();
    let sender = Addr::unchecked(TEST_CREATOR);
    let base_contracts = init_contracts(&mut app);
    let astro_dapp = init_astroport_dapp(&mut app, sender.clone(), &base_contracts);

    let resp: TreasuryMsg::ConfigResponse = app
        .wrap()
        .query_wasm_smart(&base_contracts.treasury, &TreasuryMsg::QueryMsg::Config {})
        .unwrap();

    // Check config, astro dapp is added
    assert_eq!(1, resp.dapps.len());

    // Add whale and whale_ust token to the memory assets
    // Is tested on unit-test level
    app.execute_contract(
        sender.clone(),
        base_contracts.memory.clone(),
        &MemoryMsg::ExecuteMsg::UpdateAssetAddresses {
            to_add: vec![
                ("whale".to_string(), base_contracts.whale.to_string()),
                (
                    "whale_ust".to_string(),
                    base_contracts.whale_ust.to_string(),
                ),
                ("ust".to_string(), "uusd".to_string()),
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

    // give treasury some uusd
    app.init_bank_balance(
        &base_contracts.treasury,
        vec![Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(100u64 * MILLION),
        }],
    )
    .unwrap();

    // give treasury some whale
    mint_some_whale(
        &mut app,
        sender.clone(),
        base_contracts.whale,
        Uint128::from(100u64 * MILLION),
        base_contracts.treasury.to_string(),
    );

    // Add liquidity to pair from treasury, through astroport-dapp
    app.execute_contract(
        sender.clone(),
        astro_dapp.clone(),
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
            &astroport::pair::QueryMsg::Pool {},
        )
        .unwrap();

    let lp = Cw20Contract(base_contracts.whale_ust.clone());

    // Get treasury lp token balance
    let treasury_bal = lp.balance(&app, base_contracts.treasury.clone()).unwrap();

    // 1 WHALE and UST in pool
    assert_eq!(Uint128::from(1u64 * MILLION), pool_res.assets[0].amount);
    assert_eq!(Uint128::from(1u64 * MILLION), pool_res.assets[1].amount);
    // All LP tokens owned by treasury
    assert_eq!(treasury_bal, pool_res.total_share);

    // Failed swap UST for WHALE due to max spread
    app.execute_contract(
        sender.clone(),
        astro_dapp.clone(),
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

    // Successfull swap UST for WHALE
    app.execute_contract(
        sender.clone(),
        astro_dapp.clone(),
        &ExecuteMsg::SwapAsset {
            pool_id: "whale_ust_pair".to_string(),
            offer_id: "ust".into(),
            amount: Uint128::from(10u64),
            max_spread: Some(Decimal::percent(50u64)),
            belief_price: Some(Decimal::one()),
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
        Uint128::from(1u64 * MILLION + 10u64),
        pool_res.assets[0].amount
    );
    assert_eq!(
        Uint128::from(1u64 * MILLION - 9u64),
        pool_res.assets[1].amount
    );

    // Withdraw half of the liquidity from the pool
    app.execute_contract(
        sender.clone(),
        astro_dapp.clone(),
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
            &astroport::pair::QueryMsg::Pool {},
        )
        .unwrap();

    // Get treasury lp token balance
    let treasury_bal = lp.balance(&app, base_contracts.treasury.clone()).unwrap();
    // 1 WHALE and UST in pool
    assert_eq!(
        Uint128::from((1u64 * MILLION + 10u64) / 2u64),
        pool_res.assets[0].amount
    );
    // small rounding error
    assert_eq!(
        Uint128::from((1u64 * MILLION - 8u64) / 2u64),
        pool_res.assets[1].amount
    );
    // Half of the LP tokens left
    assert_eq!(treasury_bal, Uint128::from(MILLION / 2u64));

    let price = Decimal::from_ratio(pool_res.assets[0].amount, pool_res.assets[1].amount);

    // Provide undetailed liquidity
    // Should add token at same price as pool
    app.execute_contract(
        sender.clone(),
        astro_dapp.clone(),
        &ExecuteMsg::ProvideLiquidity {
            pool_id: "whale_ust_pair".to_string(),
            main_asset_id: "whale".to_string(),
            amount: Uint128::from(MILLION),
        },
        &[],
    )
    .unwrap();

    //
    let pool_res_after: PoolResponse = app
        .wrap()
        .query_wasm_smart(
            base_contracts.whale_ust_pair.clone(),
            &astroport::pair::QueryMsg::Pool {},
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
