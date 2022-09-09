use abstract_os::vesting::{
    AllocationInfo, AllocationResponse, ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg,
    ReceiveMsg, Schedule, SimulateWithdrawResponse, StateResponse,
};
use cosmwasm_std::{
    attr,
    testing::{mock_env, MockApi, MockStorage},
    to_binary, Addr, Coin, Timestamp, Uint128,
};
use cw_multi_test::{App, AppBuilder, BankKeeper, ContractWrapper, Executor};

const OWNER: &str = "owner";

pub fn mock_app() -> App {
    let env = mock_env();
    let api = MockApi::default();
    let bank = BankKeeper::new();
    let storage = MockStorage::new();

    let sender = Addr::unchecked(OWNER);
    let funds = vec![Coin::new(1_000_000_000, "uusd")];

    AppBuilder::new()
        .with_api(api)
        .with_block(env.block)
        .with_bank(bank)
        .with_storage(storage)
        .build(|router, _, storage| {
            router
                .bank
                .init_balance(storage, &sender, funds.clone())
                .unwrap();
        })
}

fn init_contracts(app: &mut App) -> (Addr, Addr, InstantiateMsg) {
    // Instantiate WHALE Token Contract
    let whale_token_contract = Box::new(ContractWrapper::new(
        cw20_base::contract::execute,
        cw20_base::contract::instantiate,
        cw20_base::contract::query,
    ));

    let whale_token_code_id = app.store_code(whale_token_contract);

    let msg = cw20_base::msg::InstantiateMsg {
        name: String::from("Whale token"),
        symbol: String::from("WHALE"),
        decimals: 6,
        initial_balances: vec![],
        mint: Some(cw20::MinterResponse {
            minter: OWNER.to_string(),
            cap: None,
        }),
        marketing: None,
    };

    let whale_token_instance = app
        .instantiate_contract(
            whale_token_code_id,
            Addr::unchecked(OWNER.clone()),
            &msg,
            &[],
            String::from("WHALE"),
            None,
        )
        .unwrap();

    // Instantiate Vesting Contract
    let vesting_contract = Box::new(ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    ));

    let vesting_code_id = app.store_code(vesting_contract);

    let vesting_instantiate_msg = InstantiateMsg {
        owner: OWNER.clone().to_string(),
        refund_recipient: "refund_recipient".to_string(),
        token: whale_token_instance.to_string(),
        default_unlock_schedule: Schedule {
            start_time: 0u64,
            cliff: 0u64,
            duration: 1u64,
        },
    };

    // Init contract
    let vesting_instance = app
        .instantiate_contract(
            vesting_code_id,
            Addr::unchecked(OWNER.clone()),
            &vesting_instantiate_msg,
            &[],
            "vesting",
            None,
        )
        .unwrap();

    (
        vesting_instance,
        whale_token_instance,
        vesting_instantiate_msg,
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
    let (vesting_instance, _tokens_instance, init_msg) = init_contracts(&mut app);

    let resp: ConfigResponse = app
        .wrap()
        .query_wasm_smart(&vesting_instance, &QueryMsg::Config {})
        .unwrap();

    // Check config
    assert_eq!(init_msg.owner, resp.owner);
    assert_eq!(init_msg.refund_recipient, resp.refund_recipient);
    assert_eq!(init_msg.token, resp.token);
    assert_eq!(
        init_msg.default_unlock_schedule,
        resp.default_unlock_schedule
    );

    // Check state
    let resp: StateResponse = app
        .wrap()
        .query_wasm_smart(&vesting_instance, &QueryMsg::State {})
        .unwrap();

    assert_eq!(Uint128::zero(), resp.total_deposited);
    assert_eq!(Uint128::zero(), resp.remaining_tokens);
}

