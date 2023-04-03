use super::common_integration::NativeContracts;
use super::{upload::upload_base_contracts, verify::account_store_as_expected};
use crate::tests::{
    common::{ACCOUNT_NAME, SUBSCRIPTION_COST, TEST_CREATOR},
    subscription::register_subscription,
    testing_infrastructure::common_integration::mock_app,
};
use abstract_sdk::core::SUBSCRIPTION;
use abstract_sdk::core::*;
use abstract_sdk::core::{
    app::BaseInstantiateMsg, objects::module::ModuleInfo,
    subscription::InstantiateMsg as SubInitMsg, version_control::AccountBase,
};
use abstract_sdk::core::{objects::module::ModuleVersion, version_control::AccountBaseResponse};
use anyhow::Result as AnyResult;
use cosmwasm_std::Addr;
use cosmwasm_std::{to_binary, Coin, Decimal, Uint128, Uint64};
use cw_asset::AssetInfoUnchecked;
use cw_multi_test::App;
use cw_multi_test::Executor;
use std::{collections::HashMap, str::FromStr};

pub fn init_account(
    app: &mut App,
    sender: &Addr,
    native_contracts: &NativeContracts,
    account_store: &mut HashMap<u32, AccountBase>,
) -> AnyResult<()> {
    let funds = if account_store.is_empty() {
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
            name: ACCOUNT_NAME.to_string(),
            description: None,
            link: None,
        },
        &funds,
    )?;

    let resp: account_factory::ConfigResponse = app.wrap().query_wasm_smart(
        &native_contracts.account_factory,
        &account_factory::QueryMsg::Config {},
    )?;
    let account_id = resp.next_account_id - 1;

    // Check Account
    let core: AccountBaseResponse = app.wrap().query_wasm_smart(
        &native_contracts.version_control,
        &version_control::QueryMsg::OsAccountBase { account_id },
    )?;

    account_store.insert(account_id, account_base.account);
    assert!(account_store_as_expected(
        app,
        native_contracts,
        account_store,
    ));
    Ok(())
}

/// Instantiate the first Account which has the subscriber module.
/// Update the factory using this new address
pub fn init_primary_os(
    app: &mut App,
    sender: &Addr,
    native_contracts: &NativeContracts,
    account_store: &mut HashMap<u32, AccountBase>,
) -> AnyResult<()> {
    register_subscription(app, sender, &native_contracts.version_control)?;

    let account_base = account_store.get(&0u32).unwrap();

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
        .execute_contract(sender.clone(), account_base.manager.clone(), &msg, &[])
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
    let mut account_store: HashMap<u32, AccountBase> = HashMap::new();

    init_account(&mut app, &sender, &native_contracts, &mut account_store)
        .expect("created first account");

    // TODO: review on release
    // init_os(&mut app, &sender, &native_contracts, &mut account_store)
    //     .expect_err("first Account needs to have subscriptions");

    init_primary_os(&mut app, &sender, &native_contracts, &mut account_store).unwrap();
}
