use abstract_os::subscription::state::{Compensation, ContributionState, SubscriptionState, MONTH};
use abstract_os::{objects::module::ModuleInfo, SUBSCRIPTION};
use abstract_os::{subscription as msgs, subscription::state};
use anyhow::Result as AnyResult;
use cosmwasm_std::{Addr, BlockInfo, Decimal, Uint128, Uint64};
use cw_controllers::AdminError;
use cw_multi_test::{App, ContractWrapper, Executor};

use crate::tests::common::RANDOM_USER;
use crate::tests::testing_infrastructure::env::{exec_msg_on_manager, mint_tokens, token_balance};

use super::testing_infrastructure::env::init_os;
use super::{
    common::TEST_CREATOR,
    testing_infrastructure::env::{get_os_state, mock_app, register_module, AbstractEnv},
};

pub fn register_subscription(
    app: &mut App,
    sender: &Addr,
    version_control: &Addr,
) -> AnyResult<()> {
    let module = ModuleInfo {
        name: SUBSCRIPTION.into(),
        version: None,
    };

    let contract = Box::new(
        ContractWrapper::new_with_empty(
            subscription::contract::execute,
            subscription::contract::instantiate,
            subscription::contract::query,
        )
        .with_migrate_empty(subscription::contract::migrate),
    );
    register_module(app, &sender, &version_control, module, contract).unwrap();
    Ok(())
}

#[test]
fn proper_initialization() {
    let mut app = mock_app();
    let sender = Addr::unchecked(TEST_CREATOR);
    let env = AbstractEnv::new(&mut app, &sender);

    let os_state = get_os_state(&app, &env.os_store, &0u32).unwrap();

    // OS 0 has proxy and subscriber module
    assert_eq!(os_state.len(), 2);

    let subscription_addr = os_state.get(SUBSCRIPTION).unwrap();

    let config: msgs::ConfigResponse = app
        .wrap()
        .query_wasm_smart(subscription_addr, &msgs::QueryMsg::Config {})
        .unwrap();

    assert_eq!(
        config.contribution,
        state::ContributionConfig {
            protocol_income_share: Decimal::percent(10),
            emission_user_share: Decimal::percent(25),
            max_emissions_multiple: Decimal::from_ratio(2u128, 1u128),
            project_token: env.native_contracts.token,
            emissions_amp_factor: Uint128::new(680000000),
            emissions_offset: Uint128::new(52000),
            base_denom: "uusd".to_string(),
        }
    );

    assert_eq!(
        config.subscription,
        state::SubscriptionConfig {
            version_control_address: env.native_contracts.version_control,
            factory_address: env.native_contracts.os_factory,
            payment_asset: cw_asset::AssetInfoBase::native("uusd"),
            subscription_cost: Uint64::new(100)
        }
    );

    let state: msgs::StateResponse = app
        .wrap()
        .query_wasm_smart(subscription_addr, &msgs::QueryMsg::State {})
        .unwrap();

    assert_eq!(
        state.contribution,
        state::ContributionState {
            target: Uint64::zero(),
            expense: Uint64::zero(),
            total_weight: Uint128::zero(),
            emissions: Uint128::zero(),
            next_pay_day: Uint64::from(app.block_info().time.seconds() - 6)
        }
    );

    assert_eq!(
        state.subscription,
        state::SubscriptionState {
            income: Uint64::zero(),
            active_subs: 0,
            collected: false,
        }
    );
}

