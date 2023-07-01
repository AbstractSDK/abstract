use wyndex::asset::{Asset, AssetInfo, AssetInfoExt, AssetInfoValidated};
use wyndex::factory::{
    DefaultStakeConfig, ExecuteMsg as FactoryExecuteMsg, InstantiateMsg as FactoryInstantiateMsg,
    PairConfig, PairType, PartialStakeConfig, QueryMsg as FactoryQueryMsg,
};
use wyndex::fee_config::FeeConfig;
use wyndex::pair::{
    ConfigResponse, CumulativePricesResponse, Cw20HookMsg, ExecuteMsg, InstantiateMsg, PairInfo,
    PoolResponse, QueryMsg, StablePoolConfig, StablePoolParams, StablePoolUpdateParams,
    TWAP_PRECISION,
};

use cosmwasm_std::{
    attr, from_binary, to_binary, Addr, Coin, Decimal, QueryRequest, Uint128, WasmQuery,
};
use cw20::{BalanceResponse, Cw20Coin, Cw20ExecuteMsg, Cw20QueryMsg, MinterResponse};
use cw20_base::msg::InstantiateMsg as TokenInstantiateMsg;
use cw_multi_test::{App, ContractWrapper, Executor};
use wyndex::querier::query_token_balance;
use wyndex_pair_lsd::math::{MAX_AMP, MAX_AMP_CHANGE, MIN_AMP_CHANGING_TIME};

const OWNER: &str = "owner";

fn mock_app(owner: Addr, coins: Vec<Coin>) -> App {
    App::new(|router, _, storage| {
        // initialization moved to App construction
        router.bank.init_balance(storage, &owner, coins).unwrap()
    })
}

fn store_token_code(app: &mut App) -> u64 {
    let astro_token_contract = Box::new(ContractWrapper::new_with_empty(
        cw20_base::contract::execute,
        cw20_base::contract::instantiate,
        cw20_base::contract::query,
    ));

    app.store_code(astro_token_contract)
}

fn store_pair_code(app: &mut App) -> u64 {
    let pair_contract = Box::new(
        ContractWrapper::new_with_empty(
            wyndex_pair_lsd::contract::execute,
            wyndex_pair_lsd::contract::instantiate,
            wyndex_pair_lsd::contract::query,
        )
        .with_reply_empty(wyndex_pair_lsd::contract::reply),
    );

    app.store_code(pair_contract)
}

fn store_factory_code(app: &mut App) -> u64 {
    let factory_contract = Box::new(
        ContractWrapper::new_with_empty(
            wyndex_factory::contract::execute,
            wyndex_factory::contract::instantiate,
            wyndex_factory::contract::query,
        )
        .with_reply_empty(wyndex_factory::contract::reply),
    );

    app.store_code(factory_contract)
}

fn store_stake_code(app: &mut App) -> u64 {
    let staking_contract = Box::new(ContractWrapper::new_with_empty(
        wyndex_stake::contract::execute,
        wyndex_stake::contract::instantiate,
        wyndex_stake::contract::query,
    ));

    app.store_code(staking_contract)
}

fn instantiate_factory(router: &mut App, owner: &Addr) -> Addr {
    let token_contract_code_id = store_token_code(router);
    let pair_contract_code_id = store_pair_code(router);
    let factory_code_id = store_factory_code(router);
    let stake_code_id = store_stake_code(router);

    let fee_config = FeeConfig {
        protocol_fee_bps: 5000,
        total_fee_bps: 5,
    };

    let msg = FactoryInstantiateMsg {
        fee_address: Some(owner.to_string()),
        pair_configs: vec![PairConfig {
            code_id: pair_contract_code_id,
            fee_config,
            pair_type: PairType::Lsd {},
            is_disabled: false,
        }],
        token_code_id: token_contract_code_id,
        owner: owner.to_string(),
        max_referral_commission: Decimal::one(),
        default_stake_config: default_stake_config(stake_code_id),
        trading_starts: None,
    };

    router
        .instantiate_contract(factory_code_id, owner.clone(), &msg, &[], "FACTORY", None)
        .unwrap()
}

fn instantiate_pair(router: &mut App, owner: &Addr) -> Addr {
    let factory = instantiate_factory(router, owner);

    // instantiate pair
    let asset_infos = vec![
        AssetInfo::Native("uusd".to_string()),
        AssetInfo::Native("uluna".to_string()),
    ];
    let msg = FactoryExecuteMsg::CreatePair {
        pair_type: PairType::Lsd {},
        asset_infos: asset_infos.clone(),
        init_params: None,
        total_fee_bps: None,
        staking_config: PartialStakeConfig::default(),
    };

    let resp = router
        .execute_contract(owner.clone(), factory.clone(), &msg, &[])
        .unwrap_err();
    assert_eq!(
        "You need to provide init params",
        resp.root_cause().to_string()
    );

    let msg = FactoryExecuteMsg::CreatePair {
        pair_type: PairType::Lsd {},
        asset_infos: asset_infos.clone(),
        init_params: Some(
            to_binary(&StablePoolParams {
                amp: 100,
                owner: None,
                lsd: None,
            })
            .unwrap(),
        ),
        total_fee_bps: None,
        staking_config: PartialStakeConfig::default(),
    };
    router
        .execute_contract(owner.clone(), factory.clone(), &msg, &[])
        .unwrap();

    // get pair address
    let pair = router
        .wrap()
        .query_wasm_smart::<PairInfo>(factory, &FactoryQueryMsg::Pair { asset_infos })
        .unwrap()
        .contract_addr;

    let res: PairInfo = router
        .wrap()
        .query_wasm_smart(pair.clone(), &QueryMsg::Pair {})
        .unwrap();
    assert_eq!("contract1", res.contract_addr);
    assert_eq!("contract2", res.liquidity_token);

    pair
}

fn instantiate_token(router: &mut App, owner: &Addr, balances: &[(&str, u128)]) -> Addr {
    let token_contract_code_id = store_token_code(router);
    router
        .instantiate_contract(
            token_contract_code_id,
            owner.clone(),
            &TokenInstantiateMsg {
                name: "Foo token".to_string(),
                symbol: "FOO".to_string(),
                decimals: 6,
                initial_balances: balances
                    .iter()
                    .map(|&(user, amount)| Cw20Coin {
                        address: user.to_string(),
                        amount: Uint128::from(amount),
                    })
                    .collect(),
                mint: None,
                marketing: None,
            },
            &[],
            String::from("FOO"),
            None,
        )
        .unwrap()
}

fn default_stake_config(staking_code_id: u64) -> DefaultStakeConfig {
    DefaultStakeConfig {
        staking_code_id,
        tokens_per_power: Uint128::new(1000),
        min_bond: Uint128::new(1000),
        unbonding_periods: vec![1],
        max_distributions: 6,
        converter: None,
    }
}

