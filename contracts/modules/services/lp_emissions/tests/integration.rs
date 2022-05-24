use cosmwasm_std::testing::{mock_env, MockApi, MockStorage};
use cosmwasm_std::{attr, to_binary, Addr, Decimal, Timestamp, Uint128};
use cw20::{BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg};
use cw_multi_test::{App, BankKeeper, ContractWrapper, Executor};

use abstract_os::tokenomics::lp_emissions::{
    ConfigResponse, Cw20HookMsg as LpCw20HookMsg, ExecuteMsg, InstantiateMsg, QueryMsg,
    StakerInfoResponse, StateResponse,
};

fn mock_app() -> App {
    let api = MockApi::default();
    let env = mock_env();
    let bank = BankKeeper::new();
    let storage = MockStorage::new();

    App::new(api, env.block, bank, storage)
}

fn init_contracts(app: &mut App) -> (Addr, Addr, Addr, InstantiateMsg) {
    let owner = Addr::unchecked("contract_owner");

    // Instantiate WHALE Token Contract
    let cw20_token_contract = Box::new(ContractWrapper::new(
        cw20_base::contract::execute,
        cw20_base::contract::instantiate,
        cw20_base::contract::query,
    ));

    let cw20_token_code_id = app.store_code(cw20_token_contract);

    let msg = cw20_base::msg::InstantiateMsg {
        name: String::from("Whale token"),
        symbol: String::from("WHALE"),
        decimals: 6,
        initial_balances: vec![],
        mint: Some(cw20::MinterResponse {
            minter: owner.to_string(),
            cap: None,
        }),
        marketing: None,
    };

    let whale_token_instance = app
        .instantiate_contract(
            cw20_token_code_id,
            owner.clone(),
            &msg,
            &[],
            String::from("WHALE"),
            None,
        )
        .unwrap();

    // Instantiate LP Token Contract
    let msg = cw20_base::msg::InstantiateMsg {
        name: String::from("Astro LP"),
        symbol: String::from("uLP"),
        decimals: 6,
        initial_balances: vec![],
        mint: Some(cw20::MinterResponse {
            minter: owner.to_string(),
            cap: None,
        }),
        marketing: None,
    };

    let lp_token_instance = app
        .instantiate_contract(
            cw20_token_code_id,
            owner.clone(),
            &msg,
            &[],
            String::from("AstroLP"),
            None,
        )
        .unwrap();

    // Instantiate Staking Contract
    let staking_contract = Box::new(ContractWrapper::new(
        whale_lp_emissions::contract::execute,
        whale_lp_emissions::contract::instantiate,
        whale_lp_emissions::contract::query,
    ));

    let staking_code_id = app.store_code(staking_contract);

    let staking_instantiate_msg = InstantiateMsg {
        owner: owner.to_string(),
        whale_token: whale_token_instance.to_string(),
        staking_token: lp_token_instance.to_string(),
        staking_token_decimals: 6u8,
    };

    app.update_block(|b| {
        b.height += 17;
        b.time = Timestamp::from_seconds(1571797419);
    });

    // Init contract
    let lp_emissions_instance = app
        .instantiate_contract(
            staking_code_id,
            owner.clone(),
            &staking_instantiate_msg,
            &[],
            "airdrop",
            None,
        )
        .unwrap();

    (
        lp_emissions_instance,
        whale_token_instance,
        lp_token_instance,
        staking_instantiate_msg,
    )
}

fn mint_some_whale(
    app: &mut App,
    owner: Addr,
    whale_token_instance: Addr,
    amount: Uint128,
    to: String,
) {
    let msg = cw20::Cw20ExecuteMsg::Mint {
        recipient: to.clone(),
        amount: amount,
    };
    let res = app
        .execute_contract(owner.clone(), whale_token_instance.clone(), &msg, &[])
        .unwrap();
    assert_eq!(res.events[1].attributes[1], attr("action", "mint"));
    assert_eq!(res.events[1].attributes[2], attr("to", to));
    assert_eq!(res.events[1].attributes[3], attr("amount", amount));
}

#[test]
fn proper_initialization() {
    let mut app = mock_app();
    let (lp_emissions_instance, _, _, init_msg) = init_contracts(&mut app);

    let resp: ConfigResponse = app
        .wrap()
        .query_wasm_smart(&lp_emissions_instance, &QueryMsg::Config {})
        .unwrap();

    // Check config
    assert_eq!(init_msg.owner, resp.owner);
    assert_eq!(init_msg.whale_token, resp.whale_token);
    assert_eq!(init_msg.staking_token, resp.staking_token);
    assert_eq!((0, 0, Uint128::zero()), resp.distribution_schedule);

    // Check state
    let resp: StateResponse = app
        .wrap()
        .query_wasm_smart(&lp_emissions_instance, &QueryMsg::State { timestamp: None })
        .unwrap();

    assert_eq!(1571797419u64, resp.last_distributed);
    assert_eq!(Uint128::zero(), resp.total_bond_amount);
    assert_eq!(Decimal::zero(), resp.global_reward_index);
}