#[test]
fn add_and_remove_contributors() {
    let mut app: App = mock_app();
    let sender = Addr::unchecked(TEST_CREATOR);
    let random_user = Addr::unchecked(RANDOM_USER);

    let env = AbstractEnv::new(&mut app, &sender);

    let os_state = get_os_state(&app, &env.os_store, &0u32).unwrap();

    // OS 0 has proxy and subscriber module
    assert_eq!(os_state.len(), 2);

    let subscription_addr = os_state.get(SUBSCRIPTION).unwrap();
    let manager_addr = env.os_store.get(&0).unwrap().manager.clone();

    let msg = msgs::ExecuteMsg::UpdateContributor {
        contributor_addr: TEST_CREATOR.to_string(),
        compensation: Compensation {
            base: 1000,
            weight: 100,
            expiration: app
                .block_info()
                .time
                .plus_seconds(MONTH * 2)
                .seconds()
                .into(),
            // These fields should get overwritten
            next_pay_day: 0u64.into(),
        },
    };

    let resp = app
        .execute_contract(sender.clone(), subscription_addr.clone(), &msg, &[])
        .unwrap_err();

    assert_eq!(
        AdminError::NotAdmin {}.to_string(),
        resp.source().unwrap().to_string()
    );

    exec_msg_on_manager(&mut app, &sender, &manager_addr, SUBSCRIPTION, &msg).unwrap();

    let state: msgs::StateResponse = app
        .wrap()
        .query_wasm_smart(subscription_addr, &msgs::QueryMsg::State {})
        .unwrap();

    assert_eq!(
        state.contribution,
        state::ContributionState {
            target: Uint64::new(1000),
            expense: Uint64::zero(),
            total_weight: Uint128::new(100),
            emissions: Uint128::zero(),
            next_pay_day: Uint64::from(app.block_info().time.seconds() - 6)
        }
    );

    let msg = msgs::ExecuteMsg::UpdateContributor {
        contributor_addr: RANDOM_USER.to_string(),
        compensation: Compensation {
            base: 2000,
            weight: 200,
            expiration: app
                .block_info()
                .time
                .plus_seconds(MONTH * 2)
                .seconds()
                .into(),
            // This field should get overwritten
            next_pay_day: 0u64.into(),
        },
    };

    exec_msg_on_manager(&mut app, &sender, &manager_addr, SUBSCRIPTION, &msg).unwrap();

    let resp: msgs::ContributorStateResponse = app
        .wrap()
        .query_wasm_smart(
            subscription_addr,
            &msgs::QueryMsg::ContributorState {
                contributor_addr: random_user.to_string(),
            },
        )
        .unwrap();

    assert_eq!(
        resp.compensation,
        Compensation {
            base: 2000,
            weight: 200,
            expiration: app
                .block_info()
                .time
                .plus_seconds(MONTH * 2)
                .seconds()
                .into(),
            next_pay_day: app
                .block_info()
                .time
                .plus_seconds(MONTH - 6)
                .seconds()
                .into(),
        }
    );

    let msg = msgs::ExecuteMsg::RemoveContributor {
        contributor_addr: TEST_CREATOR.to_string(),
    };

    let resp =
        exec_msg_on_manager(&mut app, &random_user, &manager_addr, SUBSCRIPTION, &msg).unwrap_err();
    // Only OS root can change stuff on module
    assert_eq!(
        AdminError::NotAdmin {}.to_string(),
        resp.source().unwrap().to_string()
    );

    exec_msg_on_manager(&mut app, &sender, &manager_addr, SUBSCRIPTION, &msg).unwrap();

    let state: msgs::StateResponse = app
        .wrap()
        .query_wasm_smart(subscription_addr, &msgs::QueryMsg::State {})
        .unwrap();

    assert_eq!(
        state.contribution,
        state::ContributionState {
            target: Uint64::new(2000),
            expense: Uint64::zero(),
            total_weight: Uint128::new(200),
            emissions: Uint128::zero(),
            next_pay_day: Uint64::from(app.block_info().time.seconds() - 6)
        }
    );
}

/// On creation the contract next-pay-day is set to 6 sec before the current time.
/// In the case that now > next-pay-day we want to check that
/// 1. New OS can not claim emission
/// 2. Contributor can not collect income
/// 3. We can successfully collect the income.
/// This is tested in actions_before_first_month()
///
/// While collecting the income we want to check that
/// 4. New OS's can't be created
/// 5. OS cannot claim emissions
///
/// After collecting the income the next-pay-day will be moved by one MONTH