/// Instantiate a pair with a cw20 token as one of the assets
fn instantiate_mixed_pair(
    router: &mut App,
    owner: &Addr,
    cw20_balances: &[(&str, u128)],
) -> (Addr, Addr) {
    // instantiate cw20 token to use as a one of the assets
    let cw20_token = instantiate_token(router, owner, cw20_balances);

    let factory = instantiate_factory(router, owner);

    // instantiate pair
    let asset_infos = vec![
        AssetInfo::Native("uusd".to_string()),
        AssetInfo::Token(cw20_token.to_string()),
    ];
    let msg = FactoryExecuteMsg::CreatePair {
        pair_type: PairType::Lsd {},
        asset_infos: asset_infos.clone(),
        init_params: Some(
            to_binary(&StablePoolParams {
                amp: 100,
                owner: None,
                lsd: None,
            })
            .unwrap(),
        ),
        total_fee_bps: None,
        staking_config: PartialStakeConfig::default(),
    };
    router
        .execute_contract(owner.clone(), factory.clone(), &msg, &[])
        .unwrap();

    // get pair address
    let pair = router
        .wrap()
        .query_wasm_smart::<PairInfo>(factory, &FactoryQueryMsg::Pair { asset_infos })
        .unwrap()
        .contract_addr;

    (pair, cw20_token)
}

/// Provide liquidity with a cw20 token as one of the assets
fn provide_liquidity_mixed_msg(
    uusd_amount: Uint128,
    cw20_amount: Uint128,
    cw20_token: &Addr,
    receiver: Option<String>,
    slippage_tolerance: Option<Decimal>,
) -> (ExecuteMsg, [Coin; 1]) {
    let msg = ExecuteMsg::ProvideLiquidity {
        assets: vec![
            Asset {
                info: AssetInfo::Native("uusd".to_string()),
                amount: uusd_amount,
            },
            Asset {
                info: AssetInfo::Token(cw20_token.to_string()),
                amount: cw20_amount,
            },
        ],
        slippage_tolerance,
        receiver,
    };

    let coins = [Coin {
        denom: "uusd".to_string(),
        amount: uusd_amount,
    }];

    (msg, coins)
}

#[test]
fn test_provide_and_withdraw_liquidity() {
    let owner = Addr::unchecked("owner");
    let alice_address = Addr::unchecked("alice");

    let mut router = mock_app(
        owner.clone(),
        vec![
            Coin {
                denom: "uusd".to_string(),
                amount: Uint128::new(100_000_000_000u128),
            },
            Coin {
                denom: "uluna".to_string(),
                amount: Uint128::new(100_000_000_000u128),
            },
        ],
    );

    // Set Alice's balances
    router
        .send_tokens(
            owner.clone(),
            alice_address.clone(),
            &[
                Coin {
                    denom: "uusd".to_string(),
                    amount: Uint128::new(233_000u128),
                },
                Coin {
                    denom: "uluna".to_string(),
                    amount: Uint128::new(200_000u128),
                },
            ],
        )
        .unwrap();

    // Init pair
    let pair_instance = instantiate_pair(&mut router, &owner);

    let res: Result<PairInfo, _> = router.wrap().query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: pair_instance.to_string(),
        msg: to_binary(&QueryMsg::Pair {}).unwrap(),
    }));
    let res = res.unwrap();

    assert_eq!(
        res.asset_infos,
        [
            AssetInfoValidated::Native("uusd".to_string()),
            AssetInfoValidated::Native("uluna".to_string()),
        ],
    );

    // When dealing with native tokens, the transfer should happen before the contract call, which cw-multitest doesn't support
    router
        .send_tokens(
            owner.clone(),
            pair_instance.clone(),
            &[
                Coin {
                    denom: "uusd".to_string(),
                    amount: Uint128::new(100_000u128),
                },
                Coin {
                    denom: "uluna".to_string(),
                    amount: Uint128::new(100_000u128),
                },
            ],
        )
        .unwrap();

    // Provide liquidity
    let (msg, coins) = provide_liquidity_msg(Uint128::new(100), Uint128::new(100), None);
    let res = router
        .execute_contract(alice_address.clone(), pair_instance.clone(), &msg, &coins)
        .unwrap();

    assert_eq!(
        res.events[1].attributes[1],
        attr("action", "provide_liquidity")
    );
    assert_eq!(res.events[1].attributes[3], attr("receiver", "alice"),);
    assert_eq!(
        res.events[1].attributes[4],
        attr("assets", "100uusd, 100uluna")
    );
    assert_eq!(
        res.events[1].attributes[5],
        attr("share", 199200u128.to_string())
    );

    assert_eq!(res.events[3].attributes[1], attr("action", "mint"));
    assert_eq!(res.events[3].attributes[2], attr("to", "contract1"));
    assert_eq!(
        res.events[3].attributes[3],
        attr("amount", 1000.to_string())
    );

    assert_eq!(res.events[5].attributes[1], attr("action", "mint"));
    assert_eq!(res.events[5].attributes[2], attr("to", "alice"));
    assert_eq!(
        res.events[5].attributes[3],
        attr("amount", 199200u128.to_string())
    );

    // Provide liquidity for a custom receiver
    let (msg, coins) = provide_liquidity_msg(
        Uint128::new(100),
        Uint128::new(100),
        Some("bob".to_string()),
    );
    let res = router
        .execute_contract(alice_address, pair_instance, &msg, &coins)
        .unwrap();

    assert_eq!(
        res.events[1].attributes[1],
        attr("action", "provide_liquidity")
    );
    assert_eq!(res.events[1].attributes[3], attr("receiver", "bob"),);
    assert_eq!(
        res.events[1].attributes[4],
        attr("assets", "100uusd, 100uluna")
    );
    assert_eq!(
        res.events[1].attributes[5],
        attr("share", 200u128.to_string())
    );
    assert_eq!(res.events[3].attributes[1], attr("action", "mint"));
    assert_eq!(res.events[3].attributes[2], attr("to", "bob"));
    assert_eq!(res.events[3].attributes[3], attr("amount", 200.to_string()));
}

fn provide_liquidity_msg(
    uusd_amount: Uint128,
    uluna_amount: Uint128,
    receiver: Option<String>,
) -> (ExecuteMsg, [Coin; 2]) {
    let msg = ExecuteMsg::ProvideLiquidity {
        assets: vec![
            Asset {
                info: AssetInfo::Native("uusd".to_string()),
                amount: uusd_amount,
            },
            Asset {
                info: AssetInfo::Native("uluna".to_string()),
                amount: uluna_amount,
            },
        ],
        slippage_tolerance: None,
        receiver,
    };

    let coins = [
        Coin {
            denom: "uluna".to_string(),
            amount: uluna_amount,
        },
        Coin {
            denom: "uusd".to_string(),
            amount: uusd_amount,
        },
    ];

    (msg, coins)
}

