use cosmwasm_std::{attr, to_binary, Addr, Coin, Decimal, Uint128};
use cw20::{BalanceResponse, Cw20Coin, Cw20ExecuteMsg, Cw20QueryMsg, MinterResponse};
use cw20_base::msg::InstantiateMsg as TokenInstantiateMsg;
use cw_multi_test::{App, ContractWrapper, Executor};
use wyndex::asset::{native_asset_info, Asset, AssetInfo, AssetInfoExt, AssetInfoValidated};
use wyndex::factory::{
    DefaultStakeConfig, ExecuteMsg as FactoryExecuteMsg, InstantiateMsg as FactoryInstantiateMsg,
    PairConfig, PairType, PartialStakeConfig, QueryMsg as FactoryQueryMsg,
};
use wyndex::fee_config::FeeConfig;
use wyndex::pair::{
    ConfigResponse, CumulativePricesResponse, Cw20HookMsg, ExecuteMsg, InstantiateMsg, PairInfo,
    PoolResponse, QueryMsg, SimulationResponse, TWAP_PRECISION,
};
use wyndex::querier::query_token_balance;

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
            wyndex_pair::contract::execute,
            wyndex_pair::contract::instantiate,
            wyndex_pair::contract::query,
        )
        .with_reply_empty(wyndex_pair::contract::reply),
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

fn store_staking_code(app: &mut App) -> u64 {
    let stake_contract = Box::new(ContractWrapper::new_with_empty(
        wyndex_stake::contract::execute,
        wyndex_stake::contract::instantiate,
        wyndex_stake::contract::query,
    ));

    app.store_code(stake_contract)
}

fn instantiate_factory(router: &mut App, owner: &Addr) -> Addr {
    let token_contract_code_id = store_token_code(router);
    let pair_contract_code_id = store_pair_code(router);
    let staking_contract_code_id = store_staking_code(router);
    let factory_contract_code_id = store_factory_code(router);

    let msg = FactoryInstantiateMsg {
        pair_configs: vec![PairConfig {
            pair_type: PairType::Xyk {},
            code_id: pair_contract_code_id,
            fee_config: FeeConfig {
                total_fee_bps: 0,
                protocol_fee_bps: 0,
            },
            is_disabled: false,
        }],
        token_code_id: token_contract_code_id,
        fee_address: Some(owner.to_string()),
        owner: owner.to_string(),
        max_referral_commission: Decimal::one(),
        default_stake_config: default_stake_config(staking_contract_code_id),
        trading_starts: None,
    };

    router
        .instantiate_contract(
            factory_contract_code_id,
            owner.clone(),
            &msg,
            &[],
            String::from("FACTORY"),
            None,
        )
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
        pair_type: PairType::Xyk {},
        asset_infos: asset_infos.clone(),
        init_params: None,
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
        pair_type: PairType::Xyk {},
        asset_infos: asset_infos.clone(),
        init_params: None,
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
            Coin {
                denom: "cny".to_string(),
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
                    amount: Uint128::new(233_000_000u128),
                },
                Coin {
                    denom: "uluna".to_string(),
                    amount: Uint128::new(200_000_000u128),
                },
                Coin {
                    denom: "cny".to_string(),
                    amount: Uint128::from(100_000_000u128),
                },
            ],
        )
        .unwrap();

    // Init pair
    let pair_instance = instantiate_pair(&mut router, &owner);

    let res: PairInfo = router
        .wrap()
        .query_wasm_smart(pair_instance.to_string(), &QueryMsg::Pair {})
        .unwrap();
    let lp_token = res.liquidity_token;

    assert_eq!(
        res.asset_infos,
        [
            AssetInfoValidated::Native("uusd".to_string()),
            AssetInfoValidated::Native("uluna".to_string()),
        ],
    );

    // When dealing with native tokens the transfer should happen before the contract call, which cw-multitest doesn't support
    // Set Alice's balances
    router
        .send_tokens(
            owner.clone(),
            pair_instance.clone(),
            &[
                Coin {
                    denom: "uusd".to_string(),
                    amount: Uint128::new(100_000_000u128),
                },
                Coin {
                    denom: "uluna".to_string(),
                    amount: Uint128::new(100_000_000u128),
                },
            ],
        )
        .unwrap();

    // Provide liquidity
    let (msg, coins) = provide_liquidity_msg(
        Uint128::new(100_000_000),
        Uint128::new(100_000_000),
        None,
        None,
    );
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
        attr("assets", "100000000uusd, 100000000uluna")
    );
    assert_eq!(
        res.events[1].attributes[5],
        attr("share", 99999000u128.to_string())
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
        attr("amount", 99999000.to_string())
    );

    // Provide liquidity for receiver
    let (msg, coins) = provide_liquidity_msg(
        Uint128::new(100),
        Uint128::new(100),
        Some("bob".to_string()),
        None,
    );
    let res = router
        .execute_contract(alice_address.clone(), pair_instance.clone(), &msg, &coins)
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
        attr("share", 50u128.to_string())
    );
    assert_eq!(res.events[3].attributes[1], attr("action", "mint"));
    assert_eq!(res.events[3].attributes[2], attr("to", "bob"));
    assert_eq!(res.events[3].attributes[3], attr("amount", 50.to_string()));

    // Checking withdraw liquidity
    let token_contract_code_id = store_token_code(&mut router);
    let foo_token = router
        .instantiate_contract(
            token_contract_code_id,
            owner.clone(),
            &TokenInstantiateMsg {
                name: "Foo token".to_string(),
                symbol: "FOO".to_string(),
                decimals: 6,
                initial_balances: vec![Cw20Coin {
                    address: alice_address.to_string(),
                    amount: Uint128::from(1000000000u128),
                }],
                mint: None,
                marketing: None,
            },
            &[],
            String::from("FOO"),
            None,
        )
        .unwrap();

    let msg = Cw20ExecuteMsg::Send {
        contract: pair_instance.to_string(),
        amount: Uint128::from(50u8),
        msg: to_binary(&Cw20HookMsg::WithdrawLiquidity { assets: vec![] }).unwrap(),
    };
    // Try to send withdraw liquidity with FOO token
    let err = router
        .execute_contract(alice_address.clone(), foo_token, &msg, &[])
        .unwrap_err();
    assert_eq!(err.root_cause().to_string(), "Unauthorized");
    // Withdraw with LP token is successful
    router
        .execute_contract(alice_address.clone(), lp_token, &msg, &[])
        .unwrap();

    let err = router
        .execute_contract(
            alice_address,
            pair_instance.clone(),
            &ExecuteMsg::Swap {
                offer_asset: Asset {
                    info: AssetInfo::Native("cny".to_string()),
                    amount: Uint128::from(10u8),
                },
                ask_asset_info: None,
                belief_price: None,
                max_spread: None,
                to: None,
                referral_address: None,
                referral_commission: None,
            },
            &[Coin {
                denom: "cny".to_string(),
                amount: Uint128::from(10u8),
            }],
        )
        .unwrap_err();
    assert_eq!(
        err.root_cause().to_string(),
        "Asset mismatch between the requested and the stored asset in contract"
    );

    // Check pair config
    let config: ConfigResponse = router
        .wrap()
        .query_wasm_smart(pair_instance.to_string(), &QueryMsg::Config {})
        .unwrap();
    assert_eq!(
        config,
        ConfigResponse {
            block_time_last: router.block_info().time.seconds(),
            params: None,
            owner: None
        }
    )
}