#[test]
fn actions_before_first_month() {
    let mut app: App = mock_app();
    let sender = Addr::unchecked(TEST_CREATOR);
    let _random_user = Addr::unchecked(RANDOM_USER);
    let mut env = AbstractEnv::new(&mut app, &sender);

    let os_state = get_os_state(&app, &env.os_store, &0u32).unwrap();
    let subscription_addr = os_state.get(SUBSCRIPTION).unwrap();
    let manager_addr = env.os_store.get(&0).unwrap().manager.clone();
    let proxy_addr = env.os_store.get(&0).unwrap().proxy.clone();

    mint_tokens(
        &mut app,
        &sender,
        &env.native_contracts.token,
        1000000000u128.into(),
        proxy_addr.to_string(),
    );

    let _contributing_os = 12u32;
    for _ in 0..50u32 {
        init_os(&mut app, &sender, &env.native_contracts, &mut env.os_store).unwrap();
    }

    // Payments got forwarded to os 0
    let abstract_balance = app.wrap().query_balance(&proxy_addr, "uusd").unwrap();
    // 50 os' were created
    assert_eq!(abstract_balance.amount.u128(), 5_000u128);

    let msg = msgs::ExecuteMsg::ClaimEmissions { os_id: 2 };
    let resp = app
        .execute_contract(sender.clone(), subscription_addr.clone(), &msg, &[])
        // Can only claim after tallying
        .unwrap_err();

    assert_eq!(
        "cannot claim emissions before income is collected".to_string(),
        resp.source().unwrap().to_string()
    );
    // Locks the client map
    let msg = msgs::ExecuteMsg::CollectSubs { page_limit: None };
    let _resp = app
        .execute_contract(sender.clone(), subscription_addr.clone(), &msg, &[])
        .unwrap();

    let msg = msgs::ExecuteMsg::ClaimEmissions { os_id: 2 };
    let resp = app
        .execute_contract(sender.clone(), subscription_addr.clone(), &msg, &[])
        // Can only claim after tallying
        .unwrap_err();

    assert_eq!(
        resp.source().unwrap().to_string(),
        "cannot claim emissions before income is collected"
    );

    // Map is locked so no new OS's are allowed to be created
    let resp = init_os(&mut app, &sender, &env.native_contracts, &mut env.os_store).unwrap_err();
    assert_eq!(
        resp.source().unwrap().source().unwrap().to_string(),
        "Generic error: Can not save to map while locked. Proceed with operation first."
    );

    let msg = msgs::ExecuteMsg::ClaimCompensation {
        contributor: None,
        page_limit: None,
    };
    let resp = app
        .execute_contract(sender.clone(), subscription_addr.clone(), &msg, &[])
        .unwrap_err();
    // No contributor registered
    assert_eq!(
        resp.source().unwrap().to_string(),
        "income target is zero, no contributions can be paid out."
    );

    let msg = msgs::ExecuteMsg::UpdateContributor {
        contributor_addr: TEST_CREATOR.to_string(),
        compensation: Compensation {
            base: 1000,
            weight: 100,
            expiration: app
                .block_info()
                .time
                .plus_seconds(MONTH * 3)
                .seconds()
                .into(),
            // This field gets overwritten
            next_pay_day: 0u64.into(),
        },
    };

    let resp = app
        .execute_contract(sender.clone(), subscription_addr.clone(), &msg, &[])
        .unwrap_err();
    // Config changes always go through Manager contract
    assert_eq!(resp.source().unwrap().to_string(), "Caller is not admin");

    exec_msg_on_manager(&mut app, &sender, &manager_addr, SUBSCRIPTION, &msg).unwrap();

    // page over subscribers to collect income
    collect_subs_until_done(&mut app, &sender, &subscription_addr);

    let state: msgs::StateResponse = app
        .wrap()
        .query_wasm_smart(subscription_addr, &msgs::QueryMsg::State {})
        .unwrap();

    assert_eq!(
        state.subscription,
        SubscriptionState {
            income: 5000u64.into(),
            active_subs: 50,
            // collected is used internally
            collected: false
        }
    );

    let next_month = app
        .block_info()
        .clone()
        .time
        .plus_seconds(MONTH - 6)
        .seconds();

    assert_eq!(
        state.contribution,
        ContributionState {
            target: 1000u64.into(),
            expense: 1000u64.into(),
            total_weight: 100u64.into(),
            // Checked with spreadsheet
            emissions: 12830u64.into(),
            next_pay_day: next_month.into()
        }
    );
    send_compensations_until_done(&mut app, &sender, &subscription_addr);

    let msg = msgs::ExecuteMsg::ClaimEmissions { os_id: 2 };
    let _resp = app
        .execute_contract(sender.clone(), subscription_addr.clone(), &msg, &[])
        .unwrap();
    // Proxy has claimed assets
    let new_balance = token_balance(
        &app,
        &env.native_contracts.token,
        &env.os_store.get(&2).unwrap().proxy,
    );
    // user_emissions_share * total_emissions / amount of users
    // 64 = 0.25 * 12830 / 50
    assert_eq!(new_balance, 64);

    let msg = msgs::ExecuteMsg::ClaimCompensation {
        contributor: Some(sender.to_string()),
        page_limit: None,
    };
    let resp = app
        .execute_contract(sender.clone(), subscription_addr.clone(), &msg, &[])
        .unwrap_err();
    // Contributor has to wait one month before he can start claiming pay
    assert_eq!(
        resp.source().unwrap().to_string(),
        "Generic error: You cant claim before your next pay day."
    );
}