#[test]
fn provide_lp_for_single_token() {
    let owner = Addr::unchecked(OWNER);
    let mut app = mock_app(
        owner.clone(),
        vec![
            Coin {
                denom: "uusd".to_string(),
                amount: Uint128::new(100_000_000_000u128),
            },
            Coin {
                denom: "uluna".to_string(),
                amount: Uint128::new(100_000_000_000u128),
            },
        ],
    );

    let token_code_id = store_token_code(&mut app);

    let x_amount = Uint128::new(9_000_000_000_000_000);
    let y_amount = Uint128::new(9_000_000_000_000_000);
    let x_offer = Uint128::new(1_000_000_000_000_000);
    let swap_amount = Uint128::new(120_000_000);

    let token_name = "Xtoken";

    let init_msg = TokenInstantiateMsg {
        name: token_name.to_string(),
        symbol: token_name.to_string(),
        decimals: 6,
        initial_balances: vec![Cw20Coin {
            address: OWNER.to_string(),
            amount: x_amount,
        }],
        mint: Some(MinterResponse {
            minter: String::from(OWNER),
            cap: None,
        }),
        marketing: None,
    };

    let token_x_instance = app
        .instantiate_contract(
            token_code_id,
            owner.clone(),
            &init_msg,
            &[],
            token_name,
            None,
        )
        .unwrap();

    let token_name = "Ytoken";

    let init_msg = TokenInstantiateMsg {
        name: token_name.to_string(),
        symbol: token_name.to_string(),
        decimals: 6,
        initial_balances: vec![Cw20Coin {
            address: OWNER.to_string(),
            amount: y_amount,
        }],
        mint: Some(MinterResponse {
            minter: String::from(OWNER),
            cap: None,
        }),
        marketing: None,
    };

    let token_y_instance = app
        .instantiate_contract(
            token_code_id,
            owner.clone(),
            &init_msg,
            &[],
            token_name,
            None,
        )
        .unwrap();

    let pair_code_id = store_pair_code(&mut app);
    let factory_code_id = store_factory_code(&mut app);
    let stake_code_id = store_stake_code(&mut app);

    let init_msg = FactoryInstantiateMsg {
        fee_address: None,
        pair_configs: vec![PairConfig {
            code_id: pair_code_id,
            fee_config: FeeConfig {
                protocol_fee_bps: 0,
                total_fee_bps: 0,
            },
            pair_type: PairType::Lsd {},
            is_disabled: false,
        }],
        token_code_id,
        owner: String::from("owner0000"),
        max_referral_commission: Decimal::one(),
        default_stake_config: default_stake_config(stake_code_id),
        trading_starts: None,
    };

    let factory_instance = app
        .instantiate_contract(
            factory_code_id,
            owner.clone(),
            &init_msg,
            &[],
            "FACTORY",
            None,
        )
        .unwrap();

    let msg = FactoryExecuteMsg::CreatePair {
        pair_type: PairType::Lsd {},
        asset_infos: vec![
            AssetInfo::Token(token_x_instance.to_string()),
            AssetInfo::Token(token_y_instance.to_string()),
        ],
        init_params: Some(
            to_binary(&StablePoolParams {
                amp: 100,
                owner: None,
                lsd: None,
            })
            .unwrap(),
        ),
        staking_config: PartialStakeConfig::default(),
        total_fee_bps: None,
    };

    app.execute_contract(
        Addr::unchecked("owner0000"),
        factory_instance.clone(),
        &msg,
        &[],
    )
    .unwrap();

    let msg = FactoryQueryMsg::Pair {
        asset_infos: vec![
            AssetInfo::Token(token_x_instance.to_string()),
            AssetInfo::Token(token_y_instance.to_string()),
        ],
    };

    let res: PairInfo = app
        .wrap()
        .query_wasm_smart(&factory_instance, &msg)
        .unwrap();

    let pair_instance = res.contract_addr;

    let msg = Cw20ExecuteMsg::IncreaseAllowance {
        spender: pair_instance.to_string(),
        expires: None,
        amount: x_amount,
    };

    app.execute_contract(owner.clone(), token_x_instance.clone(), &msg, &[])
        .unwrap();

    let msg = Cw20ExecuteMsg::IncreaseAllowance {
        spender: pair_instance.to_string(),
        expires: None,
        amount: y_amount,
    };

    app.execute_contract(owner.clone(), token_y_instance.clone(), &msg, &[])
        .unwrap();

    let swap_msg = Cw20ExecuteMsg::Send {
        contract: pair_instance.to_string(),
        msg: to_binary(&Cw20HookMsg::Swap {
            ask_asset_info: None,
            belief_price: None,
            max_spread: None,
            to: None,
            referral_address: None,
            referral_commission: None,
        })
        .unwrap(),
        amount: swap_amount,
    };

    let err = app
        .execute_contract(owner.clone(), token_x_instance.clone(), &swap_msg, &[])
        .unwrap_err();
    assert_eq!(
        "Generic error: One of the pools is empty",
        err.root_cause().to_string()
    );

    let msg = ExecuteMsg::ProvideLiquidity {
        assets: vec![
            Asset {
                info: AssetInfo::Token(token_x_instance.to_string()),
                amount: x_offer,
            },
            Asset {
                info: AssetInfo::Token(token_y_instance.to_string()),
                amount: Uint128::zero(),
            },
        ],
        slippage_tolerance: None,
        receiver: None,
    };

    let err = app
        .execute_contract(owner.clone(), pair_instance.clone(), &msg, &[])
        .unwrap_err();
    assert_eq!(
        "It is not possible to provide liquidity with one token for an empty pool",
        err.root_cause().to_string()
    );

    let msg = ExecuteMsg::ProvideLiquidity {
        assets: vec![
            Asset {
                info: AssetInfo::Token(token_x_instance.to_string()),
                amount: Uint128::new(1_000_000_000_000_000),
            },
            Asset {
                info: AssetInfo::Token(token_y_instance.to_string()),
                amount: Uint128::new(1_000_000_000_000_000),
            },
        ],
        slippage_tolerance: None,
        receiver: None,
    };

    app.execute_contract(owner.clone(), pair_instance.clone(), &msg, &[])
        .unwrap();

    // try to provide for single token and increase the ratio in the pool from 1 to 1.5
    let msg = ExecuteMsg::ProvideLiquidity {
        assets: vec![
            Asset {
                info: AssetInfo::Token(token_x_instance.to_string()),
                amount: Uint128::new(500_000_000_000_000),
            },
            Asset {
                info: AssetInfo::Token(token_y_instance.to_string()),
                amount: Uint128::zero(),
            },
        ],
        slippage_tolerance: None,
        receiver: None,
    };

    app.execute_contract(owner.clone(), pair_instance.clone(), &msg, &[])
        .unwrap();

    // try swap 120_000_000 from token_y to token_x (from lower token amount to higher)
    app.execute_contract(owner.clone(), token_y_instance.clone(), &swap_msg, &[])
        .unwrap();

    // try swap 120_000_000 from token_x to token_y (from higher token amount to lower )
    app.execute_contract(owner.clone(), token_x_instance.clone(), &swap_msg, &[])
        .unwrap();

    // try to provide for single token and increase the ratio in the pool from 1 to 2.5
    let msg = ExecuteMsg::ProvideLiquidity {
        assets: vec![
            Asset {
                info: AssetInfo::Token(token_x_instance.to_string()),
                amount: Uint128::new(1_000_000_000_000_000),
            },
            Asset {
                info: AssetInfo::Token(token_y_instance.to_string()),
                amount: Uint128::zero(),
            },
        ],
        slippage_tolerance: None,
        receiver: None,
    };

    app.execute_contract(owner.clone(), pair_instance.clone(), &msg, &[])
        .unwrap();

    // try swap 120_000_000 from token_y to token_x (from lower token amount to higher)
    let msg = Cw20ExecuteMsg::Send {
        contract: pair_instance.to_string(),
        msg: to_binary(&Cw20HookMsg::Swap {
            ask_asset_info: None,
            belief_price: None,
            max_spread: None,
            to: None,
            referral_address: None,
            referral_commission: None,
        })
        .unwrap(),
        amount: swap_amount,
    };

    app.execute_contract(owner.clone(), token_y_instance, &msg, &[])
        .unwrap();

    // try swap 120_000_000 from token_x to token_y (from higher token amount to lower )
    let msg = Cw20ExecuteMsg::Send {
        contract: pair_instance.to_string(),
        msg: to_binary(&Cw20HookMsg::Swap {
            ask_asset_info: None,
            belief_price: None,
            max_spread: None,
            to: None,
            referral_address: None,
            referral_commission: None,
        })
        .unwrap(),
        amount: swap_amount,
    };

    let err = app
        .execute_contract(owner, token_x_instance, &msg, &[])
        .unwrap_err();
    assert_eq!(
        err.root_cause().to_string(),
        "Operation exceeds max spread limit"
    );
}

