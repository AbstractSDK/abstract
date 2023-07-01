use crate::asset::{format_lp_token_name, AssetInfo, AssetInfoValidated, AssetValidated};
use crate::fee_config::FeeConfig;
use crate::mock_querier::mock_dependencies;
use crate::pair::PairInfo;
use crate::querier::{
    query_all_balances, query_balance, query_pair_info, query_supply, query_token_balance,
};

use crate::factory::PairType;
use crate::DecimalCheckedOps;
use cosmwasm_std::testing::MOCK_CONTRACT_ADDR;
use cosmwasm_std::{to_binary, Addr, BankMsg, Coin, CosmosMsg, Decimal, Uint128, WasmMsg};
use cw20::Cw20ExecuteMsg;

#[test]
fn token_balance_querier() {
    let mut deps = mock_dependencies(&[]);

    deps.querier.with_token_balances(&[(
        &String::from("liquidity0000"),
        &[(&String::from(MOCK_CONTRACT_ADDR), &Uint128::new(123u128))],
    )]);

    deps.querier.with_cw20_query_handler();
    assert_eq!(
        Uint128::new(123u128),
        query_token_balance(
            &deps.as_ref().querier,
            Addr::unchecked("liquidity0000"),
            Addr::unchecked(MOCK_CONTRACT_ADDR),
        )
        .unwrap()
    );
    deps.querier.with_default_query_handler()
}

#[test]
fn balance_querier() {
    let deps = mock_dependencies(&[Coin {
        denom: "uusd".to_string(),
        amount: Uint128::new(200u128),
    }]);

    assert_eq!(
        query_balance(
            &deps.as_ref().querier,
            Addr::unchecked(MOCK_CONTRACT_ADDR),
            "uusd".to_string()
        )
        .unwrap(),
        Uint128::new(200u128)
    );
}

#[test]
fn all_balances_querier() {
    let deps = mock_dependencies(&[
        Coin {
            denom: "uusd".to_string(),
            amount: Uint128::new(200u128),
        },
        Coin {
            denom: "ukrw".to_string(),
            amount: Uint128::new(300u128),
        },
    ]);

    assert_eq!(
        query_all_balances(&deps.as_ref().querier, Addr::unchecked(MOCK_CONTRACT_ADDR),).unwrap(),
        vec![
            Coin {
                denom: "uusd".to_string(),
                amount: Uint128::new(200u128),
            },
            Coin {
                denom: "ukrw".to_string(),
                amount: Uint128::new(300u128),
            }
        ]
    );
}

#[test]
fn supply_querier() {
    let mut deps = mock_dependencies(&[]);

    deps.querier.with_token_balances(&[(
        &String::from("liquidity0000"),
        &[
            (&String::from(MOCK_CONTRACT_ADDR), &Uint128::new(123u128)),
            (&String::from("addr00000"), &Uint128::new(123u128)),
            (&String::from("addr00001"), &Uint128::new(123u128)),
            (&String::from("addr00002"), &Uint128::new(123u128)),
        ],
    )]);

    deps.querier.with_cw20_query_handler();

    assert_eq!(
        query_supply(&deps.as_ref().querier, Addr::unchecked("liquidity0000")).unwrap(),
        Uint128::new(492u128)
    )
}

#[test]
fn test_asset_info() {
    let token_info: AssetInfoValidated = AssetInfoValidated::Token(Addr::unchecked("asset0000"));
    let native_token_info: AssetInfoValidated = AssetInfoValidated::Native("uusd".to_string());

    assert!(!token_info.equal(&native_token_info));

    assert!(!token_info.equal(&AssetInfoValidated::Token(Addr::unchecked("asset0001"))));

    assert!(token_info.equal(&AssetInfoValidated::Token(Addr::unchecked("asset0000"))));

    assert!(native_token_info.is_native_token());
    assert!(!token_info.is_native_token());

    let mut deps = mock_dependencies(&[Coin {
        denom: "uusd".to_string(),
        amount: Uint128::new(123),
    }]);
    deps.querier.with_token_balances(&[(
        &String::from("asset0000"),
        &[
            (&String::from(MOCK_CONTRACT_ADDR), &Uint128::new(123u128)),
            (&String::from("addr00000"), &Uint128::new(123u128)),
            (&String::from("addr00001"), &Uint128::new(123u128)),
            (&String::from("addr00002"), &Uint128::new(123u128)),
        ],
    )]);

    assert_eq!(
        native_token_info
            .query_balance(&deps.as_ref().querier, Addr::unchecked(MOCK_CONTRACT_ADDR))
            .unwrap(),
        Uint128::new(123u128)
    );
    deps.querier.with_cw20_query_handler();
    assert_eq!(
        token_info
            .query_balance(&deps.as_ref().querier, Addr::unchecked(MOCK_CONTRACT_ADDR))
            .unwrap(),
        Uint128::new(123u128)
    );
}