#[test]
fn test_transfer_ownership() {
    let mut app = mock_app();
    let (vesting_instance, _, init_msg) = init_contracts(&mut app);

    // ######    ERROR :: Unauthorized     ######

    let err = app
        .execute_contract(
            Addr::unchecked("not_owner".to_string()),
            vesting_instance.clone(),
            &ExecuteMsg::TransferOwnership {
                new_owner: "new_owner".to_string(),
            },
            &[],
        )
        .unwrap_err();
    assert_eq!(err.root_cause().to_string(), "Generic error: Unauthorized");

    // ######    SUCCESSFULLY TRANSFERS OWNERSHIP    ######

    app.execute_contract(
        Addr::unchecked(OWNER.to_string()),
        vesting_instance.clone(),
        &ExecuteMsg::TransferOwnership {
            new_owner: "new_owner".to_string(),
        },
        &[],
    )
    .unwrap();

    let resp: ConfigResponse = app
        .wrap()
        .query_wasm_smart(&vesting_instance, &QueryMsg::Config {})
        .unwrap();

    // Check config
    assert_eq!("new_owner".to_string(), resp.owner);
    assert_eq!(init_msg.refund_recipient, resp.refund_recipient);
    assert_eq!(init_msg.token, resp.token);
    assert_eq!(
        init_msg.default_unlock_schedule,
        resp.default_unlock_schedule
    );
}