/// Here we test what happens if users and contributors don't claim for a month.
/// For users this should mean that they lose their emission claim.
/// For contributors it means they will lose their compensation

#[test]
fn actions_after_first_month() {
    let mut app: App = mock_app();
    let sender = Addr::unchecked(TEST_CREATOR);
    let _random_user = Addr::unchecked(RANDOM_USER);
    let mut env = AbstractEnv::new(&mut app, &sender);

    let os_state = get_os_state(&app, &env.os_store, &0u32).unwrap();
    let subscription_addr = os_state.get(SUBSCRIPTION).unwrap();
    let manager_addr = env.os_store.get(&0).unwrap().manager.clone();
    let proxy_addr = env.os_store.get(&0).unwrap().proxy.clone();

    mint_tokens(
        &mut app,
        &sender,
        &env.native_contracts.token,
        1000000000u128.into(),
        proxy_addr.to_string(),
    );

    for _ in 0..50u32 {
        init_os(&mut app, &sender, &env.native_contracts, &mut env.os_store).unwrap();
    }

    let msg = msgs::ExecuteMsg::UpdateContributor {
        contributor_addr: TEST_CREATOR.to_string(),
        compensation: Compensation {
            base: 1000,
            weight: 100,
            expiration: app
                .block_info()
                .time
                .plus_seconds(MONTH * 3)
                .seconds()
                .into(),
            // This field gets overwritten
            next_pay_day: 0u64.into(),
        },
    };

    exec_msg_on_manager(&mut app, &sender, &manager_addr, SUBSCRIPTION, &msg).unwrap();

    collect_subs_until_done(&mut app, &sender, &subscription_addr);

    app.update_block(add_month);

    collect_subs_until_done(&mut app, &sender, &subscription_addr);

    let state: msgs::StateResponse = app
        .wrap()
        .query_wasm_smart(subscription_addr, &msgs::QueryMsg::State {})
        .unwrap();

    // no one paid so income back to 0.
    assert_eq!(
        state.subscription,
        SubscriptionState {
            income: 0u64.into(),
            active_subs: 0,
            // collected is used internally
            collected: false
        }
    );

    let next_month = app
        .block_info()
        .clone()
        .time
        .plus_seconds(MONTH - 6)
        .seconds();

    assert_eq!(
        state.contribution,
        ContributionState {
            target: 1000u64.into(),
            expense: 0u64.into(),
            total_weight: 100u64.into(),
            // Checked with spreadsheet
            emissions: 25660u64.into(),
            next_pay_day: next_month.into()
        }
    );

    let msg = msgs::ExecuteMsg::ClaimCompensation {
        contributor: Some(sender.to_string()),
        page_limit: None,
    };
    let _resp = app
        .execute_contract(sender.clone(), subscription_addr.clone(), &msg, &[])
        .unwrap();

    // contributor has claimed assets
    let _new_balance = token_balance(&app, &env.native_contracts.token, &sender);
    // 50/50 emission split
}
// #[test]
// fn add_subscribers_contributors() {
//     let mut app: App = mock_app();
//     let sender = Addr::unchecked(TEST_CREATOR);
//     let random_user = Addr::unchecked(RANDOM_USER);
//     app.init_bank_balance(&sender, vec![Coin::new(1_000_000_000, "uusd")])
//         .unwrap();
//     app.init_bank_balance(&random_user, vec![Coin::new(1_000_000_000, "uusd")])
//         .unwrap();
//     let mut env = AbstractEnv::new(&mut app, &sender);