#[test]
fn test_compatibility_of_tokens_with_different_precision() {
    let owner = Addr::unchecked(OWNER);
    let mut app = mock_app(
        owner.clone(),
        vec![
            Coin {
                denom: "uusd".to_string(),
                amount: Uint128::new(100_000_000_000u128),
            },
            Coin {
                denom: "uluna".to_string(),
                amount: Uint128::new(100_000_000_000u128),
            },
        ],
    );

    let token_code_id = store_token_code(&mut app);

    let x_amount = Uint128::new(100_000_000_000);
    let y_amount = Uint128::new(1000000_0000000);
    let x_offer = Uint128::new(1_00000);
    let y_expected_return = Uint128::new(1_0000000);

    let token_name = "Xtoken";

    let init_msg = TokenInstantiateMsg {
        name: token_name.to_string(),
        symbol: token_name.to_string(),
        decimals: 5,
        initial_balances: vec![Cw20Coin {
            address: OWNER.to_string(),
            amount: x_amount + x_offer,
        }],
        mint: Some(MinterResponse {
            minter: String::from(OWNER),
            cap: None,
        }),
        marketing: None,
    };

    let token_x_instance = app
        .instantiate_contract(
            token_code_id,
            owner.clone(),
            &init_msg,
            &[],
            token_name,
            None,
        )
        .unwrap();

    let token_name = "Ytoken";

    let init_msg = TokenInstantiateMsg {
        name: token_name.to_string(),
        symbol: token_name.to_string(),
        decimals: 7,
        initial_balances: vec![Cw20Coin {
            address: OWNER.to_string(),
            amount: y_amount,
        }],
        mint: Some(MinterResponse {
            minter: String::from(OWNER),
            cap: None,
        }),
        marketing: None,
    };

    let token_y_instance = app
        .instantiate_contract(
            token_code_id,
            owner.clone(),
            &init_msg,
            &[],
            token_name,
            None,
        )
        .unwrap();

    let pair_code_id = store_pair_code(&mut app);
    let factory_code_id = store_factory_code(&mut app);
    let stake_code_id = store_stake_code(&mut app);

    let init_msg = FactoryInstantiateMsg {
        fee_address: None,
        pair_configs: vec![PairConfig {
            code_id: pair_code_id,
            fee_config: FeeConfig {
                protocol_fee_bps: 0,
                total_fee_bps: 0,
            },
            pair_type: PairType::Lsd {},
            is_disabled: false,
        }],
        token_code_id,
        owner: String::from("owner0000"),
        max_referral_commission: Decimal::one(),
        default_stake_config: default_stake_config(stake_code_id),
        trading_starts: None,
    };

    let factory_instance = app
        .instantiate_contract(
            factory_code_id,
            owner.clone(),
            &init_msg,
            &[],
            "FACTORY",
            None,
        )
        .unwrap();

    let msg = FactoryExecuteMsg::CreatePair {
        pair_type: PairType::Lsd {},
        asset_infos: vec![
            AssetInfo::Token(token_x_instance.to_string()),
            AssetInfo::Token(token_y_instance.to_string()),
        ],
        init_params: Some(
            to_binary(&StablePoolParams {
                amp: 100,
                owner: None,
                lsd: None,
            })
            .unwrap(),
        ),
        staking_config: PartialStakeConfig::default(),
        total_fee_bps: None,
    };

    app.execute_contract(
        Addr::unchecked("owner0000"),
        factory_instance.clone(),
        &msg,
        &[],
    )
    .unwrap();

    let msg = FactoryQueryMsg::Pair {
        asset_infos: vec![
            AssetInfo::Token(token_x_instance.to_string()),
            AssetInfo::Token(token_y_instance.to_string()),
        ],
    };

    let res: PairInfo = app
        .wrap()
        .query_wasm_smart(&factory_instance, &msg)
        .unwrap();

    let pair_instance = res.contract_addr;

    let msg = Cw20ExecuteMsg::IncreaseAllowance {
        spender: pair_instance.to_string(),
        expires: None,
        amount: x_amount + x_offer,
    };

    app.execute_contract(owner.clone(), token_x_instance.clone(), &msg, &[])
        .unwrap();

    let msg = Cw20ExecuteMsg::IncreaseAllowance {
        spender: pair_instance.to_string(),
        expires: None,
        amount: y_amount,
    };

    app.execute_contract(owner.clone(), token_y_instance.clone(), &msg, &[])
        .unwrap();

    let msg = ExecuteMsg::ProvideLiquidity {
        assets: vec![
            Asset {
                info: AssetInfo::Token(token_x_instance.to_string()),
                amount: x_amount,
            },
            Asset {
                info: AssetInfo::Token(token_y_instance.to_string()),
                amount: y_amount,
            },
        ],
        slippage_tolerance: None,
        receiver: None,
    };

    app.execute_contract(owner.clone(), pair_instance.clone(), &msg, &[])
        .unwrap();

    let d: u128 = app
        .wrap()
        .query_wasm_smart(&pair_instance, &QueryMsg::QueryComputeD {})
        .unwrap();
    assert_eq!(d, 20000000000000);

    let user = Addr::unchecked("user");

    let msg = Cw20ExecuteMsg::Send {
        contract: pair_instance.to_string(),
        msg: to_binary(&Cw20HookMsg::Swap {
            ask_asset_info: None,
            belief_price: None,
            max_spread: None,
            to: Some(user.to_string()),
            referral_address: None,
            referral_commission: None,
        })
        .unwrap(),
        amount: x_offer,
    };

    app.execute_contract(owner, token_x_instance, &msg, &[])
        .unwrap();

    let msg = Cw20QueryMsg::Balance {
        address: user.to_string(),
    };

    let res: BalanceResponse = app
        .wrap()
        .query_wasm_smart(&token_y_instance, &msg)
        .unwrap();

    assert_eq!(res.balance, y_expected_return);

    let d: u128 = app
        .wrap()
        .query_wasm_smart(&pair_instance, &QueryMsg::QueryComputeD {})
        .unwrap();
    assert_eq!(d, 19999999999999);
}

