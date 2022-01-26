use cosmwasm_std::{to_binary, Addr, Coin, Decimal, Uint128};
use cw20::{BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg, TokenInfoResponse};

use terra_multi_test::App;

use crate::msg::{DepositHookMsg, ExecuteMsg};
use crate::tests::integration_tests::common_integration::{
    init_contracts, mint_some_whale, mock_app,
};
use crate::tests::integration_tests::instantiate::{configure_memory, init_vault_dapp};
use terra_multi_test::Executor;
use terraswap::asset::Asset;

use crate::dapp_base::common::TEST_CREATOR;
use pandora::treasury::msg as TreasuryMsg;

use super::update::change_base_to_whale;
const MILLION: u64 = 1_000_000u64;

#[test]
fn proper_initialization() {
    let mut app = mock_app();
    let sender = Addr::unchecked(TEST_CREATOR);
    let base_contracts = init_contracts(&mut app);
    configure_memory(&mut app, sender.clone(), &base_contracts);
    let (_vault_dapp, _vault_l_token) = init_vault_dapp(&mut app, sender.clone(), &base_contracts);

    let resp: TreasuryMsg::ConfigResponse = app
        .wrap()
        .query_wasm_smart(&base_contracts.treasury, &TreasuryMsg::QueryMsg::Config {})
        .unwrap();
    // Check config, vault dapp is added
    assert_eq!(1, resp.dapps.len());
}

#[test]
fn deposit_and_withdraw_with_native_token() {
    let mut app = mock_app();
    let sender = Addr::unchecked(TEST_CREATOR);
    let base_contracts = init_contracts(&mut app);
    configure_memory(&mut app, sender.clone(), &base_contracts);
    let (vault_dapp, vault_l_token) = init_vault_dapp(&mut app, sender.clone(), &base_contracts);

    // give sender some uusd
    app.init_bank_balance(
        &sender,
        vec![Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(100u64 * MILLION),
        }],
    )
    .unwrap();

    // Try deposit without sending tokens, should err
    app.execute_contract(
        sender.clone(),
        vault_dapp.clone(),
        &ExecuteMsg::ProvideLiquidity {
            asset: Asset {
                info: terraswap::asset::AssetInfo::NativeToken {
                    denom: "uusd".to_string(),
                },
                amount: Uint128::from(10u64 * MILLION),
            },
        },
        &[],
    )
    .unwrap_err();

    // Add UST to treasury through vault dapp contract interaction
    app.execute_contract(
        sender.clone(),
        vault_dapp.clone(),
        &ExecuteMsg::ProvideLiquidity {
            asset: Asset {
                info: terraswap::asset::AssetInfo::NativeToken {
                    denom: "uusd".to_string(),
                },
                amount: Uint128::from(10u64 * MILLION),
            },
        },
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(10u64 * MILLION),
        }],
    )
    .unwrap();

    // Check treasury Value
    let treasury_res: TreasuryMsg::TotalValueResponse = app
        .wrap()
        .query_wasm_smart(
            base_contracts.treasury.clone(),
            &TreasuryMsg::QueryMsg::TotalValue {},
        )
        .unwrap();

    // Value of vault = deposit
    assert_eq!(10_000_000u128, treasury_res.value.u128());

    // First addition to pool so we own it all -> 10 UST
    let owned_locked_value =
        liquidity_token_value(&app, &vault_l_token, &base_contracts.treasury, &sender);
    assert_eq!(Uint128::from(10u64 * MILLION), owned_locked_value);

    let staker_balance: BalanceResponse = app
        .wrap()
        .query_wasm_smart(
            &vault_l_token,
            &Cw20QueryMsg::Balance {
                address: sender.to_string(),
            },
        )
        .unwrap();

    // token balance = sent balance
    assert_eq!(10_000_000u128, staker_balance.balance.u128());

    // add some whale to the treasury
    // worth 1000 UST
    mint_some_whale(
        &mut app,
        sender.clone(),
        base_contracts.whale.clone(),
        Uint128::from(2_000u64 * MILLION),
        base_contracts.treasury.to_string(),
    );

    // Check treasury Value
    let treasury_res: TreasuryMsg::TotalValueResponse = app
        .wrap()
        .query_wasm_smart(
            base_contracts.treasury.clone(),
            &TreasuryMsg::QueryMsg::TotalValue {},
        )
        .unwrap();

    // Value should be 10_000_000 UST + 0.5 UST/WHALE * 2_000u64*MILLION WHALE
    assert_eq!(
        (10_000_000u64 + 2_000u64 * MILLION / 2) as u128,
        treasury_res.value.u128()
    );

    // Withdraw from vault.
    app.execute_contract(
        sender.clone(),
        vault_l_token.clone(),
        &cw20::Cw20ExecuteMsg::Send {
            contract: vault_dapp.to_string(),
            amount: Uint128::from(10_000_000u128),
            msg: to_binary(&DepositHookMsg::WithdrawLiquidity {}).unwrap(),
        },
        &[],
    )
    .unwrap();

    // We withdrew everthing so own 0 in liquidity tokens
    let owned_locked_value =
        liquidity_token_value(&app, &vault_l_token, &base_contracts.treasury, &sender);
    assert_eq!(Uint128::from(0u64), owned_locked_value);

    // Check treasury Value
    let treasury_res: TreasuryMsg::TotalValueResponse = app
        .wrap()
        .query_wasm_smart(
            base_contracts.treasury.clone(),
            &TreasuryMsg::QueryMsg::TotalValue {},
        )
        .unwrap();
    // 10% fee so 10% remains in the pool
    assert_eq!(
        ((10_000_000u64 + 2_000u64 * MILLION / 2) / 10) as u128,
        treasury_res.value.u128()
    );

    // Check whale recieved by withdrawer
    let whale_balance: BalanceResponse = app
        .wrap()
        .query_wasm_smart(
            &base_contracts.whale,
            &Cw20QueryMsg::Balance {
                address: sender.to_string(),
            },
        )
        .unwrap();

    let sender_whale_balance = whale_balance.balance;

    assert_eq!(
        // Total amount minted to pool - 10% fee
        ((2_000u64 * MILLION) as f64 * 0.9f64) as u128,
        sender_whale_balance.u128()
    );
}