#[test]
fn test_update_config() {
    let mut app = mock_app();
    let (lp_emissions_instance, _, _, init_msg) = init_contracts(&mut app);

    // *** Test : Error "Only owner can update configuration" ****
    let err = app
        .execute_contract(
            Addr::unchecked("wrong_owner"),
            lp_emissions_instance.clone(),
            &abstract_os::tokenomics::lp_emissions::ExecuteMsg::UpdateConfig {
                new_owner: "new_owner".to_string(),
            },
            &[],
        )
        .unwrap_err();

    assert_eq!(
        err.to_string(),
        "Generic error: Only owner can update configuration"
    );

    // *** Test : Should update successfully ****

    // should be a success
    app.execute_contract(
        Addr::unchecked(init_msg.owner),
        lp_emissions_instance.clone(),
        &abstract_os::tokenomics::lp_emissions::ExecuteMsg::UpdateConfig {
            new_owner: "new_owner".to_string(),
        },
        &[],
    )
    .unwrap();

    let resp: ConfigResponse = app
        .wrap()
        .query_wasm_smart(&lp_emissions_instance, &QueryMsg::Config {})
        .unwrap();

    // Check config and make sure all fields are updated
    assert_eq!("new_owner".to_string(), resp.owner);
    assert_eq!(init_msg.whale_token, resp.whale_token);
    assert_eq!(init_msg.staking_token, resp.staking_token);
    assert_eq!((0, 0, Uint128::zero()), resp.distribution_schedule);
}

#[test]
fn test_update_reward_schedule() {
    let mut app = mock_app();
    let (lp_emissions_instance, whale_token_instance, lp_token_instance, init_msg) =
        init_contracts(&mut app);

    mint_some_whale(
        &mut app,
        Addr::unchecked(init_msg.owner.clone()),
        whale_token_instance.clone(),
        Uint128::new(900_000_000_000),
        init_msg.owner.clone().to_string(),
    );

    mint_some_whale(
        &mut app,
        Addr::unchecked(init_msg.owner.clone()),
        whale_token_instance.clone(),
        Uint128::new(900_000_000_000),
        "wrong_owner".to_string(),
    );

    // *** Test : Error "Only owner can update the schedule" ****

    let err = app
        .execute_contract(
            Addr::unchecked("wrong_owner"),
            whale_token_instance.clone(),
            &Cw20ExecuteMsg::Send {
                contract: lp_emissions_instance.clone().to_string(),
                amount: Uint128::new(900_000_000_000),
                msg: to_binary(&LpCw20HookMsg::UpdateRewardSchedule {
                    period_start: 1572000000u64,
                    period_finish: 1772000000u64,
                    amount: Uint128::new(900_000_000_000),
                })
                .unwrap(),
            },
            &[],
        )
        .unwrap_err();

    assert_eq!(
        err.to_string(),
        "Generic error: Only owner can update the schedule"
    );

    // *** Test : Error "Only WHALE token contract can execute this message" ****

    // Mint LP Tokens
    app.execute_contract(
        Addr::unchecked(init_msg.owner.clone()),
        lp_token_instance.clone(),
        &cw20::Cw20ExecuteMsg::Mint {
            recipient: init_msg.owner.clone().to_string(),
            amount: Uint128::new(9000),
        },
        &[],
    )
    .unwrap();

    let err = app
        .execute_contract(
            Addr::unchecked(init_msg.owner.clone()),
            lp_token_instance.clone(),
            &Cw20ExecuteMsg::Send {
                contract: lp_emissions_instance.clone().to_string(),
                amount: Uint128::new(9000),
                msg: to_binary(&LpCw20HookMsg::UpdateRewardSchedule {
                    period_start: 1572000000u64,
                    period_finish: 1772000000u64,
                    amount: Uint128::new(9000),
                })
                .unwrap(),
            },
            &[],
        )
        .unwrap_err();

    assert_eq!(
        err.to_string(),
        "Generic error: Unauthorized : Only WHALE Token is allowed"
    );

    // *** Test : Error "insufficient funds on contract" ****

    let err = app
        .execute_contract(
            Addr::unchecked(init_msg.owner.clone()),
            whale_token_instance.clone(),
            &Cw20ExecuteMsg::Send {
                contract: lp_emissions_instance.clone().to_string(),
                amount: Uint128::new(900_000_000_000),
                msg: to_binary(&LpCw20HookMsg::UpdateRewardSchedule {
                    period_start: 1572000000u64,
                    period_finish: 1772000000u64,
                    amount: Uint128::new(900_000_000_001),
                })
                .unwrap(),
            },
            &[],
        )
        .unwrap_err();

    assert_eq!(
        err.to_string(),
        "Generic error: insufficient funds on contract"
    );

    // *** Test : Should update successfully ****

    // should be a success
    app.execute_contract(
        Addr::unchecked(init_msg.owner.clone()),
        whale_token_instance.clone(),
        &Cw20ExecuteMsg::Send {
            contract: lp_emissions_instance.clone().to_string(),
            amount: Uint128::new(900_000_000_000),
            msg: to_binary(&LpCw20HookMsg::UpdateRewardSchedule {
                period_start: 1572000000u64,
                period_finish: 1772000000u64,
                amount: Uint128::new(900_000_000_000),
            })
            .unwrap(),
        },
        &[],
    )
    .unwrap();

    assert_eq!(
        err.to_string(),
        "Generic error: insufficient funds on contract"
    );

    let resp: ConfigResponse = app
        .wrap()
        .query_wasm_smart(&lp_emissions_instance, &QueryMsg::Config {})
        .unwrap();

    // Check config and make sure all fields are updated
    assert_eq!(init_msg.whale_token, resp.whale_token);
    assert_eq!(init_msg.staking_token, resp.staking_token);
    assert_eq!(
        (1572000000, 1772000000, Uint128::from(900000000000u64)),
        resp.distribution_schedule
    );
}