#[test]
fn test_if_twap_is_calculated_correctly_when_pool_idles() {
    let owner = Addr::unchecked(OWNER);
    let mut app = mock_app(
        owner.clone(),
        vec![
            Coin {
                denom: "uusd".to_string(),
                amount: Uint128::new(100_000_000_000_000_u128),
            },
            Coin {
                denom: "uluna".to_string(),
                amount: Uint128::new(100_000_000_000_000_u128),
            },
        ],
    );

    let user1 = Addr::unchecked("user1");

    // Set User1's balances
    app.send_tokens(
        owner,
        user1.clone(),
        &[
            Coin {
                denom: "uusd".to_string(),
                amount: Uint128::new(4_666_666_000_000),
            },
            Coin {
                denom: "uluna".to_string(),
                amount: Uint128::new(2_000_000_000_000),
            },
        ],
    )
    .unwrap();

    // Instantiate pair
    let pair_instance = instantiate_pair(&mut app, &user1);

    // Provide liquidity, accumulators are empty
    let (msg, coins) = provide_liquidity_msg(
        Uint128::new(1_000_000_000_000),
        Uint128::new(1_000_000_000_000),
        None,
    );
    app.execute_contract(user1.clone(), pair_instance.clone(), &msg, &coins)
        .unwrap();

    const BLOCKS_PER_DAY: u64 = 17280;
    const ELAPSED_SECONDS: u64 = BLOCKS_PER_DAY * 5;

    // A day later
    app.update_block(|b| {
        b.height += BLOCKS_PER_DAY;
        b.time = b.time.plus_seconds(ELAPSED_SECONDS);
    });

    // Provide liquidity, accumulators firstly filled with the same prices
    let (msg, coins) = provide_liquidity_msg(
        Uint128::new(3_000_000_000_000),
        Uint128::new(1_000_000_000_000),
        None,
    );
    app.execute_contract(user1.clone(), pair_instance.clone(), &msg, &coins)
        .unwrap();

    // Get current TWAP accumulator values
    let msg = QueryMsg::CumulativePrices {};
    let cpr_old: CumulativePricesResponse =
        app.wrap().query_wasm_smart(&pair_instance, &msg).unwrap();

    // A day later
    app.update_block(|b| {
        b.height += BLOCKS_PER_DAY;
        b.time = b.time.plus_seconds(ELAPSED_SECONDS);
    });

    // Get current twap accumulator values
    let msg = QueryMsg::CumulativePrices {};
    let cpr_new: CumulativePricesResponse =
        app.wrap().query_wasm_smart(&pair_instance, &msg).unwrap();

    let twap0 = cpr_new.cumulative_prices[0].2 - cpr_old.cumulative_prices[0].2;
    let twap1 = cpr_new.cumulative_prices[1].2 - cpr_old.cumulative_prices[1].2;

    // Prices weren't changed for the last day, uusd amount in pool = 4000000_000000, uluna = 2000000_000000
    let price_precision = Uint128::from(10u128.pow(TWAP_PRECISION.into()));
    assert_eq!(twap0 / price_precision, Uint128::new(85684)); // 1.008356286 * ELAPSED_SECONDS (86400)
    assert_eq!(twap1 / price_precision, Uint128::new(87121)); // 0.991712963 * ELAPSED_SECONDS
}

#[test]
fn create_pair_with_same_assets() {
    let owner = Addr::unchecked(OWNER);
    let mut router = mock_app(
        owner.clone(),
        vec![
            Coin {
                denom: "uusd".to_string(),
                amount: Uint128::new(100_000_000_000u128),
            },
            Coin {
                denom: "uluna".to_string(),
                amount: Uint128::new(100_000_000_000u128),
            },
        ],
    );

    let token_contract_code_id = store_token_code(&mut router);
    let pair_contract_code_id = store_pair_code(&mut router);
    let stake_code_id = store_stake_code(&mut router);

    let msg = InstantiateMsg {
        asset_infos: vec![
            AssetInfo::Native("uusd".to_string()),
            AssetInfo::Native("uusd".to_string()),
        ],
        token_code_id: token_contract_code_id,
        factory_addr: String::from("factory"),
        init_params: None,
        staking_config: default_stake_config(stake_code_id).to_stake_config(),
        trading_starts: 0,
        fee_config: FeeConfig {
            total_fee_bps: 0,
            protocol_fee_bps: 0,
        },
        circuit_breaker: None,
    };

    let resp = router
        .instantiate_contract(
            pair_contract_code_id,
            owner,
            &msg,
            &[],
            String::from("PAIR"),
            None,
        )
        .unwrap_err();

    assert_eq!(
        resp.root_cause().to_string(),
        "Doubling assets in asset infos"
    )
}