#[test]
fn test_asset() {
    let mut deps = mock_dependencies(&[Coin {
        denom: "uusd".to_string(),
        amount: Uint128::new(123),
    }]);

    deps.querier.with_token_balances(&[(
        &String::from("asset0000"),
        &[
            (&String::from(MOCK_CONTRACT_ADDR), &Uint128::new(123u128)),
            (&String::from("addr00000"), &Uint128::new(123u128)),
            (&String::from("addr00001"), &Uint128::new(123u128)),
            (&String::from("addr00002"), &Uint128::new(123u128)),
        ],
    )]);

    let token_asset = AssetValidated {
        amount: Uint128::new(123123u128),
        info: AssetInfoValidated::Token(Addr::unchecked("asset0000")),
    };

    let native_token_asset = AssetValidated {
        amount: Uint128::new(123123u128),
        info: AssetInfoValidated::Native("uusd".to_string()),
    };

    assert_eq!(
        token_asset.into_msg(Addr::unchecked("addr0000")).unwrap(),
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: String::from("asset0000"),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: String::from("addr0000"),
                amount: Uint128::new(123123u128),
            })
            .unwrap(),
            funds: vec![],
        })
    );

    assert_eq!(
        native_token_asset
            .into_msg(Addr::unchecked("addr0000"))
            .unwrap(),
        CosmosMsg::Bank(BankMsg::Send {
            to_address: String::from("addr0000"),
            amount: vec![Coin {
                denom: "uusd".to_string(),
                amount: Uint128::new(123123u128),
            }]
        })
    );
}

#[test]
fn query_wyndex_pair_contract() {
    let mut deps = mock_dependencies(&[]);

    deps.querier.with_wyndex_pairs(&[(
        &"asset0000uusd".to_string(),
        &PairInfo {
            asset_infos: vec![
                AssetInfoValidated::Token(Addr::unchecked("asset0000")),
                AssetInfoValidated::Native("uusd".to_string()),
            ],
            contract_addr: Addr::unchecked("pair0000"),
            staking_addr: Addr::unchecked("stake0000"),
            liquidity_token: Addr::unchecked("liquidity0000"),
            pair_type: PairType::Xyk {},
            fee_config: FeeConfig {
                protocol_fee_bps: 0,
                total_fee_bps: 0,
            },
        },
    )]);

    let pair_info: PairInfo = query_pair_info(
        &deps.as_ref().querier,
        Addr::unchecked(MOCK_CONTRACT_ADDR),
        &[
            AssetInfo::Token("asset0000".to_string()),
            AssetInfo::Native("uusd".to_string()),
        ],
    )
    .unwrap();

    assert_eq!(pair_info.contract_addr, String::from("pair0000"),);
    assert_eq!(pair_info.liquidity_token, String::from("liquidity0000"),);
}

#[test]
fn test_format_lp_token_name() {
    let mut deps = mock_dependencies(&[]);
    deps.querier.with_wyndex_pairs(&[(
        &"asset0000uusd".to_string(),
        &PairInfo {
            asset_infos: vec![
                AssetInfoValidated::Token(Addr::unchecked("asset0000")),
                AssetInfoValidated::Native("uusd".to_string()),
            ],
            contract_addr: Addr::unchecked("pair0000"),
            staking_addr: Addr::unchecked("stake0000"),
            liquidity_token: Addr::unchecked("liquidity0000"),
            pair_type: PairType::Xyk {},
            fee_config: FeeConfig {
                protocol_fee_bps: 0,
                total_fee_bps: 0,
            },
        },
    )]);

    let pair_info: PairInfo = query_pair_info(
        &deps.as_ref().querier,
        Addr::unchecked(MOCK_CONTRACT_ADDR),
        &[
            AssetInfo::Token("asset0000".to_string()),
            AssetInfo::Native("uusd".to_string()),
        ],
    )
    .unwrap();

    deps.querier.with_token_balances(&[(
        &String::from("asset0000"),
        &[(&String::from(MOCK_CONTRACT_ADDR), &Uint128::new(123u128))],
    )]);

    deps.querier.with_cw20_query_handler();

    let lp_name = format_lp_token_name(&pair_info.asset_infos, &deps.as_ref().querier).unwrap();
    assert_eq!(lp_name, "MAPP-UUSD-LP")
}

#[test]
fn test_decimal_checked_ops() {
    for i in 0u32..100u32 {
        let dec = Decimal::from_ratio(i, 1u32);
        assert_eq!(dec + dec, dec.checked_add(dec).unwrap());
    }
    assert!(
        Decimal::from_ratio(Uint128::MAX, Uint128::from(10u128.pow(18u32)))
            .checked_add(Decimal::one())
            .is_err()
    );

    for i in 0u128..100u128 {
        let dec = Decimal::from_ratio(i, 1u128);
        assert_eq!(
            dec * Uint128::new(i),
            dec.checked_mul_uint128(Uint128::from(i)).unwrap()
        );
    }
    assert!(
        Decimal::from_ratio(Uint128::MAX, Uint128::from(10u128.pow(18u32)))
            .checked_mul(Decimal::new(Uint128::from(10u128.pow(18u32) + 1u128)))
            .is_err()
    );
}