//     let os_state = get_os_state(&app, &env.os_store, &0u32).unwrap();
//     let subscription_addr = os_state.get(SUBSCRIPTION).unwrap();
//     let manager_addr = env.os_store.get(&0).unwrap().manager.clone();
//     let proxy_addr = env.os_store.get(&0).unwrap().proxy.clone();

//     mint_tokens(
//         &mut app,
//         &sender,
//         &env.native_contracts.token,
//         1000000000u128.into(),
//         proxy_addr.to_string(),
//     );

//     for _ in 0..50u32 {
//         init_os(&mut app, &sender, &env.native_contracts, &mut env.os_store).unwrap();
//     }

//     // Payments got forwarded to os 0
//     let abstract_balance = app.wrap().query_balance(&proxy_addr, "uusd").unwrap();
//     // 50 os' were created
//     assert_eq!(abstract_balance.amount.u128(), 5_000u128);

//     let msg = msgs::ExecuteMsg::ClaimEmissions { os_id: 2 };
//     let resp = app
//         .execute_contract(sender.clone(), subscription_addr.clone(), &msg, &[])
//         // Can only claim after tallying
//         .unwrap_err();
//     assert_eq!(
//         resp.to_string(),
//         "cannot claim emissions before income is collected"
//     );

//     // Locks the client map
//     let msg = msgs::ExecuteMsg::CollectSubs { page_limit: None };
//     let resp = app
//         .execute_contract(sender.clone(), subscription_addr.clone(), &msg, &[])
//         .unwrap();

//     let msg = msgs::ExecuteMsg::ClaimEmissions { os_id: 2 };
//     let resp = app
//         .execute_contract(sender.clone(), subscription_addr.clone(), &msg, &[])
//         // Can only claim after tallying
//         .unwrap_err();

//     assert_eq!(
//         resp.to_string(),
//         "cannot claim emissions before income is collected"
//     );

//     // Map is locked so no new OS's are allowed to be created
//     let resp = init_os(&mut app, &sender, &env.native_contracts, &mut env.os_store).unwrap_err();
//     assert_eq!(
//         resp.to_string(),
//         "Generic error: Can not save to map while locked. Proceed with operation first."
//     );

//     let msg = msgs::ExecuteMsg::ClaimCompensation {
//         contributor: None,
//         page_limit: None,
//     };
//     let resp = app
//         .execute_contract(sender.clone(), subscription_addr.clone(), &msg, &[])
//         .unwrap_err();
//     // No contributor registered
//     assert_eq!(
//         resp.to_string(),
//         "income target is zero, no contributions can be paid out."
//     );

//     let msg = msgs::ExecuteMsg::UpdateContributor {
//         contributor_addr: TEST_CREATOR.to_string(),
//         compensation: Compensation {
//             base: 1000,
//             weight: 100,
//             expiration: app
//                 .block_info()
//                 .time
//                 .plus_seconds(MONTH * 3)
//                 .seconds()
//                 .into(),
//             // This field gets overwritten
//             next_pay_day: 0u64.into(),
//         },
//     };

//     let resp = app
//         .execute_contract(sender.clone(), subscription_addr.clone(), &msg, &[])
//         .unwrap_err();
//     // Config changes always go through Manager contract
//     assert_eq!(resp.to_string(), "Caller is not admin");