#[test]
fn provide_liquidity_with_one_asset() {
    let owner = Addr::unchecked("owner");
    let mut router = mock_app(
        owner.clone(),
        vec![
            Coin {
                denom: "uusd".to_string(),
                amount: Uint128::new(100_000_100_000u128),
            },
            Coin {
                denom: "uluna".to_string(),
                amount: Uint128::new(100_000_000_000u128),
            },
        ],
    );

    let pair = instantiate_pair(&mut router, &owner);

    // first provide liquidity with two assets
    let (msg, coins) = provide_liquidity_msg(
        Uint128::from(100_000_000_000u128),
        Uint128::from(100_000_000_000u128),
        None,
    );
    router
        .execute_contract(owner.clone(), pair.clone(), &msg, &coins)
        .unwrap();

    let res: PoolResponse = router
        .wrap()
        .query_wasm_smart(pair.clone(), &QueryMsg::Pool {})
        .unwrap();
    assert_eq!(200_000_000_000, res.total_share.u128());

    // then with only one asset
    let msg = ExecuteMsg::ProvideLiquidity {
        assets: vec![AssetInfo::Native("uusd".to_string()).with_balance(100_000u128)],
        slippage_tolerance: None,
        receiver: None,
    };
    router
        .execute_contract(
            owner.clone(),
            pair.clone(),
            &msg,
            &[Coin {
                denom: "uusd".to_string(),
                amount: Uint128::new(100_000u128),
            }],
        )
        .unwrap();

    let res: PoolResponse = router
        .wrap()
        .query_wasm_smart(pair, &QueryMsg::Pool {})
        .unwrap();
    // should have 200_000_000_000 + 99_974 shares (losing some because of spread)
    assert_eq!(200_000_000_000 + 99_974, res.total_share.u128());
}

#[test]
fn provide_liquidity_sad_path() {
    let owner = Addr::unchecked("owner");
    let mut router = mock_app(
        owner.clone(),
        vec![
            Coin {
                denom: "uusd".to_string(),
                amount: Uint128::new(100_000_100_000u128),
            },
            Coin {
                denom: "uluna".to_string(),
                amount: Uint128::new(100_000_000_000u128),
            },
        ],
    );

    let pair = instantiate_pair(&mut router, &owner);

    // try with only one asset before any liquidity is there
    let msg = ExecuteMsg::ProvideLiquidity {
        assets: vec![AssetInfo::Native("uusd".to_string()).with_balance(100_000u128)],
        slippage_tolerance: None,
        receiver: None,
    };
    let err = router
        .execute_contract(
            owner.clone(),
            pair.clone(),
            &msg,
            &[Coin {
                denom: "uusd".to_string(),
                amount: Uint128::from(100_000u128),
            }],
        )
        .unwrap_err();
    assert_eq!(
        err.root_cause().to_string(),
        "It is not possible to provide liquidity with one token for an empty pool"
    );

    // provide liquidity with two assets
    let (msg, coins) = provide_liquidity_msg(
        Uint128::from(100_000_000_000u128),
        Uint128::from(100_000_000_000u128),
        None,
    );
    router
        .execute_contract(owner.clone(), pair.clone(), &msg, &coins)
        .unwrap();

    // try with 0 amount
    let msg = ExecuteMsg::ProvideLiquidity {
        assets: vec![AssetInfo::Native("uusd".to_string()).with_balance(0u128)],
        slippage_tolerance: None,
        receiver: None,
    };
    let err = router
        .execute_contract(owner.clone(), pair.clone(), &msg, &[])
        .unwrap_err();

    assert_eq!(err.root_cause().to_string(), "Event of zero transfer");

    // try with empty assets
    let msg = ExecuteMsg::ProvideLiquidity {
        assets: vec![],
        slippage_tolerance: None,
        receiver: None,
    };

    let err = router
        .execute_contract(owner.clone(), pair, &msg, &[])
        .unwrap_err();

    assert_eq!(err.root_cause().to_string(), "Event of zero transfer");
}

#[test]
fn provide_liquidity_with_one_cw20_asset() {
    let owner = Addr::unchecked("owner");
    let mut router = mock_app(owner.clone(), vec![]);

    let token1 = instantiate_token(
        &mut router,
        &owner,
        &[(owner.as_str(), 100_000_000_000u128)],
    );
    let token2 = instantiate_token(
        &mut router,
        &owner,
        &[(owner.as_str(), 100_000_100_000u128)],
    );

    let factory = instantiate_factory(&mut router, &owner);
    // create pair
    let asset_infos = vec![
        AssetInfo::Token(token1.to_string()),
        AssetInfo::Token(token2.to_string()),
    ];
    let msg = FactoryExecuteMsg::CreatePair {
        pair_type: PairType::Lsd {},
        asset_infos: asset_infos.clone(),
        init_params: Some(
            to_binary(&StablePoolParams {
                amp: 100,
                owner: None,
                lsd: None,
            })
            .unwrap(),
        ),
        total_fee_bps: None,
        staking_config: PartialStakeConfig::default(),
    };
    router
        .execute_contract(owner.clone(), factory.clone(), &msg, &[])
        .unwrap();
    // get pair address
    let pair = router
        .wrap()
        .query_wasm_smart::<PairInfo>(factory, &FactoryQueryMsg::Pair { asset_infos })
        .unwrap()
        .contract_addr;

    // increase allowances
    router
        .execute_contract(
            owner.clone(),
            token1.clone(),
            &Cw20ExecuteMsg::IncreaseAllowance {
                spender: pair.to_string(),
                expires: None,
                amount: 100_000_000_000u128.into(),
            },
            &[],
        )
        .unwrap();
    router
        .execute_contract(
            owner.clone(),
            token2.clone(),
            &Cw20ExecuteMsg::IncreaseAllowance {
                spender: pair.to_string(),
                expires: None,
                amount: 100_000_100_000u128.into(),
            },
            &[],
        )
        .unwrap();

    // first provide liquidity with two assets
    let msg = ExecuteMsg::ProvideLiquidity {
        assets: vec![
            AssetInfo::Token(token1.to_string()).with_balance(100_000_000_000u128),
            AssetInfo::Token(token2.to_string()).with_balance(100_000_000_000u128),
        ],
        slippage_tolerance: None,
        receiver: None,
    };
    router
        .execute_contract(owner.clone(), pair.clone(), &msg, &[])
        .unwrap();

    // then with only one asset
    let msg = ExecuteMsg::ProvideLiquidity {
        assets: vec![AssetInfo::Token(token2.to_string()).with_balance(100_000u128)],
        slippage_tolerance: None,
        receiver: None,
    };
    router
        .execute_contract(owner.clone(), pair.clone(), &msg, &[])
        .unwrap();

    // should have no more balance of token2
    assert_eq!(
        query_token_balance(&router.wrap(), token2, owner)
            .unwrap()
            .u128(),
        0
    );
    let res: PoolResponse = router
        .wrap()
        .query_wasm_smart(pair, &QueryMsg::Pool {})
        .unwrap();
    // should have 200_000_000_000 + 99_974 shares (losing some because of spread)
    assert_eq!(200_000_000_000 + 99_974, res.total_share.u128());
}