#[test]
fn test_bond_tokens() {
    let mut app = mock_app();
    let (lp_emissions_instance, whale_token_instance, lp_token_instance, init_msg) =
        init_contracts(&mut app);

    mint_some_whale(
        &mut app,
        Addr::unchecked(init_msg.owner.clone()),
        whale_token_instance.clone(),
        Uint128::new(900_000_000_000),
        init_msg.owner.clone().to_string(),
    );

    // Mint LP Tokens
    app.execute_contract(
        Addr::unchecked(init_msg.owner.clone()),
        lp_token_instance.clone(),
        &cw20::Cw20ExecuteMsg::Mint {
            recipient: "user1".to_string(),
            amount: Uint128::new(9000_000_000_000000),
        },
        &[],
    )
    .unwrap();

    // Mint LP Tokens
    app.execute_contract(
        Addr::unchecked(init_msg.owner.clone()),
        lp_token_instance.clone(),
        &cw20::Cw20ExecuteMsg::Mint {
            recipient: "user2".to_string(),
            amount: Uint128::new(9000_000_000_000000),
        },
        &[],
    )
    .unwrap();
    // should set reward schedule
    app.execute_contract(
        Addr::unchecked(init_msg.owner.clone()),
        whale_token_instance.clone(),
        &Cw20ExecuteMsg::Send {
            contract: lp_emissions_instance.clone().to_string(),
            amount: Uint128::new(900_000_000_000),
            msg: to_binary(&LpCw20HookMsg::UpdateRewardSchedule {
                period_start: 1572000000u64,
                period_finish: 1772000000u64,
                amount: Uint128::new(900_000_000_000),
            })
            .unwrap(),
        },
        &[],
    )
    .unwrap();

    // ***
    // *** Test :: Staking before reward distribution goes live ***
    // ***

    app.update_block(|b| {
        b.height += 17;
        b.time = Timestamp::from_seconds(1571500000u64);
    });

    // should bond LP Tokens
    app.execute_contract(
        Addr::unchecked("user1".to_string()),
        lp_token_instance.clone(),
        &Cw20ExecuteMsg::Send {
            contract: lp_emissions_instance.clone().to_string(),
            amount: Uint128::new(900_000_000),
            msg: to_binary(&LpCw20HookMsg::Bond {}).unwrap(),
        },
        &[],
    )
    .unwrap();

    let mut state_resp: StateResponse = app
        .wrap()
        .query_wasm_smart(&lp_emissions_instance, &QueryMsg::State { timestamp: None })
        .unwrap();

    // Check state and make sure all fields are updated
    assert_eq!(1571500000u64, state_resp.last_distributed);
    assert_eq!(Uint128::new(900_000_000), state_resp.total_bond_amount);
    assert_eq!(Decimal::zero(), state_resp.global_reward_index);
    assert_eq!(Uint128::new(900_000_000_000), state_resp.leftover);
    assert_eq!(Decimal::zero(), state_resp.reward_rate_per_token);

    let mut staker_resp: StakerInfoResponse = app
        .wrap()
        .query_wasm_smart(
            &lp_emissions_instance,
            &QueryMsg::StakerInfo {
                staker: "user1".to_string(),
                timestamp: None,
            },
        )
        .unwrap();

    // Check state and make sure all fields are updated
    assert_eq!("user1".to_string(), staker_resp.staker);
    assert_eq!(Decimal::zero(), staker_resp.reward_index);
    assert_eq!(Uint128::new(900_000_000), staker_resp.bond_amount);
    assert_eq!(Uint128::zero(), staker_resp.pending_reward);

    // ***
    // *** Test :: Staking when reward distribution just goes live ***
    // ***

    app.update_block(|b| {
        b.height += 17;
        b.time = Timestamp::from_seconds(1572000001u64);
    });

    // should bond LP Tokens
    app.execute_contract(
        Addr::unchecked("user2".to_string()),
        lp_token_instance.clone(),
        &Cw20ExecuteMsg::Send {
            contract: lp_emissions_instance.clone().to_string(),
            amount: Uint128::new(900_000_000),
            msg: to_binary(&LpCw20HookMsg::Bond {}).unwrap(),
        },
        &[],
    )
    .unwrap();

    state_resp = app
        .wrap()
        .query_wasm_smart(&lp_emissions_instance, &QueryMsg::State { timestamp: None })
        .unwrap();

    // Check state and make sure all fields are updated
    assert_eq!(1572000001u64, state_resp.last_distributed);
    assert_eq!(Uint128::new(1800_000_000), state_resp.total_bond_amount);
    assert_eq!(Uint128::new(899999995500), state_resp.leftover);

    staker_resp = app
        .wrap()
        .query_wasm_smart(
            &lp_emissions_instance,
            &QueryMsg::StakerInfo {
                staker: "user1".to_string(),
                timestamp: None,
            },
        )
        .unwrap();

    // Check user state and make sure all fields are updated
    assert_eq!("user1".to_string(), staker_resp.staker);
    assert_eq!(Uint128::new(900_000_000), staker_resp.bond_amount);
    assert_eq!(Uint128::new(4500), staker_resp.pending_reward);

    staker_resp = app
        .wrap()
        .query_wasm_smart(
            &lp_emissions_instance,
            &QueryMsg::StakerInfo {
                staker: "user2".to_string(),
                timestamp: None,
            },
        )
        .unwrap();

    // Check user state and make sure all fields are updated
    assert_eq!("user2".to_string(), staker_resp.staker);
    assert_eq!(Uint128::new(900_000_000), staker_resp.bond_amount);
    assert_eq!(Uint128::new(0), staker_resp.pending_reward);

    // ***
    // *** Test :: Staking when reward distribution has been live ***
    // ***

    app.update_block(|b| {
        b.height += 17;
        b.time = Timestamp::from_seconds(1572000101u64);
    });

    // should bond LP Tokens
    app.execute_contract(
        Addr::unchecked("user1".to_string()),
        lp_token_instance.clone(),
        &Cw20ExecuteMsg::Send {
            contract: lp_emissions_instance.clone().to_string(),
            amount: Uint128::new(900_000),
            msg: to_binary(&LpCw20HookMsg::Bond {}).unwrap(),
        },
        &[],
    )
    .unwrap();

    // should bond LP Tokens
    app.execute_contract(
        Addr::unchecked("user2".to_string()),
        lp_token_instance.clone(),
        &Cw20ExecuteMsg::Send {
            contract: lp_emissions_instance.clone().to_string(),
            amount: Uint128::new(900_000),
            msg: to_binary(&LpCw20HookMsg::Bond {}).unwrap(),
        },
        &[],
    )
    .unwrap();

    state_resp = app
        .wrap()
        .query_wasm_smart(&lp_emissions_instance, &QueryMsg::State { timestamp: None })
        .unwrap();

    // Check state and make sure all fields are updated
    assert_eq!(1572000101u64, state_resp.last_distributed);
    assert_eq!(Uint128::new(1801800000), state_resp.total_bond_amount);
    assert_eq!(Uint128::new(899999545500), state_resp.leftover);

    staker_resp = app
        .wrap()
        .query_wasm_smart(
            &lp_emissions_instance,
            &QueryMsg::StakerInfo {
                staker: "user1".to_string(),
                timestamp: None,
            },
        )
        .unwrap();

    // Check user state and make sure all fields are updated
    assert_eq!("user1".to_string(), staker_resp.staker);
    assert_eq!(Uint128::new(900_900_000), staker_resp.bond_amount);
    assert_eq!(Uint128::new(229500), staker_resp.pending_reward);

    staker_resp = app
        .wrap()
        .query_wasm_smart(
            &lp_emissions_instance,
            &QueryMsg::StakerInfo {
                staker: "user2".to_string(),
                timestamp: None,
            },
        )
        .unwrap();

    // Check user state and make sure all fields are updated
    assert_eq!("user2".to_string(), staker_resp.staker);
    assert_eq!(Uint128::new(900_900_000), staker_resp.bond_amount);
    assert_eq!(Uint128::new(225000), staker_resp.pending_reward);

    // ***
    // *** Test :: Staking when reward distribution is over ***
    // ***

    app.update_block(|b| {
        b.height += 17;
        b.time = Timestamp::from_seconds(1772000001u64);
    });

    // should bond LP Tokens
    app.execute_contract(
        Addr::unchecked("user1".to_string()),
        lp_token_instance.clone(),
        &Cw20ExecuteMsg::Send {
            contract: lp_emissions_instance.clone().to_string(),
            amount: Uint128::new(900_000),
            msg: to_binary(&LpCw20HookMsg::Bond {}).unwrap(),
        },
        &[],
    )
    .unwrap();

    // should bond LP Tokens
    app.execute_contract(
        Addr::unchecked("user2".to_string()),
        lp_token_instance.clone(),
        &Cw20ExecuteMsg::Send {
            contract: lp_emissions_instance.clone().to_string(),
            amount: Uint128::new(900_000),
            msg: to_binary(&LpCw20HookMsg::Bond {}).unwrap(),
        },
        &[],
    )
    .unwrap();

    state_resp = app
        .wrap()
        .query_wasm_smart(&lp_emissions_instance, &QueryMsg::State { timestamp: None })
        .unwrap();

    // Check state and make sure all fields are updated
    assert_eq!(1772000001u64, state_resp.last_distributed);
    assert_eq!(Uint128::new(1803600000), state_resp.total_bond_amount);
    assert_eq!(Uint128::new(0), state_resp.leftover);

    staker_resp = app
        .wrap()
        .query_wasm_smart(
            &lp_emissions_instance,
            &QueryMsg::StakerInfo {
                staker: "user1".to_string(),
                timestamp: None,
            },
        )
        .unwrap();

    // Check user state and make sure all fields are updated
    assert_eq!("user1".to_string(), staker_resp.staker);
    assert_eq!(Uint128::new(901_800_000), staker_resp.bond_amount);
    assert_eq!(Uint128::new(450000002250), staker_resp.pending_reward);

    staker_resp = app
        .wrap()
        .query_wasm_smart(
            &lp_emissions_instance,
            &QueryMsg::StakerInfo {
                staker: "user2".to_string(),
                timestamp: None,
            },
        )
        .unwrap();

    // Check user state and make sure all fields are updated
    assert_eq!("user2".to_string(), staker_resp.staker);
    assert_eq!(Uint128::new(901_800_000), staker_resp.bond_amount);
    assert_eq!(Uint128::new(449999997750), staker_resp.pending_reward);

    // ***
    // *** Test :: Staking when reward distribution is over (2nd time) ***
    // ***

    app.update_block(|b| {
        b.height += 17;
        b.time = Timestamp::from_seconds(1772000101u64);
    });

    // should bond LP Tokens
    app.execute_contract(
        Addr::unchecked("user1".to_string()),
        lp_token_instance.clone(),
        &Cw20ExecuteMsg::Send {
            contract: lp_emissions_instance.clone().to_string(),
            amount: Uint128::new(900_000),
            msg: to_binary(&LpCw20HookMsg::Bond {}).unwrap(),
        },
        &[],
    )
    .unwrap();

    // should bond LP Tokens
    app.execute_contract(
        Addr::unchecked("user2".to_string()),
        lp_token_instance.clone(),
        &Cw20ExecuteMsg::Send {
            contract: lp_emissions_instance.clone().to_string(),
            amount: Uint128::new(900_000),
            msg: to_binary(&LpCw20HookMsg::Bond {}).unwrap(),
        },
        &[],
    )
    .unwrap();

    state_resp = app
        .wrap()
        .query_wasm_smart(&lp_emissions_instance, &QueryMsg::State { timestamp: None })
        .unwrap();

    // Check state and make sure all fields are updated
    assert_eq!(1772000101, state_resp.last_distributed);
    assert_eq!(Uint128::new(1805400000), state_resp.total_bond_amount);
    assert_eq!(Uint128::new(0), state_resp.leftover);

    staker_resp = app
        .wrap()
        .query_wasm_smart(
            &lp_emissions_instance,
            &QueryMsg::StakerInfo {
                staker: "user1".to_string(),
                timestamp: None,
            },
        )
        .unwrap();

    // Check user state and make sure all fields are updated
    assert_eq!("user1".to_string(), staker_resp.staker);
    assert_eq!(Uint128::new(902_700_000), staker_resp.bond_amount);
    assert_eq!(Uint128::new(450000002250), staker_resp.pending_reward);

    staker_resp = app
        .wrap()
        .query_wasm_smart(
            &lp_emissions_instance,
            &QueryMsg::StakerInfo {
                staker: "user2".to_string(),
                timestamp: None,
            },
        )
        .unwrap();

    // Check user state and make sure all fields are updated
    assert_eq!("user2".to_string(), staker_resp.staker);
    assert_eq!(Uint128::new(902_700_000), staker_resp.bond_amount);
    assert_eq!(Uint128::new(449999997750), staker_resp.pending_reward);
}

