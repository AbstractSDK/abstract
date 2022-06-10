use std::collections::HashMap;

use cosmwasm_std::Addr;

use abstract_os::common_module::add_on_msg::AddOnInstantiateMsg;
use abstract_os::core::modules::ModuleInfo;
use abstract_os::modules::add_ons::subscription::msg::InstantiateMsg as SubInitMsg;
use abstract_os::native::version_control::state::Core;
use cosmwasm_std::to_binary;
use cosmwasm_std::Coin;
use cosmwasm_std::Decimal;
use cosmwasm_std::Uint128;
use cosmwasm_std::Uint64;
use cw_asset::AssetInfoUnchecked;

use abstract_os::registery::SUBSCRIPTION;
use cw_multi_test::App;

use crate::tests::common::TEST_CREATOR;
use crate::tests::subscription::register_subscription;
use crate::tests::testing_infrastructure::common_integration::mock_app;

use abstract_os::core::*;
use anyhow::Result as AnyResult;

use abstract_os::native::*;
use cw_multi_test::Executor;

use super::common_integration::NativeContracts;

use super::upload::upload_base_contracts;
use super::verify::os_store_as_expected;

pub fn init_os(
    app: &mut App,
    sender: &Addr,
    native_contracts: &NativeContracts,
    os_store: &mut HashMap<u32, Core>,
) -> AnyResult<()> {
    let funds = if os_store.is_empty() {
        vec![]
    } else {
        vec![Coin::new(100, "uusd")]
    };

    let _resp = app.execute_contract(
        sender.clone(),
        native_contracts.os_factory.clone(),
        &abstract_os::native::os_factory::msg::ExecuteMsg::CreateOs {
            governance: abstract_os::governance::gov_type::GovernanceDetails::Monarchy {
                monarch: sender.to_string(),
            },
        },
        &funds,
    )?;

    let resp: os_factory::msg::ConfigResponse = app.wrap().query_wasm_smart(
        &native_contracts.os_factory,
        &os_factory::msg::QueryMsg::Config {},
    )?;
    let os_id = resp.next_os_id - 1;

    // Check OS
    let core: Core = app.wrap().query_wasm_smart(
        &native_contracts.version_control,
        &version_control::msg::QueryMsg::QueryOsAddress { os_id },
    )?;

    os_store.insert(os_id, core.clone());
    assert!(os_store_as_expected(&app, &native_contracts, &os_store));
    Ok(())
}

/// Instantiate the first OS which has the subscriber module.
/// Update the factory using this new address
pub fn init_primary_os(
    app: &mut App,
    sender: &Addr,
    native_contracts: &NativeContracts,
    os_store: &mut HashMap<u32, Core>,
) -> AnyResult<()> {
    register_subscription(app, sender, &native_contracts.version_control)?;

    let core = os_store.get(&0u32).unwrap();

    let init_msg = to_binary(&SubInitMsg {
        base: AddOnInstantiateMsg {
            memory_address: native_contracts.memory.to_string(),
        },
        contribution:
            abstract_os::modules::add_ons::subscription::msg::ContributionInstantiateMsg {
                protocol_income_share: Decimal::percent(10),
                emission_user_share: Decimal::percent(25),
                max_emissions_multiple: Decimal::from_ratio(2u128, 1u128),
                project_token: native_contracts.token.to_string(),
                emissions_amp_factor: Uint128::new(680000000),
                emissions_offset: Uint128::new(52000),
                base_denom: "uusd".to_string(),
            },
        subscription:
            abstract_os::modules::add_ons::subscription::msg::SubscriptionInstantiateMsg {
                factory_addr: native_contracts.os_factory.to_string(),
                payment_asset: AssetInfoUnchecked::native("uusd"),
                subscription_cost: Uint64::new(100),
                version_control_addr: native_contracts.version_control.to_string(),
            },
    })?;

    let msg = abstract_os::core::manager::msg::ExecuteMsg::CreateModule {
        module: modules::Module {
            info: ModuleInfo {
                name: SUBSCRIPTION.to_string(),
                version: None,
            },
            kind: modules::ModuleKind::AddOn,
        },
        init_msg: Some(init_msg),
    };

    let resp = app
        .execute_contract(sender.clone(), core.manager.clone(), &msg, &[])
        .unwrap();

    let msg = abstract_os::native::os_factory::msg::ExecuteMsg::UpdateConfig {
        admin: None,
        memory_contract: None,
        version_control_contract: None,
        module_factory_address: None,
        subscription_address: Some(resp.events[5].attributes[1].value.clone()),
    };

    app.execute_contract(
        sender.clone(),
        native_contracts.os_factory.clone(),
        &msg,
        &[],
    )
    .unwrap();

    Ok(())
}

#[test]
fn proper_initialization() {
    let mut app = mock_app();
    let sender = Addr::unchecked(TEST_CREATOR);
    let (_code_ids, native_contracts) = upload_base_contracts(&mut app);
    let mut os_store: HashMap<u32, Core> = HashMap::new();

    init_os(&mut app, &sender, &native_contracts, &mut os_store).expect("created first os");

    init_os(&mut app, &sender, &native_contracts, &mut os_store)
        .expect_err("first OS needs to have subscriptions");

    init_primary_os(&mut app, &sender, &native_contracts, &mut os_store).unwrap();
}