#[test]
fn swap_with_referral() {
    let owner = Addr::unchecked(OWNER);
    let referral = "referral".to_string();

    let mut app = mock_app(
        owner.clone(),
        vec![
            Coin {
                denom: "uusd".to_string(),
                amount: Uint128::new(100_000_000_000u128),
            },
            Coin {
                denom: "uluna".to_string(),
                amount: Uint128::new(100_000_000_100u128),
            },
        ],
    );

    // Init pair
    let pair_instance = instantiate_pair(&mut app, &owner);

    // Provide liquidity
    let (msg, coins) = provide_liquidity_msg(
        Uint128::new(100_000_000_000u128),
        Uint128::new(100_000_000_000u128),
        None,
    );
    app.execute_contract(owner.clone(), pair_instance.clone(), &msg, &coins)
        .unwrap();

    app.execute_contract(
        owner.clone(),
        pair_instance,
        &ExecuteMsg::Swap {
            offer_asset: Asset {
                amount: 100u128.into(),
                info: AssetInfo::Native("uluna".to_string()),
            },
            ask_asset_info: Some(AssetInfo::Native("uusd".to_string())),
            belief_price: None,
            max_spread: None,
            to: None,
            referral_address: Some(referral.clone()),
            referral_commission: Some(Decimal::percent(1)),
        },
        &[Coin::new(100, "uluna")],
    )
    .unwrap();

    // assert referral commission
    assert_eq!(
        1,
        app.wrap()
            .query_balance(referral, "uluna")
            .unwrap()
            .amount
            .u128()
    );
}

#[test]
fn update_pair_config() {
    let owner = Addr::unchecked(OWNER);
    let mut router = mock_app(
        owner.clone(),
        vec![
            Coin {
                denom: "uusd".to_string(),
                amount: Uint128::new(100_000_000_000u128),
            },
            Coin {
                denom: "uluna".to_string(),
                amount: Uint128::new(100_000_000_000u128),
            },
        ],
    );

    let token_contract_code_id = store_token_code(&mut router);
    let pair_contract_code_id = store_pair_code(&mut router);

    let factory_code_id = store_factory_code(&mut router);
    let stake_code_id = store_stake_code(&mut router);

    let init_msg = FactoryInstantiateMsg {
        fee_address: None,
        pair_configs: vec![],
        token_code_id: token_contract_code_id,
        owner: owner.to_string(),
        max_referral_commission: Decimal::one(),
        default_stake_config: default_stake_config(stake_code_id),
        trading_starts: None,
    };

    let factory_instance = router
        .instantiate_contract(
            factory_code_id,
            owner.clone(),
            &init_msg,
            &[],
            "FACTORY",
            None,
        )
        .unwrap();

    let msg = InstantiateMsg {
        asset_infos: vec![
            AssetInfo::Native("uusd".to_string()),
            AssetInfo::Native("uluna".to_string()),
        ],
        token_code_id: token_contract_code_id,
        factory_addr: factory_instance.to_string(),
        init_params: Some(
            to_binary(&StablePoolParams {
                amp: 100,
                owner: None,
                lsd: None,
            })
            .unwrap(),
        ),
        staking_config: default_stake_config(stake_code_id).to_stake_config(),
        trading_starts: 0,
        fee_config: FeeConfig {
            protocol_fee_bps: 0,
            total_fee_bps: 0,
        },
        circuit_breaker: None,
    };

    let pair = router
        .instantiate_contract(
            pair_contract_code_id,
            owner.clone(),
            &msg,
            &[],
            String::from("PAIR"),
            None,
        )
        .unwrap();

    let res: ConfigResponse = router
        .wrap()
        .query_wasm_smart(pair.clone(), &QueryMsg::Config {})
        .unwrap();

    let params: StablePoolConfig = from_binary(&res.params.unwrap()).unwrap();

    assert_eq!(params.amp, Decimal::from_ratio(100u32, 1u32));

    // Start changing amp with incorrect next amp
    let msg = ExecuteMsg::UpdateConfig {
        params: to_binary(&StablePoolUpdateParams::StartChangingAmp {
            next_amp: MAX_AMP + 1,
            next_amp_time: router.block_info().time.seconds(),
        })
        .unwrap(),
    };

    let resp = router
        .execute_contract(owner.clone(), pair.clone(), &msg, &[])
        .unwrap_err();

    assert_eq!(
        resp.root_cause().to_string(),
        format!(
            "Amp coefficient must be greater than 0 and less than or equal to {}",
            MAX_AMP
        )
    );

    // Start changing amp with big difference between the old and new amp value
    let msg = ExecuteMsg::UpdateConfig {
        params: to_binary(&StablePoolUpdateParams::StartChangingAmp {
            next_amp: 100 * MAX_AMP_CHANGE + 1,
            next_amp_time: router.block_info().time.seconds(),
        })
        .unwrap(),
    };

    let resp = router
        .execute_contract(owner.clone(), pair.clone(), &msg, &[])
        .unwrap_err();

    assert_eq!(
        resp.root_cause().to_string(),
        format!(
            "The difference between the old and new amp value must not exceed {} times",
            MAX_AMP_CHANGE
        )
    );

    // Start changing amp before the MIN_AMP_CHANGING_TIME has elapsed
    let msg = ExecuteMsg::UpdateConfig {
        params: to_binary(&StablePoolUpdateParams::StartChangingAmp {
            next_amp: 250,
            next_amp_time: router.block_info().time.seconds(),
        })
        .unwrap(),
    };

    let resp = router
        .execute_contract(owner.clone(), pair.clone(), &msg, &[])
        .unwrap_err();

    assert_eq!(
        resp.root_cause().to_string(),
        format!(
            "Amp coefficient cannot be changed more often than once per {} seconds",
            MIN_AMP_CHANGING_TIME
        )
    );

    // Start increasing amp
    router.update_block(|b| {
        b.time = b.time.plus_seconds(MIN_AMP_CHANGING_TIME);
    });

    let msg = ExecuteMsg::UpdateConfig {
        params: to_binary(&StablePoolUpdateParams::StartChangingAmp {
            next_amp: 250,
            next_amp_time: router.block_info().time.seconds() + MIN_AMP_CHANGING_TIME,
        })
        .unwrap(),
    };

    router
        .execute_contract(owner.clone(), pair.clone(), &msg, &[])
        .unwrap();

    router.update_block(|b| {
        b.time = b.time.plus_seconds(MIN_AMP_CHANGING_TIME / 2);
    });

    let res: ConfigResponse = router
        .wrap()
        .query_wasm_smart(pair.clone(), &QueryMsg::Config {})
        .unwrap();

    let params: StablePoolConfig = from_binary(&res.params.unwrap()).unwrap();

    assert_eq!(params.amp, Decimal::from_ratio(175u32, 1u32));

    router.update_block(|b| {
        b.time = b.time.plus_seconds(MIN_AMP_CHANGING_TIME / 2);
    });

    let res: ConfigResponse = router
        .wrap()
        .query_wasm_smart(pair.clone(), &QueryMsg::Config {})
        .unwrap();

    let params: StablePoolConfig = from_binary(&res.params.unwrap()).unwrap();

    assert_eq!(params.amp, Decimal::from_ratio(250u32, 1u32));

    // Start decreasing amp
    router.update_block(|b| {
        b.time = b.time.plus_seconds(MIN_AMP_CHANGING_TIME);
    });

    let msg = ExecuteMsg::UpdateConfig {
        params: to_binary(&StablePoolUpdateParams::StartChangingAmp {
            next_amp: 50,
            next_amp_time: router.block_info().time.seconds() + MIN_AMP_CHANGING_TIME,
        })
        .unwrap(),
    };

    router
        .execute_contract(owner.clone(), pair.clone(), &msg, &[])
        .unwrap();

    router.update_block(|b| {
        b.time = b.time.plus_seconds(MIN_AMP_CHANGING_TIME / 2);
    });

    let res: ConfigResponse = router
        .wrap()
        .query_wasm_smart(pair.clone(), &QueryMsg::Config {})
        .unwrap();

    let params: StablePoolConfig = from_binary(&res.params.unwrap()).unwrap();

    assert_eq!(params.amp, Decimal::from_ratio(150u32, 1u32));

    // Stop changing amp
    let msg = ExecuteMsg::UpdateConfig {
        params: to_binary(&StablePoolUpdateParams::StopChangingAmp {}).unwrap(),
    };

    router
        .execute_contract(owner, pair.clone(), &msg, &[])
        .unwrap();

    router.update_block(|b| {
        b.time = b.time.plus_seconds(MIN_AMP_CHANGING_TIME / 2);
    });

    let res: ConfigResponse = router
        .wrap()
        .query_wasm_smart(pair, &QueryMsg::Config {})
        .unwrap();

    let params: StablePoolConfig = from_binary(&res.params.unwrap()).unwrap();

    assert_eq!(params.amp, Decimal::from_ratio(150u32, 1u32));
}