#[test]
fn test_unbond_tokens() {
    let mut app = mock_app();
    let (lp_emissions_instance, whale_token_instance, lp_token_instance, init_msg) =
        init_contracts(&mut app);

    mint_some_whale(
        &mut app,
        Addr::unchecked(init_msg.owner.clone()),
        whale_token_instance.clone(),
        Uint128::new(900_000_000_000),
        init_msg.owner.clone().to_string(),
    );

    // Mint LP Tokens
    app.execute_contract(
        Addr::unchecked(init_msg.owner.clone()),
        lp_token_instance.clone(),
        &cw20::Cw20ExecuteMsg::Mint {
            recipient: "user1".to_string(),
            amount: Uint128::new(9000_000_000_000000),
        },
        &[],
    )
    .unwrap();

    // Mint LP Tokens
    app.execute_contract(
        Addr::unchecked(init_msg.owner.clone()),
        lp_token_instance.clone(),
        &cw20::Cw20ExecuteMsg::Mint {
            recipient: "user2".to_string(),
            amount: Uint128::new(9000_000_000_000000),
        },
        &[],
    )
    .unwrap();

    // should set reward schedule
    app.execute_contract(
        Addr::unchecked(init_msg.owner.clone()),
        whale_token_instance.clone(),
        &Cw20ExecuteMsg::Send {
            contract: lp_emissions_instance.clone().to_string(),
            amount: Uint128::new(900_000_000_000),
            msg: to_binary(&LpCw20HookMsg::UpdateRewardSchedule {
                period_start: 1572000000u64,
                period_finish: 1772000000u64,
                amount: Uint128::new(900_000_000_000),
            })
            .unwrap(),
        },
        &[],
    )
    .unwrap();

    // Staking before reward distribution goes live

    app.update_block(|b| {
        b.height += 17;
        b.time = Timestamp::from_seconds(1571500000u64);
    });

    // should bond LP Tokens
    app.execute_contract(
        Addr::unchecked("user1".to_string()),
        lp_token_instance.clone(),
        &Cw20ExecuteMsg::Send {
            contract: lp_emissions_instance.clone().to_string(),
            amount: Uint128::new(900_000_000),
            msg: to_binary(&LpCw20HookMsg::Bond {}).unwrap(),
        },
        &[],
    )
    .unwrap();

    app.update_block(|b| {
        b.height += 17;
        b.time = Timestamp::from_seconds(1571500005u64);
    });

    //
    // Test ::: should unbond LP Tokens (without rewards claim)
    //
    app.execute_contract(
        Addr::unchecked("user1".to_string()),
        lp_emissions_instance.clone(),
        &ExecuteMsg::Unbond {
            amount: Uint128::new(10),
            withdraw_pending_reward: Some(false),
        },
        &[],
    )
    .unwrap();

    let mut state_resp: StateResponse = app
        .wrap()
        .query_wasm_smart(&lp_emissions_instance, &QueryMsg::State { timestamp: None })
        .unwrap();

    // Check state and make sure all fields are updated
    assert_eq!(1571500005u64, state_resp.last_distributed);
    assert_eq!(Uint128::new(899999990), state_resp.total_bond_amount);
    assert_eq!(Uint128::new(900_000_000_000), state_resp.leftover);

    let mut staker_resp: StakerInfoResponse = app
        .wrap()
        .query_wasm_smart(
            &lp_emissions_instance,
            &QueryMsg::StakerInfo {
                staker: "user1".to_string(),
                timestamp: None,
            },
        )
        .unwrap();

    // Check user state and make sure all fields are updated
    assert_eq!("user1".to_string(), staker_resp.staker);
    assert_eq!(Uint128::new(899999990), staker_resp.bond_amount);
    assert_eq!(Uint128::new(0), staker_resp.pending_reward);

    //
    // Test ::: should unbond LP Tokens (with rewards claim) :::
    //
    app.execute_contract(
        Addr::unchecked("user1".to_string()),
        lp_emissions_instance.clone(),
        &ExecuteMsg::Unbond {
            amount: Uint128::new(10),
            withdraw_pending_reward: Some(true),
        },
        &[],
    )
    .unwrap();

    state_resp = app
        .wrap()
        .query_wasm_smart(&lp_emissions_instance, &QueryMsg::State { timestamp: None })
        .unwrap();

    // Check state and make sure all fields are updated
    assert_eq!(1571500005u64, state_resp.last_distributed);
    assert_eq!(Uint128::new(899999980), state_resp.total_bond_amount);
    assert_eq!(Uint128::new(900_000_000_000), state_resp.leftover);

    staker_resp = app
        .wrap()
        .query_wasm_smart(
            &lp_emissions_instance,
            &QueryMsg::StakerInfo {
                staker: "user1".to_string(),
                timestamp: None,
            },
        )
        .unwrap();

    // Check user state and make sure all fields are updated
    assert_eq!("user1".to_string(), staker_resp.staker);
    assert_eq!(Uint128::new(899999980), staker_resp.bond_amount);
    assert_eq!(Uint128::new(0), staker_resp.pending_reward);

    // Unbonding when reward distribution is live

    app.update_block(|b| {
        b.height += 17;
        b.time = Timestamp::from_seconds(1572000001u64);
    });

    //
    // Test ::: should unbond LP Tokens (without rewards claim)
    //
    app.execute_contract(
        Addr::unchecked("user1".to_string()),
        lp_emissions_instance.clone(),
        &ExecuteMsg::Unbond {
            amount: Uint128::new(10),
            withdraw_pending_reward: Some(false),
        },
        &[],
    )
    .unwrap();

    let mut state_resp: StateResponse = app
        .wrap()
        .query_wasm_smart(&lp_emissions_instance, &QueryMsg::State { timestamp: None })
        .unwrap();

    // Check state and make sure all fields are updated
    assert_eq!(1572000001u64, state_resp.last_distributed);
    assert_eq!(Uint128::new(899999970), state_resp.total_bond_amount);
    assert_eq!(Uint128::new(899999995500), state_resp.leftover);

    let mut staker_resp: StakerInfoResponse = app
        .wrap()
        .query_wasm_smart(
            &lp_emissions_instance,
            &QueryMsg::StakerInfo {
                staker: "user1".to_string(),
                timestamp: None,
            },
        )
        .unwrap();

    // Check user state and make sure all fields are updated
    assert_eq!("user1".to_string(), staker_resp.staker);
    assert_eq!(Uint128::new(899999970), staker_resp.bond_amount);
    assert_eq!(Uint128::new(4499), staker_resp.pending_reward);

    //
    // Test ::: should unbond LP Tokens (with rewards claim) :::
    //
    app.execute_contract(
        Addr::unchecked("user1".to_string()),
        lp_emissions_instance.clone(),
        &ExecuteMsg::Unbond {
            amount: Uint128::new(10),
            withdraw_pending_reward: Some(true),
        },
        &[],
    )
    .unwrap();

    state_resp = app
        .wrap()
        .query_wasm_smart(&lp_emissions_instance, &QueryMsg::State { timestamp: None })
        .unwrap();

    // Check state and make sure all fields are updated
    assert_eq!(1572000001u64, state_resp.last_distributed);
    assert_eq!(Uint128::new(899999960), state_resp.total_bond_amount);
    assert_eq!(Uint128::new(899999995500), state_resp.leftover);

    staker_resp = app
        .wrap()
        .query_wasm_smart(
            &lp_emissions_instance,
            &QueryMsg::StakerInfo {
                staker: "user1".to_string(),
                timestamp: None,
            },
        )
        .unwrap();

    // Check user state and make sure all fields are updated
    assert_eq!("user1".to_string(), staker_resp.staker);
    assert_eq!(Uint128::new(899999960), staker_resp.bond_amount);
    assert_eq!(Uint128::new(0), staker_resp.pending_reward);

    // ***
    // *** Test :: Unbonding when reward distribution is over ***
    // ***

    app.update_block(|b| {
        b.height += 17;
        b.time = Timestamp::from_seconds(1772000001u64);
    });

    //
    // Test ::: should unbond LP Tokens (without rewards claim)
    //
    app.execute_contract(
        Addr::unchecked("user1".to_string()),
        lp_emissions_instance.clone(),
        &ExecuteMsg::Unbond {
            amount: Uint128::new(10),
            withdraw_pending_reward: Some(false),
        },
        &[],
    )
    .unwrap();

    let mut state_resp: StateResponse = app
        .wrap()
        .query_wasm_smart(&lp_emissions_instance, &QueryMsg::State { timestamp: None })
        .unwrap();

    // Check state and make sure all fields are updated
    assert_eq!(1772000001u64, state_resp.last_distributed);
    assert_eq!(Uint128::new(899999950), state_resp.total_bond_amount);
    assert_eq!(Uint128::new(0), state_resp.leftover);

    let mut staker_resp: StakerInfoResponse = app
        .wrap()
        .query_wasm_smart(
            &lp_emissions_instance,
            &QueryMsg::StakerInfo {
                staker: "user1".to_string(),
                timestamp: None,
            },
        )
        .unwrap();

    // Check user state and make sure all fields are updated
    assert_eq!("user1".to_string(), staker_resp.staker);
    assert_eq!(Uint128::new(899999950), staker_resp.bond_amount);
    assert_eq!(Uint128::new(899999995500), staker_resp.pending_reward);

    //
    // Test ::: should unbond LP Tokens (with rewards claim) :::
    //
    app.execute_contract(
        Addr::unchecked("user1".to_string()),
        lp_emissions_instance.clone(),
        &ExecuteMsg::Unbond {
            amount: Uint128::new(10),
            withdraw_pending_reward: Some(true),
        },
        &[],
    )
    .unwrap();

    state_resp = app
        .wrap()
        .query_wasm_smart(&lp_emissions_instance, &QueryMsg::State { timestamp: None })
        .unwrap();

    // Check state and make sure all fields are updated
    assert_eq!(1772000001, state_resp.last_distributed);
    assert_eq!(Uint128::new(899999940), state_resp.total_bond_amount);
    assert_eq!(Uint128::new(0), state_resp.leftover);

    staker_resp = app
        .wrap()
        .query_wasm_smart(
            &lp_emissions_instance,
            &QueryMsg::StakerInfo {
                staker: "user1".to_string(),
                timestamp: None,
            },
        )
        .unwrap();

    // Check user state and make sure all fields are updated
    assert_eq!("user1".to_string(), staker_resp.staker);
    assert_eq!(Uint128::new(899999940), staker_resp.bond_amount);
    assert_eq!(Uint128::new(0), staker_resp.pending_reward);

    //
    // Test Error ::: Cannot unbond more than bond amount
    //
    let err = app
        .execute_contract(
            Addr::unchecked("user1".to_string()),
            lp_emissions_instance.clone(),
            &ExecuteMsg::Unbond {
                amount: Uint128::new(100000_00000000),
                withdraw_pending_reward: Some(false),
            },
            &[],
        )
        .unwrap_err();
    assert_eq!(
        err.to_string(),
        "Generic error: Cannot unbond more than bond amount"
    );
}