#[test]
fn deposit_and_withdraw_with_cw20() {
    let mut app = mock_app();
    let sender = Addr::unchecked(TEST_CREATOR);
    let base_contracts = init_contracts(&mut app);
    configure_memory(&mut app, sender.clone(), &base_contracts);
    let (vault_dapp, vault_l_token) = init_vault_dapp(&mut app, sender.clone(), &base_contracts);
    // Change the base token to whale
    change_base_to_whale(&mut app, sender.clone(), &base_contracts, &vault_dapp);

    // give sender some uusd
    app.init_bank_balance(
        &sender,
        vec![Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(100u64 * MILLION),
        }],
    )
    .unwrap();

    // Try deposit with UST, should error
    app.execute_contract(
        sender.clone(),
        vault_dapp.clone(),
        &ExecuteMsg::ProvideLiquidity {
            asset: Asset {
                info: terraswap::asset::AssetInfo::NativeToken {
                    denom: "uusd".to_string(),
                },
                amount: Uint128::from(10u64 * MILLION),
            },
        },
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(10u64 * MILLION),
        }],
    )
    .unwrap_err();

    // Try deposit with WHALE while actually sending ust, should error
    app.execute_contract(
        sender.clone(),
        vault_dapp.clone(),
        &ExecuteMsg::ProvideLiquidity {
            asset: Asset {
                info: terraswap::asset::AssetInfo::Token {
                    contract_addr: base_contracts.whale.to_string(),
                },
                amount: Uint128::from(10u64 * MILLION),
            },
        },
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(10u64 * MILLION),
        }],
    )
    .unwrap_err();

    mint_some_whale(
        &mut app,
        sender.clone(),
        base_contracts.whale.clone(),
        Uint128::from(10u64 * MILLION),
        sender.to_string(),
    );

    // Deposit with WHALE
    app.execute_contract(
        sender.clone(),
        base_contracts.whale.clone(),
        &Cw20ExecuteMsg::Send {
            contract: vault_dapp.to_string(),
            amount: Uint128::from(10u64 * MILLION),
            msg: to_binary(&DepositHookMsg::ProvideLiquidity {}).unwrap(),
        },
        &[],
    )
    .unwrap();

    // All tokens are claimed by sender as he's only provider
    let owned_locked_value =
        liquidity_token_value(&app, &vault_l_token, &base_contracts.treasury, &sender);
    // Base token is whale so it's worth 10 whale
    assert_eq!(Uint128::from(10u64 * MILLION), owned_locked_value);

    // Add 50 UST to vault, value should be 110 WHALE
    app.init_bank_balance(
        &base_contracts.treasury,
        vec![Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(50u64 * MILLION),
        }],
    )
    .unwrap();

    let owned_locked_value =
        liquidity_token_value(&app, &vault_l_token, &base_contracts.treasury, &sender);
    assert_eq!(Uint128::from(110u64 * MILLION), owned_locked_value);

    // Withdraw all from vault.
    app.execute_contract(
        sender.clone(),
        vault_l_token.clone(),
        &cw20::Cw20ExecuteMsg::Send {
            contract: vault_dapp.to_string(),
            amount: Uint128::from(10u64 * MILLION),
            msg: to_binary(&DepositHookMsg::WithdrawLiquidity {}).unwrap(),
        },
        &[],
    )
    .unwrap();

    let owned_locked_value =
        liquidity_token_value(&app, &vault_l_token, &base_contracts.treasury, &sender);
    // owned value is 0
    assert_eq!(Uint128::from(0u64 * MILLION), owned_locked_value);

    // User got whale and ust back
    let sender_balance = app
        .wrap()
        .query_balance(sender.clone(), "uusd")
        .unwrap()
        .amount;

    // initial balance + vaultvalue - fee
    // 100 + 50 - (50 * 0.1)
    // 100 + 45
    assert_eq!(Uint128::from(145u64 * MILLION), sender_balance);

    // Check whale recieved by withdrawer
    let whale_balance: BalanceResponse = app
        .wrap()
        .query_wasm_smart(
            &base_contracts.whale,
            &Cw20QueryMsg::Balance {
                address: sender.to_string(),
            },
        )
        .unwrap();

    // deposit - 10% fee
    assert_eq!(Uint128::from(9u64 * MILLION), whale_balance.balance);
}

fn liquidity_token_value(app: &App, l_token: &Addr, treasury_addr: &Addr, owner: &Addr) -> Uint128 {
    let info_res: TokenInfoResponse = app
        .wrap()
        .query_wasm_smart(l_token, &Cw20QueryMsg::TokenInfo {})
        .unwrap();

    let total_supply = info_res.total_supply;

    let balance: BalanceResponse = app
        .wrap()
        .query_wasm_smart(
            l_token,
            &Cw20QueryMsg::Balance {
                address: owner.to_string(),
            },
        )
        .unwrap();

    let vault_res: TreasuryMsg::TotalValueResponse = app
        .wrap()
        .query_wasm_smart(treasury_addr, &TreasuryMsg::QueryMsg::TotalValue {})
        .unwrap();

    // value per liquidity token = total value/total supply
    let liquidity_token_value = Decimal::from_ratio(vault_res.value, total_supply);
    balance.balance * liquidity_token_value
}