// Integration test showing the incorrect behaviour:
#[test]
fn test_mixed_twap_calculation() {
    let owner = Addr::unchecked("owner");
    let user1 = Addr::unchecked("user1");

    let mut app = mock_app(
        owner.clone(),
        vec![Coin {
            denom: "uusd".to_string(),
            amount: Uint128::new(1_000_000_000_000_000),
        }],
    );

    // Instantiate pair
    let (pair_instance, cw20_token) = instantiate_mixed_pair(
        &mut app,
        &user1,
        &[(user1.as_str(), 1_000_000_000_000_000u128)],
    );

    // Set Alice's balances
    app.send_tokens(
        owner,
        user1.clone(),
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::new(1_000_000_000_000_000),
        }],
    )
    .unwrap();

    // Set allowance for the pair contract to take tokens from Alice
    let msg = Cw20ExecuteMsg::IncreaseAllowance {
        spender: pair_instance.to_string(),
        amount: Uint128::new(1_000_000_000_000_000),
        expires: None,
    };
    app.execute_contract(user1.clone(), cw20_token.clone(), &msg, &[])
        .unwrap();

    // Provide liquidity, accumulators are empty (because the cw20 pool registers as empty)
    let (msg, coins) = provide_liquidity_mixed_msg(
        Uint128::new(1_000_000_000_000),
        Uint128::new(1_000_000_000_000),
        &cw20_token,
        None,
        Option::from(Decimal::one()),
    );
    app.execute_contract(user1.clone(), pair_instance.clone(), &msg, &coins)
        .unwrap();

    const BLOCKS_PER_DAY: u64 = 17280;
    const ELAPSED_SECONDS: u64 = BLOCKS_PER_DAY * 5;

    // A day later
    app.update_block(|b| {
        b.height += BLOCKS_PER_DAY;
        b.time = b.time.plus_seconds(ELAPSED_SECONDS);
    });

    // Provide liquidity, accumulators firstly filled
    // They *should* be filled with the price from the previous call (1.000000),
    // but are actually filled with prices differing from that, because the pools are calculated as:
    // uusd: 10_000_000_000_000 + 1_000_000_000_000, cw20: 1_000_000_000_000
    // The 10_000_000_000_000 added by this call should be subtracted from the uusd pool, but they are not.
    let (msg, coins) = provide_liquidity_mixed_msg(
        Uint128::new(10_000_000_000_000),
        Uint128::new(10_000_000_000_000),
        &cw20_token,
        None,
        Some(Decimal::percent(50)),
    );
    app.execute_contract(user1.clone(), pair_instance.clone(), &msg, &coins)
        .unwrap();

    // Get current twap accumulator values
    let msg = QueryMsg::CumulativePrices {};
    let cpr_old: CumulativePricesResponse =
        app.wrap().query_wasm_smart(&pair_instance, &msg).unwrap();

    // A day later
    app.update_block(|b| {
        b.height += BLOCKS_PER_DAY;
        b.time = b.time.plus_seconds(ELAPSED_SECONDS);
    });

    // Provide liquidity a second time
    // Accumulator *should* be filled with the price from the previous call (1.000000),
    // but are actually filled with prices differing from that, because the pools are calculated as:
    // uusd: 10_000_000_000_000 + 11_000_000_000_000, cw20: 11_000_000_000_000
    // The 10_000_000_000_000 added by this call should be subtracted from the uusd pool, but they are not.
    let (msg, coins) = provide_liquidity_mixed_msg(
        Uint128::new(10_000_000_000_000),
        Uint128::new(10_000_000_000_000),
        &cw20_token,
        None,
        Some(Decimal::percent(50)),
    );
    app.execute_contract(user1.clone(), pair_instance.clone(), &msg, &coins)
        .unwrap();

    // Get current twap accumulator values
    let msg = QueryMsg::CumulativePrices {};
    let cpr_new: CumulativePricesResponse =
        app.wrap().query_wasm_smart(&pair_instance, &msg).unwrap();

    let twap0 = cpr_new.cumulative_prices[0].2 - cpr_old.cumulative_prices[0].2;
    let twap1 = cpr_new.cumulative_prices[1].2 - cpr_old.cumulative_prices[1].2;

    let price_precision = Uint128::from(10u128.pow(TWAP_PRECISION.into()));
    assert_eq!(twap0 / price_precision, Uint128::new(86400)); // expecting: 1.0 * ELAPSED_SECONDS (86400)
    assert_eq!(twap1 / price_precision, Uint128::new(86400)); // expecting: 1.0 * ELAPSED_SECONDS
}