#[test]
fn test_claim_rewards() {
    let mut app = mock_app();
    let (lp_emissions_instance, whale_token_instance, lp_token_instance, init_msg) =
        init_contracts(&mut app);

    mint_some_whale(
        &mut app,
        Addr::unchecked(init_msg.owner.clone()),
        whale_token_instance.clone(),
        Uint128::new(900_000_000_000),
        init_msg.owner.clone().to_string(),
    );

    // Mint LP Tokens
    app.execute_contract(
        Addr::unchecked(init_msg.owner.clone()),
        lp_token_instance.clone(),
        &cw20::Cw20ExecuteMsg::Mint {
            recipient: "user1".to_string(),
            amount: Uint128::new(9000_000_000_000000),
        },
        &[],
    )
    .unwrap();

    // Mint LP Tokens
    app.execute_contract(
        Addr::unchecked(init_msg.owner.clone()),
        lp_token_instance.clone(),
        &cw20::Cw20ExecuteMsg::Mint {
            recipient: "user2".to_string(),
            amount: Uint128::new(9000_000_000_000000),
        },
        &[],
    )
    .unwrap();

    // should set reward schedule
    app.execute_contract(
        Addr::unchecked(init_msg.owner.clone()),
        whale_token_instance.clone(),
        &Cw20ExecuteMsg::Send {
            contract: lp_emissions_instance.clone().to_string(),
            amount: Uint128::new(900_000_000_000),
            msg: to_binary(&LpCw20HookMsg::UpdateRewardSchedule {
                period_start: 1572000000u64,
                period_finish: 1772000000u64,
                amount: Uint128::new(900_000_000_000),
            })
            .unwrap(),
        },
        &[],
    )
    .unwrap();

    // Staking before reward distribution goes live

    app.update_block(|b| {
        b.height += 17;
        b.time = Timestamp::from_seconds(1571500000u64);
    });

    // should bond LP Tokens
    app.execute_contract(
        Addr::unchecked("user1".to_string()),
        lp_token_instance.clone(),
        &Cw20ExecuteMsg::Send {
            contract: lp_emissions_instance.clone().to_string(),
            amount: Uint128::new(900_000_000),
            msg: to_binary(&LpCw20HookMsg::Bond {}).unwrap(),
        },
        &[],
    )
    .unwrap();

    app.update_block(|b| {
        b.height += 17;
        b.time = Timestamp::from_seconds(1571500005u64);
    });

    //
    // Test Error  ::: should claim Rewards Tokens (Error :: )
    //
    let err = app
        .execute_contract(
            Addr::unchecked("user1".to_string()),
            lp_emissions_instance.clone(),
            &ExecuteMsg::Claim {},
            &[],
        )
        .unwrap_err();
    assert_eq!(err.to_string(), "Generic error: No rewards to claim");

    // Unbonding when reward distribution is live

    app.update_block(|b| {
        b.height += 17;
        b.time = Timestamp::from_seconds(1572000001u64);
    });

    //
    // Test ::: should claim rewards (When rewards are being distributed)
    //

    app.execute_contract(
        Addr::unchecked("user1".to_string()),
        lp_emissions_instance.clone(),
        &ExecuteMsg::Claim {},
        &[],
    )
    .unwrap();

    let state_resp: StateResponse = app
        .wrap()
        .query_wasm_smart(&lp_emissions_instance, &QueryMsg::State { timestamp: None })
        .unwrap();

    // Check state and make sure all fields are updated
    assert_eq!(1572000001u64, state_resp.last_distributed);
    assert_eq!(Uint128::new(900_000_000), state_resp.total_bond_amount);
    assert_eq!(Uint128::new(899999995500), state_resp.leftover);

    let staker_resp: StakerInfoResponse = app
        .wrap()
        .query_wasm_smart(
            &lp_emissions_instance,
            &QueryMsg::StakerInfo {
                staker: "user1".to_string(),
                timestamp: None,
            },
        )
        .unwrap();

    // Check user state and make sure all fields are updated
    assert_eq!("user1".to_string(), staker_resp.staker);
    assert_eq!(Uint128::new(900_000_000), staker_resp.bond_amount);
    assert_eq!(Uint128::new(0), staker_resp.pending_reward);

    //
    // Test ::: should claim rewards (When rewards are being distributed)
    //

    app.update_block(|b| {
        b.height += 17;
        b.time = Timestamp::from_seconds(1572005001u64);
    });

    let staker_resp_before: StakerInfoResponse = app
        .wrap()
        .query_wasm_smart(
            &lp_emissions_instance,
            &QueryMsg::StakerInfo {
                staker: "user1".to_string(),
                timestamp: None,
            },
        )
        .unwrap();

    assert_eq!(Uint128::new(22500000), staker_resp_before.pending_reward);

    app.execute_contract(
        Addr::unchecked("user1".to_string()),
        lp_emissions_instance.clone(),
        &ExecuteMsg::Claim {},
        &[],
    )
    .unwrap();

    let _staker_resp_after: StakerInfoResponse = app
        .wrap()
        .query_wasm_smart(
            &lp_emissions_instance,
            &QueryMsg::StakerInfo {
                staker: "user1".to_string(),
                timestamp: None,
            },
        )
        .unwrap();

    assert_eq!(Uint128::new(0), staker_resp.pending_reward);

    let staker_balance: BalanceResponse = app
        .wrap()
        .query_wasm_smart(
            &whale_token_instance,
            &Cw20QueryMsg::Balance {
                address: "user1".to_string(),
            },
        )
        .unwrap();

    assert_eq!(
        Uint128::new(4500) + staker_resp_before.pending_reward,
        staker_balance.balance
    );

    // ***
    // *** Test :: Unbonding when reward distribution is over ***
    // ***

    app.update_block(|b| {
        b.height += 17;
        b.time = Timestamp::from_seconds(1772000001u64);
    });

    app.execute_contract(
        Addr::unchecked("user1".to_string()),
        lp_emissions_instance.clone(),
        &ExecuteMsg::Claim {},
        &[],
    )
    .unwrap();

    let _staker_resp_after: StakerInfoResponse = app
        .wrap()
        .query_wasm_smart(
            &lp_emissions_instance,
            &QueryMsg::StakerInfo {
                staker: "user1".to_string(),
                timestamp: None,
            },
        )
        .unwrap();

    assert_eq!(Uint128::new(0), staker_resp.pending_reward);
}