//     exec_msg_on_manager(&mut app, &sender, &manager_addr, SUBSCRIPTION, &msg).unwrap();

//     // page over subscribers to collect income
//     collect_subs_until_done(&mut app, &sender, &subscription_addr);

//     let state: msgs::StateResponse = app
//         .wrap()
//         .query_wasm_smart(subscription_addr, &msgs::QueryMsg::State {})
//         .unwrap();

//     assert_eq!(
//         state.subscription,
//         SubscriptionState {
//             income: 5000u64.into(),
//             active_subs: 50,
//             // collected is used internally
//             collected: false
//         }
//     );

//     let next_month = app
//         .block_info()
//         .clone()
//         .time
//         .plus_seconds(MONTH - 6)
//         .seconds();

//     assert_eq!(
//         state.contribution,
//         ContributionState {
//             target: 1000u64.into(),
//             expense: 1000u64.into(),
//             total_weight: 100u64.into(),
//             // Checked with spreadsheet
//             emissions: 12830u64.into(),
//             next_pay_day: next_month.into()
//         }
//     );

//     let msg = msgs::ExecuteMsg::ClaimEmissions { os_id: 2 };
//     let resp = app
//         .execute_contract(sender.clone(), subscription_addr.clone(), &msg, &[])
//         .unwrap();
//     // Proxy has claimed assets
//     let new_balance = token_balance(
//         &app,
//         &env.native_contracts.token,
//         &env.os_store.get(&2).unwrap().proxy,
//     );
//     assert_eq!(new_balance, 256);

//     let msg = msgs::ExecuteMsg::ClaimCompensation {
//         contributor: Some(sender.to_string()),
//         page_limit: None,
//     };
//     let resp = app
//         .execute_contract(sender.clone(), subscription_addr.clone(), &msg, &[])
//         .unwrap_err();
//     // Contributor has to wait one month before he can start claiming pay
//     assert_eq!(
//         resp.to_string(),
//         "Generic error: You cant claim before your next pay day."
//     );

//     app.update_block(add_month);

//     let msg = msgs::ExecuteMsg::ClaimCompensation {
//         contributor: Some(sender.to_string()),
//         page_limit: None,
//     };
//     let resp = app
//         .execute_contract(sender.clone(), subscription_addr.clone(), &msg, &[])
//         .unwrap();

//     // contributor has claimed assets
//     let new_balance = token_balance(&app, &env.native_contracts.token, &sender);
//     // 50/50 emission split
//     assert_eq!(state.contribution.emissions.u128() / 2, 6415);
// }

fn add_month(b: &mut BlockInfo) {
    b.time = b.time.plus_seconds(MONTH);
    b.height += MONTH / 6;
}

fn add_block(b: &mut BlockInfo) {
    b.time = b.time.plus_seconds(6);
    b.height += 1;
}

fn collect_subs_until_done(app: &mut App, sender: &Addr, subscription_addr: &Addr) {
    let mut state: msgs::StateResponse = app
        .wrap()
        .query_wasm_smart(subscription_addr, &msgs::QueryMsg::State {})
        .unwrap();

    while state.contribution.next_pay_day.u64() < app.block_info().time.seconds() {
        let msg = msgs::ExecuteMsg::CollectSubs { page_limit: None };
        let _resp = app
            .execute_contract(sender.clone(), subscription_addr.clone(), &msg, &[])
            .unwrap();
        let new_state: msgs::StateResponse = app
            .wrap()
            .query_wasm_smart(subscription_addr, &msgs::QueryMsg::State {})
            .unwrap();
        state.contribution.next_pay_day = new_state.contribution.next_pay_day;
    }
}
fn send_compensations_until_done(app: &mut App, sender: &Addr, subscription_addr: &Addr) {
    let mut done = false;

    while !done {
        let msg = msgs::ExecuteMsg::ClaimCompensation {
            contributor: None,
            page_limit: None,
        };
        let resp = app
            .execute_contract(sender.clone(), subscription_addr.clone(), &msg, &[])
            .unwrap();
        done = resp.custom_attrs(1)[1].value == "true";
    }
}