fn provide_liquidity_msg(
    uusd_amount: Uint128,
    uluna_amount: Uint128,
    receiver: Option<String>,
    slippage_tolerance: Option<Decimal>,
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
        slippage_tolerance,
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
fn test_compatibility_of_tokens_with_different_precision() {
    let owner = Addr::unchecked(OWNER);

    let mut app = mock_app(
        owner.clone(),
        vec![
            Coin {
                denom: "uusd".to_string(),
                amount: Uint128::new(100_000_000_000_000u128),
            },
            Coin {
                denom: "uluna".to_string(),
                amount: Uint128::new(100_000_000_000_000u128),
            },
        ],
    );

    let token_code_id = store_token_code(&mut app);

    let x_amount = Uint128::new(100_000_000_000);
    let y_amount = Uint128::new(10_000_000_000_000);
    let x_offer = Uint128::new(100_000);
    let y_expected_return = Uint128::new(10_000_000);

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
    let staking_code_id = store_staking_code(&mut app);

    let init_msg = FactoryInstantiateMsg {
        fee_address: None,
        pair_configs: vec![PairConfig {
            code_id: pair_code_id,
            pair_type: PairType::Xyk {},
            fee_config: FeeConfig {
                total_fee_bps: 0,
                protocol_fee_bps: 0,
            },
            is_disabled: false,
        }],
        token_code_id,
        owner: owner.to_string(),
        max_referral_commission: Decimal::one(),
        default_stake_config: default_stake_config(staking_code_id),
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
        asset_infos: vec![
            AssetInfo::Token(token_x_instance.to_string()),
            AssetInfo::Token(token_y_instance.to_string()),
        ],
        pair_type: PairType::Xyk {},
        init_params: None,
        staking_config: PartialStakeConfig::default(),
        total_fee_bps: None,
    };

    app.execute_contract(owner.clone(), factory_instance.clone(), &msg, &[])
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

    let user = Addr::unchecked("user");

    let swap_msg = Cw20ExecuteMsg::Send {
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

    app.execute_contract(owner.clone(), pair_instance, &msg, &[])
        .unwrap();

    // try to swap after provide liquidity
    app.execute_contract(owner, token_x_instance, &swap_msg, &[])
        .unwrap();

    let msg = Cw20QueryMsg::Balance {
        address: user.to_string(),
    };

    let res: BalanceResponse = app
        .wrap()
        .query_wasm_smart(&token_y_instance, &msg)
        .unwrap();

    let acceptable_spread_amount = Uint128::new(10);

    assert_eq!(res.balance, y_expected_return - acceptable_spread_amount);
}

#[test]
fn test_if_twap_is_calculated_correctly_when_pool_idles() {
    let owner = Addr::unchecked("owner");
    let user1 = Addr::unchecked("user1");

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

    // Set Alice's balances
    app.send_tokens(
        owner,
        user1.clone(),
        &[
            Coin {
                denom: "uusd".to_string(),
                amount: Uint128::new(4_000_000_000_000),
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

    // Provide liquidity, accumulators firstly filled with the same prices
    let (msg, coins) = provide_liquidity_msg(
        Uint128::new(2_000_000_000_000),
        Uint128::new(1_000_000_000_000),
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

    // Get current cumulative price values; they should have been updated by the query method with new 2/1 ratio
    let msg = QueryMsg::CumulativePrices {};
    let cpr_new: CumulativePricesResponse =
        app.wrap().query_wasm_smart(&pair_instance, &msg).unwrap();

    let twap0 = cpr_new.cumulative_prices[0].2 - cpr_old.cumulative_prices[0].2;
    let twap1 = cpr_new.cumulative_prices[1].2 - cpr_old.cumulative_prices[1].2;

    // Prices weren't changed for the last day, uusd amount in pool = 3000000_000000, uluna = 2000000_000000
    // In accumulators we don't have any precision so we rely on elapsed time so we don't need to consider it
    let price_precision = Uint128::from(10u128.pow(TWAP_PRECISION.into()));
    assert_eq!(twap0 / price_precision, Uint128::new(57600)); // 0.666666 * ELAPSED_SECONDS (86400)
    assert_eq!(twap1 / price_precision, Uint128::new(129600)); //   1.5 * ELAPSED_SECONDS
}

#[test]
fn create_pair_with_same_assets() {
    let owner = Addr::unchecked("owner");
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
    let staking_contract_code_id = store_staking_code(&mut router);

    let msg = InstantiateMsg {
        asset_infos: vec![
            AssetInfo::Native("uusd".to_string()),
            AssetInfo::Native("uusd".to_string()),
        ],
        token_code_id: token_contract_code_id,
        factory_addr: String::from("factory"),
        init_params: None,
        staking_config: default_stake_config(staking_contract_code_id).to_stake_config(),
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
        None,
    );
    router
        .execute_contract(owner.clone(), pair.clone(), &msg, &coins)
        .unwrap();

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
    // should have 100_000_000_000 + 49999 shares (losing some because of spread)
    assert_eq!(100_000_000_000 + 49_999, res.total_share.u128());
}

#[test]
fn provide_liquidity_with_swap() {
    // This is more of a reference implementation to compare `provide_liquidity_with_one_asset` to.
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
        None,
    );
    router
        .execute_contract(owner.clone(), pair.clone(), &msg, &coins)
        .unwrap();

    // now swap half of my uusd
    let msg = ExecuteMsg::Swap {
        offer_asset: AssetInfo::Native("uusd".to_string()).with_balance(50_000u128),
        ask_asset_info: None,
        max_spread: None,
        belief_price: None,
        to: None,
        referral_address: None,
        referral_commission: None,
    };
    router
        .execute_contract(
            owner.clone(),
            pair.clone(),
            &msg,
            &[Coin {
                denom: "uusd".to_string(),
                amount: Uint128::from(50_000u128),
            }],
        )
        .unwrap();

    let uluna_balance = router.wrap().query_balance(&owner, "uluna").unwrap().amount;

    // provide liquidity the swapped uluna and remaining uusd
    let (msg, coins) = provide_liquidity_msg(Uint128::from(50_000u128), uluna_balance, None, None);
    router
        .execute_contract(owner.clone(), pair.clone(), &msg, &coins)
        .unwrap();

    let res: PoolResponse = router
        .wrap()
        .query_wasm_smart(pair, &QueryMsg::Pool {})
        .unwrap();
    // should have 100_000_000_000 + 49999 shares (losing some because of spread)
    assert_eq!(100_000_000_000 + 49_999, res.total_share.u128());
}

#[test]
fn provide_liquidity_with_unequal_pool() {
    let owner = Addr::unchecked("owner");
    let mut router = mock_app(
        owner.clone(),
        vec![
            Coin {
                denom: "uusd".to_string(),
                amount: Uint128::new(123_456_789_123u128),
            },
            Coin {
                denom: "uluna".to_string(),
                amount: Uint128::new(100_000_100_000u128),
            },
        ],
    );

    let pair = instantiate_pair(&mut router, &owner);

    // first provide liquidity with two assets
    let (msg, coins) = provide_liquidity_msg(
        Uint128::from(123_456_789_123u128),
        Uint128::from(100_000_000_000u128),
        None,
        None,
    );
    router
        .execute_contract(owner.clone(), pair.clone(), &msg, &coins)
        .unwrap();

    let res: PoolResponse = router
        .wrap()
        .query_wasm_smart(pair.clone(), &QueryMsg::Pool {})
        .unwrap();
    // should have sqrt(100_000_000_000 * 123_456_789_123) = 111_111_110_660 shares
    assert_eq!(111_111_110_660, res.total_share.u128());

    // simulate swapping the 100_000 manually, just to understand the numbers better
    let res: SimulationResponse = router
        .wrap()
        .query_wasm_smart(
            pair.clone(),
            &QueryMsg::Simulation {
                offer_asset: AssetInfo::Native("uluna".to_string()).with_balance(50_000u128),
                ask_asset_info: Some(AssetInfo::Native("uusd".to_string())),
                referral: false,
                referral_commission: None,
            },
        )
        .unwrap();
    assert_eq!(res.return_amount.u128(), 61_728);

    // then with only one asset
    let msg = ExecuteMsg::ProvideLiquidity {
        assets: vec![AssetInfo::Native("uluna".to_string()).with_balance(100_000u128)],
        slippage_tolerance: None,
        receiver: None,
    };
    router
        .execute_contract(
            owner.clone(),
            pair.clone(),
            &msg,
            &[Coin {
                denom: "uluna".to_string(),
                amount: Uint128::new(100_000u128),
            }],
        )
        .unwrap();

    let res: PoolResponse = router
        .wrap()
        .query_wasm_smart(pair, &QueryMsg::Pool {})
        .unwrap();
    // as seen above, the swap should result in 61_728 uusd, so we
    // should have received sqrt(50_000 * 61_728) = 55_555 shares
    assert_eq!(111_111_110_660 + 55_555, res.total_share.u128());
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
        pair_type: PairType::Xyk {},
        asset_infos: asset_infos.clone(),
        init_params: None,
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
    // should have 100_000_000_000 + 49999 shares (losing some because of spread)
    assert_eq!(100_000_000_000 + 49_999, res.total_share.u128());
}

#[test]
fn wrong_number_of_assets() {
    let owner = Addr::unchecked("owner");
    let mut router = mock_app(owner.clone(), vec![]);

    let pair_contract_code_id = store_pair_code(&mut router);
    let staking_contract_code_id = store_staking_code(&mut router);

    let msg = InstantiateMsg {
        asset_infos: vec![AssetInfo::Native("uusd".to_string())],
        token_code_id: 123,
        factory_addr: String::from("factory"),
        init_params: None,
        staking_config: default_stake_config(staking_contract_code_id).to_stake_config(),
        trading_starts: 0,
        fee_config: FeeConfig {
            total_fee_bps: 0,
            protocol_fee_bps: 0,
        },
        circuit_breaker: None,
    };

    let err = router
        .instantiate_contract(
            pair_contract_code_id,
            owner.clone(),
            &msg,
            &[],
            String::from("PAIR"),
            None,
        )
        .unwrap_err();

    assert_eq!(
        err.root_cause().to_string(),
        "Invalid number of assets. This pair supports at least 2 and at most 2 assets within a pool"
    );

    let msg = InstantiateMsg {
        asset_infos: vec![
            native_asset_info("uusd"),
            native_asset_info("dust"),
            native_asset_info("stone"),
        ],
        token_code_id: 123,
        factory_addr: String::from("factory"),
        init_params: None,
        staking_config: default_stake_config(staking_contract_code_id).to_stake_config(),
        trading_starts: 0,
        fee_config: FeeConfig {
            total_fee_bps: 0,
            protocol_fee_bps: 0,
        },
        circuit_breaker: None,
    };

    let err = router
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
        err.root_cause().to_string(),
        "Invalid number of assets. This pair supports at least 2 and at most 2 assets within a pool"
    );
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