#[test]
fn test_create_allocations() {
    let mut app = mock_app();
    let (vesting_instance, whale_instance, _init_msg) = init_contracts(&mut app);

    mint_some_whale(
        &mut app,
        Addr::unchecked(OWNER.clone()),
        whale_instance.clone(),
        Uint128::new(1_000_000_000_000000),
        OWNER.to_string(),
    );

    let mut allocations: Vec<(String, AllocationInfo)> = vec![];
    allocations.push((
        "investor_1".to_string(),
        AllocationInfo {
            total_amount: Uint128::from(5_000_000_000000u64),
            withdrawn_amount: Uint128::from(0u64),
            vest_schedule: Schedule {
                start_time: 1642402274u64,
                cliff: 0u64,
                duration: 31536000u64,
            },
            unlock_schedule: None,
            canceled: false,
        },
    ));
    allocations.push((
        "advisor_1".to_string(),
        AllocationInfo {
            total_amount: Uint128::from(5_000_000_000000u64),
            withdrawn_amount: Uint128::from(0u64),
            vest_schedule: Schedule {
                start_time: 1642402274u64,
                cliff: 7776000u64,
                duration: 31536000u64,
            },
            unlock_schedule: None,
            canceled: false,
        },
    ));
    allocations.push((
        "team_1".to_string(),
        AllocationInfo {
            total_amount: Uint128::from(5_000_000_000000u64),
            withdrawn_amount: Uint128::from(0u64),
            vest_schedule: Schedule {
                start_time: 1642402274u64,
                cliff: 7776000u64,
                duration: 31536000u64,
            },
            unlock_schedule: None,
            canceled: false,
        },
    ));

    // ######    ERROR :: Only owner can create allocations     ######

    mint_some_whale(
        &mut app,
        Addr::unchecked(OWNER.clone()),
        whale_instance.clone(),
        Uint128::new(1_000),
        "not_owner".to_string(),
    );

    let mut err = app
        .execute_contract(
            Addr::unchecked("not_owner".to_string()),
            whale_instance.clone(),
            &cw20::Cw20ExecuteMsg::Send {
                contract: vesting_instance.clone().to_string(),
                amount: Uint128::from(1_000u64),
                msg: to_binary(&ReceiveMsg::CreateAllocations {
                    allocations: allocations.clone(),
                })
                .unwrap(),
            },
            &[],
        )
        .unwrap_err();
    assert_eq!(
        err.root_cause().to_string(),
        "Generic error: Only owner can create allocations"
    );

    // ######    ERROR :: Only WHALE Token can be  can be deposited     ######

    // Instantiate WHALE Token Contract
    let not_whale_token_contract = Box::new(ContractWrapper::new(
        cw20_base::contract::execute,
        cw20_base::contract::instantiate,
        cw20_base::contract::query,
    ));

    let not_whale_token_code_id = app.store_code(not_whale_token_contract);

    let msg = cw20_base::msg::InstantiateMsg {
        name: String::from("Fake Whale token"),
        symbol: String::from("NWHALE"),
        decimals: 6,
        initial_balances: vec![],
        mint: Some(cw20::MinterResponse {
            minter: OWNER.to_string(),
            cap: None,
        }),
        marketing: None,
    };

    let not_whale_token_instance = app
        .instantiate_contract(
            not_whale_token_code_id,
            Addr::unchecked(OWNER.clone()),
            &msg,
            &[],
            String::from("WHALE"),
            None,
        )
        .unwrap();

    app.execute_contract(
        Addr::unchecked(OWNER.clone()),
        not_whale_token_instance.clone(),
        &cw20::Cw20ExecuteMsg::Mint {
            recipient: OWNER.clone().to_string(),
            amount: Uint128::from(15_000_000_000000u64),
        },
        &[],
    )
    .unwrap();

    err = app
        .execute_contract(
            Addr::unchecked(OWNER.clone()),
            not_whale_token_instance.clone(),
            &cw20::Cw20ExecuteMsg::Send {
                contract: vesting_instance.clone().to_string(),
                amount: Uint128::from(15_000_000_000000u64),
                msg: to_binary(&ReceiveMsg::CreateAllocations {
                    allocations: allocations.clone(),
                })
                .unwrap(),
            },
            &[],
        )
        .unwrap_err();
    assert_eq!(
        err.root_cause().to_string(),
        "Generic error: Only WHALE token can be deposited"
    );

    // ######    ERROR :: WHALE deposit amount mismatch     ######

    err = app
        .execute_contract(
            Addr::unchecked(OWNER.clone()),
            whale_instance.clone(),
            &cw20::Cw20ExecuteMsg::Send {
                contract: vesting_instance.clone().to_string(),
                amount: Uint128::from(15_000_000_000001u64),
                msg: to_binary(&ReceiveMsg::CreateAllocations {
                    allocations: allocations.clone(),
                })
                .unwrap(),
            },
            &[],
        )
        .unwrap_err();
    assert_eq!(
        err.root_cause().to_string(),
        "Generic error: WHALE deposit amount mismatch"
    );

    // ######    SUCCESSFULLY CREATES ALLOCATIONS    ######

    app.execute_contract(
        Addr::unchecked(OWNER.clone()),
        whale_instance.clone(),
        &cw20::Cw20ExecuteMsg::Send {
            contract: vesting_instance.clone().to_string(),
            amount: Uint128::from(15_000_000_000000u64),
            msg: to_binary(&ReceiveMsg::CreateAllocations {
                allocations: allocations.clone(),
            })
            .unwrap(),
        },
        &[],
    )
    .unwrap();

    // Check state
    let resp: StateResponse = app
        .wrap()
        .query_wasm_smart(&vesting_instance, &QueryMsg::State {})
        .unwrap();
    assert_eq!(resp.total_deposited, Uint128::from(15_000_000_000000u64));
    assert_eq!(resp.remaining_tokens, Uint128::from(15_000_000_000000u64));

    let _resp: StateResponse = app
        .wrap()
        .query_wasm_smart(&vesting_instance, &QueryMsg::State {})
        .unwrap();

    // Check allocation #1
    let resp: AllocationResponse = app
        .wrap()
        .query_wasm_smart(
            &vesting_instance,
            &QueryMsg::Allocation {
                account: "investor_1".to_string(),
            },
        )
        .unwrap();
    assert_eq!(resp.total_amount, Uint128::from(5_000_000_000000u64));
    assert_eq!(resp.withdrawn_amount, Uint128::from(0u64));
    assert_eq!(
        resp.vest_schedule,
        Schedule {
            start_time: 1642402274u64,
            cliff: 0u64,
            duration: 31536000u64
        }
    );
    assert_eq!(resp.unlock_schedule, None);

    // Check allocation #2
    let resp: AllocationResponse = app
        .wrap()
        .query_wasm_smart(
            &vesting_instance,
            &QueryMsg::Allocation {
                account: "advisor_1".to_string(),
            },
        )
        .unwrap();
    assert_eq!(resp.total_amount, Uint128::from(5_000_000_000000u64));
    assert_eq!(resp.withdrawn_amount, Uint128::from(0u64));
    assert_eq!(
        resp.vest_schedule,
        Schedule {
            start_time: 1642402274u64,
            cliff: 7776000u64,
            duration: 31536000u64
        }
    );
    assert_eq!(resp.unlock_schedule, None);

    // Check allocation #3
    let resp: AllocationResponse = app
        .wrap()
        .query_wasm_smart(
            &vesting_instance,
            &QueryMsg::Allocation {
                account: "team_1".to_string(),
            },
        )
        .unwrap();
    assert_eq!(resp.total_amount, Uint128::from(5_000_000_000000u64));
    assert_eq!(resp.withdrawn_amount, Uint128::from(0u64));
    assert_eq!(
        resp.vest_schedule,
        Schedule {
            start_time: 1642402274u64,
            cliff: 7776000u64,
            duration: 31536000u64
        }
    );
    assert_eq!(resp.unlock_schedule, None);

    // ######    ERROR :: Allocation already exists for user {}     ######

    err = app
        .execute_contract(
            Addr::unchecked(OWNER.clone()),
            whale_instance.clone(),
            &cw20::Cw20ExecuteMsg::Send {
                contract: vesting_instance.clone().to_string(),
                amount: Uint128::from(5_000_000_000000u64),
                msg: to_binary(&ReceiveMsg::CreateAllocations {
                    allocations: vec![allocations[0].clone()],
                })
                .unwrap(),
            },
            &[],
        )
        .unwrap_err();
    assert_eq!(
        err.root_cause().to_string(),
        "Generic error: Allocation already exists for user investor_1"
    );

    // ######    ERROR :: Allocation already exists for user {}     ######

    err = app
        .execute_contract(
            Addr::unchecked(OWNER.clone()),
            whale_instance.clone(),
            &cw20::Cw20ExecuteMsg::Send {
                contract: vesting_instance.clone().to_string(),
                amount: Uint128::from(5_000_000_000000u64),
                msg: to_binary(&ReceiveMsg::CreateAllocations {
                    allocations: vec![(
                        "team_2".to_string(),
                        AllocationInfo {
                            total_amount: Uint128::from(5_000_000_000000u64),
                            withdrawn_amount: Uint128::from(0u64),
                            vest_schedule: Schedule {
                                start_time: 1642402274u64,
                                cliff: 7776000u64,
                                duration: 31536000u64,
                            },
                            unlock_schedule: Some(Schedule {
                                start_time: 1642402279u64,
                                cliff: 7776000u64,
                                duration: 31536000u64,
                            }),
                            canceled: false,
                        },
                    )],
                })
                .unwrap(),
            },
            &[],
        )
        .unwrap_err();
    assert_eq!(err.root_cause().to_string(), "Generic error: Invalid Allocation for team_2. Unlock schedule needs to begin before vest schedule");
}

