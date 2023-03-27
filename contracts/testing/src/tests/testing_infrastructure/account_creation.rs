use super::common_integration::NativeContracts;
use super::{upload::upload_base_contracts, verify::os_store_as_expected};
use crate::tests::{
    common::{OS_NAME, SUBSCRIPTION_COST, TEST_CREATOR},
    subscription::register_subscription,
    testing_infrastructure::common_integration::mock_app,
};
use abstract_sdk::core::SUBSCRIPTION;
use abstract_sdk::core::*;
use abstract_sdk::core::{
    app::BaseInstantiateMsg, objects::module::ModuleInfo,
    subscription::InstantiateMsg as SubInitMsg, version_control::Core,
};
use abstract_sdk::core::{objects::module::ModuleVersion, version_control::AccountBaseResponse};
use anyhow::Result as AnyResult;
use cosmwasm_std::Addr;
use cosmwasm_std::{to_binary, Coin, Decimal, Uint128, Uint64};
use cw_asset::AssetInfoUnchecked;
use cw_multi_test::App;
use cw_multi_test::Executor;
use std::{collections::HashMap, str::FromStr};

pub fn init_os(
    app: &mut App,
    sender: &Addr,
    native_contracts: &NativeContracts,
    os_store: &mut HashMap<u32, Core>,
) -> AnyResult<()> {
    let funds = if os_store.is_empty() {
        vec![]
    } else {
        vec![Coin::new(43200, "uusd")]
    };

    let _resp = app.execute_contract(
        sender.clone(),
        native_contracts.account_factory.clone(),
        &abstract_sdk::core::account_factory::ExecuteMsg::CreateOs {
            governance: abstract_sdk::core::objects::gov_type::GovernanceDetails::Monarchy {
                monarch: sender.to_string(),
            },
            name: OS_NAME.to_string(),
            description: None,
            link: None,
        },
        &funds,
    )?;

    let resp: account_factory::ConfigResponse = app.wrap().query_wasm_smart(
        &native_contracts.account_factory,
        &account_factory::QueryMsg::Config {},
    )?;
    let account_id = resp.next_acct_id - 1;

    // Check OS
    let core: AccountBaseResponse = app.wrap().query_wasm_smart(
        &native_contracts.version_control,
        &version_control::QueryMsg::OsCore { account_id },
    )?;

    os_store.insert(account_id, core.account);
    assert!(os_store_as_expected(app, native_contracts, os_store));
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

    let init_msg = to_binary(&app::InstantiateMsg {
        app: SubInitMsg {
            contribution: Some(
                abstract_sdk::core::subscription::ContributionInstantiateMsg {
                    protocol_income_share: Decimal::percent(10),
                    emission_user_share: Decimal::percent(25),
                    max_emissions_multiple: Decimal::from_ratio(2u128, 1u128),
                    token_info: cw_asset::AssetInfoBase::Cw20(native_contracts.token.to_string()),
                    emissions_amp_factor: Uint128::new(680000000),
                    emissions_offset: Uint128::new(52000),
                    income_averaging_period: Uint64::new(100),
                },
            ),
            subscription: abstract_sdk::core::subscription::SubscriptionInstantiateMsg {
                factory_addr: native_contracts.account_factory.to_string(),
                payment_asset: AssetInfoUnchecked::native("uusd"),
                subscription_cost_per_block: Decimal::from_str(SUBSCRIPTION_COST).unwrap(),
                version_control_addr: native_contracts.version_control.to_string(),
                subscription_per_block_emissions:
                    subscription::state::UncheckedEmissionType::IncomeBased(
                        cw_asset::AssetInfoBase::Cw20(native_contracts.token.to_string()),
                    ),
            },
        },
        base: BaseInstantiateMsg {
            ans_host_address: native_contracts.ans_host.to_string(),
        },
    })?;

    let msg = abstract_core::manager::ExecuteMsg::InstallModule {
        module: ModuleInfo::from_id(SUBSCRIPTION, ModuleVersion::Latest)?,
        init_msg: Some(init_msg),
    };

    let resp = app
        .execute_contract(sender.clone(), core.manager.clone(), &msg, &[])
        .unwrap();

    let msg = abstract_sdk::core::account_factory::ExecuteMsg::UpdateConfig {
        admin: None,
        ans_host_contract: None,
        version_control_contract: None,
        module_factory_address: None,
        subscription_address: Some(resp.events[4].attributes[1].value.clone()),
    };

    app.execute_contract(
        sender.clone(),
        native_contracts.account_factory.clone(),
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

    init_os(&mut app, &sender, &native_contracts, &mut os_store).expect("created first account");

    // TODO: review on release
    // init_os(&mut app, &sender, &native_contracts, &mut os_store)
    //     .expect_err("first OS needs to have subscriptions");

    init_primary_os(&mut app, &sender, &native_contracts, &mut os_store).unwrap();
}