#[test]
fn test_withdraw() {
    let mut app = mock_app();
    let (vesting_instance, whale_instance, _) = init_contracts(&mut app);

    mint_some_whale(
        &mut app,
        Addr::unchecked(OWNER.clone()),
        whale_instance.clone(),
        Uint128::new(1_000_000_000_000000),
        OWNER.to_string(),
    );

    let mut allocations: Vec<(String, AllocationInfo)> = vec![];
    allocations.push((
        "investor_1".to_string(),
        AllocationInfo {
            total_amount: Uint128::from(5_000_000_000000u64),
            withdrawn_amount: Uint128::from(0u64),
            vest_schedule: Schedule {
                start_time: 1642402274u64,
                cliff: 0u64,
                duration: 31536000u64,
            },
            unlock_schedule: None,
            canceled: false,
        },
    ));
    allocations.push((
        "advisor_1".to_string(),
        AllocationInfo {
            total_amount: Uint128::from(5_000_000_000000u64),
            withdrawn_amount: Uint128::from(0u64),
            vest_schedule: Schedule {
                start_time: 1642402274u64,
                cliff: 7776000u64,
                duration: 31536000u64,
            },
            unlock_schedule: None,
            canceled: false,
        },
    ));
    allocations.push((
        "team_1".to_string(),
        AllocationInfo {
            total_amount: Uint128::from(5_000_000_000000u64),
            withdrawn_amount: Uint128::from(0u64),
            vest_schedule: Schedule {
                start_time: 1642402274u64,
                cliff: 7776000u64,
                duration: 31536000u64,
            },
            unlock_schedule: Some(Schedule {
                start_time: 1642400000u64,
                cliff: 7770000u64,
                duration: 31536000u64,
            }),
            canceled: false,
        },
    ));

    // SUCCESSFULLY CREATES ALLOCATIONS
    app.execute_contract(
        Addr::unchecked(OWNER.clone()),
        whale_instance.clone(),
        &cw20::Cw20ExecuteMsg::Send {
            contract: vesting_instance.clone().to_string(),
            amount: Uint128::from(15_000_000_000000u64),
            msg: to_binary(&ReceiveMsg::CreateAllocations {
                allocations: allocations.clone(),
            })
            .unwrap(),
        },
        &[],
    )
    .unwrap();

    // ######    ERROR :: Allocation doesn't exist    ######

    let err = app
        .execute_contract(
            Addr::unchecked(OWNER.clone()),
            vesting_instance.clone(),
            &ExecuteMsg::Withdraw {},
            &[],
        )
        .unwrap_err();
    assert_eq!(
        err.root_cause().to_string(),
        "abstract_os::vesting::AllocationInfo not found"
    );

    // ######    ERROR :: Withdrawals not allowed yet   ######

    app.update_block(|b| {
        b.height += 17280;
        b.time = Timestamp::from_seconds(1642402273)
    });

    let err = app
        .execute_contract(
            Addr::unchecked("investor_1".clone()),
            vesting_instance.clone(),
            &ExecuteMsg::Withdraw {},
            &[],
        )
        .unwrap_err();
    assert_eq!(
        err.root_cause().to_string(),
        "Generic error: Withdrawals not allowed yet"
    );

    // ######   SUCCESSFULLY WITHDRAWS WHALE #1   ######

    app.update_block(|b| {
        b.height += 17280;
        b.time = Timestamp::from_seconds(1642402275)
    });

    app.execute_contract(
        Addr::unchecked("investor_1".clone()),
        vesting_instance.clone(),
        &ExecuteMsg::Withdraw {},
        &[],
    )
    .unwrap();

    // Check allocation #1
    let resp: AllocationResponse = app
        .wrap()
        .query_wasm_smart(
            &vesting_instance,
            &QueryMsg::Allocation {
                account: "investor_1".to_string(),
            },
        )
        .unwrap();
    assert_eq!(resp.total_amount, Uint128::from(5_000_000_000000u64));
    assert_eq!(resp.withdrawn_amount, Uint128::from(158548u64));

    // ######    ERROR :: No unlocked WHALE to be withdrawn   ######

    let err = app
        .execute_contract(
            Addr::unchecked("investor_1".clone()),
            vesting_instance.clone(),
            &ExecuteMsg::Withdraw {},
            &[],
        )
        .unwrap_err();
    assert_eq!(
        err.root_cause().to_string(),
        "Generic error: No unlocked WHALE to be withdrawn"
    );

    // ######   SUCCESSFULLY WITHDRAWS WHALE #2   ######

    let resp: SimulateWithdrawResponse = app
        .wrap()
        .query_wasm_smart(
            &vesting_instance,
            &QueryMsg::SimulateWithdraw {
                account: "investor_1".to_string(),
                timestamp: Some(1642402285u64),
            },
        )
        .unwrap();
    assert_eq!(resp.total_tokens_locked, Uint128::from(5_000_000_000000u64));
    assert_eq!(
        resp.total_tokens_unlocked,
        Uint128::from(5_000_000_000000u64)
    );
    assert_eq!(resp.total_tokens_vested, Uint128::from(1744038u64));
    assert_eq!(resp.withdrawn_amount, Uint128::from(158548u64));
    assert_eq!(resp.withdrawable_amount, Uint128::from(1585490u64));

    app.update_block(|b| {
        b.height += 17280;
        b.time = Timestamp::from_seconds(1642402285)
    });

    app.execute_contract(
        Addr::unchecked("investor_1".clone()),
        vesting_instance.clone(),
        &ExecuteMsg::Withdraw {},
        &[],
    )
    .unwrap();

    let resp: AllocationResponse = app
        .wrap()
        .query_wasm_smart(
            &vesting_instance,
            &QueryMsg::Allocation {
                account: "investor_1".to_string(),
            },
        )
        .unwrap();
    assert_eq!(resp.total_amount, Uint128::from(5_000_000_000000u64));
    assert_eq!(resp.withdrawn_amount, Uint128::from(1744038u64));

    // ######    ERROR :: No unlocked WHALE to be withdrawn   ######

    let err = app
        .execute_contract(
            Addr::unchecked("investor_1".clone()),
            vesting_instance.clone(),
            &ExecuteMsg::Withdraw {},
            &[],
        )
        .unwrap_err();
    assert_eq!(
        err.root_cause().to_string(),
        "Generic error: No unlocked WHALE to be withdrawn"
    );

    // ######   SUCCESSFULLY WITHDRAWS WHALE #3   ######

    app.update_block(|b| {
        b.height += 17280;
        b.time = Timestamp::from_seconds(1650170001)
    });

    let resp: SimulateWithdrawResponse = app
        .wrap()
        .query_wasm_smart(
            &vesting_instance,
            &QueryMsg::SimulateWithdraw {
                account: "team_1".to_string(),
                timestamp: None,
            },
        )
        .unwrap();
    assert_eq!(resp.total_tokens_locked, Uint128::from(5_000_000_000000u64));
    assert_eq!(resp.total_tokens_unlocked, Uint128::from(1231925577118u64));
    assert_eq!(resp.total_tokens_vested, Uint128::from(1231565036783u64));
    assert_eq!(resp.withdrawn_amount, Uint128::from(0u64));
    assert_eq!(resp.withdrawable_amount, Uint128::from(0u64));

    app.update_block(|b| {
        b.height += 17280;
        b.time = Timestamp::from_seconds(1650178275)
    });

    let resp: SimulateWithdrawResponse = app
        .wrap()
        .query_wasm_smart(
            &vesting_instance,
            &QueryMsg::SimulateWithdraw {
                account: "team_1".to_string(),
                timestamp: None,
            },
        )
        .unwrap();
    assert_eq!(resp.total_tokens_locked, Uint128::from(5_000_000_000000u64));
    assert_eq!(resp.total_tokens_vested, Uint128::from(1232876870877u64));
    assert_eq!(resp.withdrawable_amount, Uint128::from(1232876870877u64));

    app.execute_contract(
        Addr::unchecked("team_1".clone()),
        vesting_instance.clone(),
        &ExecuteMsg::Withdraw {},
        &[],
    )
    .unwrap();

    let resp: AllocationResponse = app
        .wrap()
        .query_wasm_smart(
            &vesting_instance,
            &QueryMsg::Allocation {
                account: "team_1".to_string(),
            },
        )
        .unwrap();
    assert_eq!(resp.total_amount, Uint128::from(5_000_000_000000u64));
    assert_eq!(resp.withdrawn_amount, Uint128::from(1232876870877u64));
}

#[test]
fn test_terminate() {
    let mut app = mock_app();
    let (vesting_instance, whale_instance, _) = init_contracts(&mut app);

    mint_some_whale(
        &mut app,
        Addr::unchecked(OWNER.clone()),
        whale_instance.clone(),
        Uint128::new(1_000_000_000_000000),
        OWNER.to_string(),
    );

    let mut allocations: Vec<(String, AllocationInfo)> = vec![];
    allocations.push((
        "investor_1".to_string(),
        AllocationInfo {
            total_amount: Uint128::from(5_000_000_000000u64),
            withdrawn_amount: Uint128::from(0u64),
            vest_schedule: Schedule {
                start_time: 1642402274u64,
                cliff: 0u64,
                duration: 31536000u64,
            },
            unlock_schedule: None,
            canceled: false,
        },
    ));
    allocations.push((
        "advisor_1".to_string(),
        AllocationInfo {
            total_amount: Uint128::from(5_000_000_000000u64),
            withdrawn_amount: Uint128::from(0u64),
            vest_schedule: Schedule {
                start_time: 1642402274u64,
                cliff: 7776000u64,
                duration: 31536000u64,
            },
            unlock_schedule: None,
            canceled: false,
        },
    ));
    allocations.push((
        "team_1".to_string(),
        AllocationInfo {
            total_amount: Uint128::from(5_000_000_000000u64),
            withdrawn_amount: Uint128::from(0u64),
            vest_schedule: Schedule {
                start_time: 1642402274u64,
                cliff: 7776000u64,
                duration: 31536000u64,
            },
            unlock_schedule: Some(Schedule {
                start_time: 1642400000u64,
                cliff: 7770000u64,
                duration: 31536000u64,
            }),
            canceled: false,
        },
    ));

    // SUCCESSFULLY CREATES ALLOCATIONS
    app.execute_contract(
        Addr::unchecked(OWNER.clone()),
        whale_instance.clone(),
        &cw20::Cw20ExecuteMsg::Send {
            contract: vesting_instance.clone().to_string(),
            amount: Uint128::from(15_000_000_000000u64),
            msg: to_binary(&ReceiveMsg::CreateAllocations {
                allocations: allocations.clone(),
            })
            .unwrap(),
        },
        &[],
    )
    .unwrap();

    // ######    ERROR :: Unauthorized    ######

    let err = app
        .execute_contract(
            Addr::unchecked("NOT_OWNER".to_string()),
            vesting_instance.clone(),
            &ExecuteMsg::Terminate {
                user_address: "investor_1".to_string(),
            },
            &[],
        )
        .unwrap_err();
    assert_eq!(err.root_cause().to_string(), "Generic error: Unauthorized");

    // ######    ERROR :: No WHALE available to refund.    ######

    let err = app
        .execute_contract(
            Addr::unchecked(OWNER.clone()),
            vesting_instance.clone(),
            &ExecuteMsg::Terminate {
                user_address: "investor_1".to_string(),
            },
            &[],
        )
        .unwrap_err();
    assert_eq!(
        err.root_cause().to_string(),
        "Generic error: No WHALE available to refund."
    );

    // ######    SUCCESSFULLY TERMINATES ALLOCATION   ######

    app.update_block(|b| {
        b.height += 17280;
        b.time = Timestamp::from_seconds(1642402273)
    });

    app.execute_contract(
        Addr::unchecked(OWNER.clone()),
        vesting_instance.clone(),
        &ExecuteMsg::Terminate {
            user_address: "team_1".to_string(),
        },
        &[],
    )
    .unwrap();

    // Try to terminate again, should err
    app.execute_contract(
        Addr::unchecked(OWNER.clone()),
        vesting_instance.clone(),
        &ExecuteMsg::Terminate {
            user_address: "team_1".to_string(),
        },
        &[],
    )
    .unwrap_err();

    let resp: SimulateWithdrawResponse = app
        .wrap()
        .query_wasm_smart(
            &vesting_instance,
            &QueryMsg::SimulateWithdraw {
                account: "team_1".to_string(),
                timestamp: None,
            },
        )
        .unwrap();
    assert_eq!(resp.total_tokens_locked, Uint128::from(360381785u64));
    assert_eq!(resp.total_tokens_unlocked, Uint128::from(360381785u64));
    assert_eq!(resp.total_tokens_vested, Uint128::from(0u64));
    assert_eq!(resp.withdrawn_amount, Uint128::from(0u64));
    assert_eq!(resp.withdrawable_amount, Uint128::from(0u64));

    app.update_block(|b| {
        b.height += 17280;
        b.time = Timestamp::from_seconds(1642702273)
    });

    let resp: SimulateWithdrawResponse = app
        .wrap()
        .query_wasm_smart(
            &vesting_instance,
            &QueryMsg::SimulateWithdraw {
                account: "team_1".to_string(),
                timestamp: None,
            },
        )
        .unwrap();
    assert_eq!(resp.total_tokens_locked, Uint128::from(360381785u64));
    assert_eq!(resp.total_tokens_unlocked, Uint128::from(360381785u64));
    assert_eq!(resp.total_tokens_vested, Uint128::from(3428278u64));
    assert_eq!(resp.withdrawn_amount, Uint128::from(0u64));
    assert_eq!(resp.withdrawable_amount, Uint128::from(0u64));
}
