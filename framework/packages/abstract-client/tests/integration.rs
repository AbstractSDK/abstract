use abstract_adapter::mock::{
    BootMockAdapter, MockExecMsg as BootMockExecMsg, MockInitMsg as BootMockInitMsg,
    MockQueryMsg as BootMockQueryMsg, TEST_METADATA,
};
use abstract_app::{
    abstract_sdk::base::Handler,
    mock::{
        interface::MockAppWithDepI, mock_app_dependency::interface::MockAppI, MockExecMsgFns,
        MockInitMsg, MockQueryMsgFns, MockQueryResponse,
    },
    traits::ModuleIdentification,
};
use abstract_client::{
    builder::cw20_builder::{self, Cw20ExecuteMsgFns, Cw20QueryMsgFns},
    AbstractClient, Account, Application, Publisher,
};
use abstract_core::{
    manager::{
        state::AccountInfo, ManagerModuleInfo, ModuleAddressesResponse, ModuleInfosResponse,
    },
    objects::{
        dependency::Dependency, gov_type::GovernanceDetails, module_version::ModuleDataResponse,
        namespace::Namespace, AccountId, AssetEntry,
    },
};
use abstract_interface::{ClientResolve, VCQueryFns};
use abstract_testing::{
    prelude::{TEST_MODULE_ID, TEST_NAMESPACE, TEST_VERSION, TEST_WITH_DEP_NAMESPACE},
    OWNER,
};
use cosmwasm_std::{coins, Addr, BankMsg, Coin, Empty, Uint128};
use cw_asset::{AssetInfo, AssetInfoUnchecked};
use cw_orch::{
    contract::interface_traits::{CwOrchExecute, CwOrchQuery},
    prelude::{CallAs, Mock},
};
use cw_ownable::Ownership;

#[test]
fn can_create_account_without_optional_parameters() -> anyhow::Result<()> {
    let client = AbstractClient::builder(Mock::new(&Addr::unchecked(OWNER))).build()?;

    let account: Account<Mock> = client.account_builder().build()?;

    let account_info = account.info()?;
    assert_eq!(
        AccountInfo {
            name: String::from("Default Abstract Account"),
            chain_id: String::from("cosmos-testnet-14002"),
            description: None,
            governance_details: GovernanceDetails::Monarchy {
                monarch: Addr::unchecked(OWNER)
            },
            link: None,
        },
        account_info
    );

    let ownership: Ownership<String> = account.ownership()?;
    assert_eq!(
        Ownership {
            owner: Some(OWNER.to_owned()),
            pending_owner: None,
            pending_expiry: None
        },
        ownership
    );

    Ok(())
}

#[test]
fn can_create_account_with_optional_parameters() -> anyhow::Result<()> {
    let asset = "asset";

    let client = AbstractClient::builder(Mock::new(&Addr::unchecked(OWNER)))
        .asset(asset, AssetInfoUnchecked::native(asset))
        .build()?;

    let name = "test-account";
    let description = "description";
    let link = "https://abstract.money";
    let governance_details = GovernanceDetails::Monarchy {
        monarch: String::from("monarch"),
    };
    let namespace = Namespace::new("test-namespace")?;
    let base_asset = AssetEntry::new(asset);
    let account: Account<Mock> = client
        .account_builder()
        .name(name)
        .link(link)
        .description(description)
        .ownership(governance_details.clone())
        .namespace(namespace.clone())
        .base_asset(base_asset)
        .build()?;

    let account_info = account.info()?;
    assert_eq!(
        AccountInfo {
            name: String::from(name),
            chain_id: String::from("cosmos-testnet-14002"),
            description: Some(String::from(description)),
            governance_details,
            link: Some(String::from(link)),
        },
        account_info.into()
    );

    // Namespace is claimed.
    let account_id = client
        .version_control()
        .namespace(namespace)?
        .unwrap()
        .account_id;
    assert_eq!(account_id, AccountId::local(1));

    Ok(())
}

#[test]
fn can_get_account_from_namespace() -> anyhow::Result<()> {
    let client = AbstractClient::builder(Mock::new(&Addr::unchecked(OWNER))).build()?;

    let namespace = Namespace::new("namespace")?;
    let account: Account<Mock> = client
        .account_builder()
        .namespace(namespace.clone())
        .build()?;

    let account_from_namespace: Account<Mock> = client
        .account_builder()
        .fetch_if_namespace_claimed(true)
        .namespace(namespace)
        .build()?;

    assert_eq!(account.info()?, account_from_namespace.info()?);

    Ok(())
}

#[test]
fn can_create_publisher_without_optional_parameters() -> anyhow::Result<()> {
    let client = AbstractClient::builder(Mock::new(&Addr::unchecked(OWNER))).build()?;

    let publisher: Publisher<Mock> = client
        .publisher_builder(Namespace::new(TEST_NAMESPACE)?)
        .build()?;

    let account_info = publisher.account().info()?;
    assert_eq!(
        AccountInfo {
            name: String::from("Default Abstract Account"),
            chain_id: String::from("cosmos-testnet-14002"),
            description: None,
            governance_details: GovernanceDetails::Monarchy {
                monarch: Addr::unchecked(OWNER)
            },
            link: None,
        },
        account_info
    );

    Ok(())
}

#[test]
fn can_create_publisher_with_optional_parameters() -> anyhow::Result<()> {
    let asset = "asset";
    let client = AbstractClient::builder(Mock::new(&Addr::unchecked(OWNER)))
        .asset(asset, AssetInfoUnchecked::native(asset))
        .build()?;

    let name = "test-account";
    let description = "description";
    let link = "https://abstract.money";
    let governance_details = GovernanceDetails::Monarchy {
        monarch: String::from("monarch"),
    };
    let namespace = Namespace::new("test-namespace")?;
    let base_asset = AssetEntry::new(asset);
    let publisher: Publisher<Mock> = client
        .publisher_builder(namespace.clone())
        .name(name)
        .link(link)
        .description(description)
        .ownership(governance_details.clone())
        .base_asset(base_asset)
        .build()?;

    let account_info = publisher.account().info()?;
    assert_eq!(
        AccountInfo {
            name: String::from(name),
            chain_id: String::from("cosmos-testnet-14002"),
            description: Some(String::from(description)),
            governance_details,
            link: Some(String::from(link)),
        },
        account_info.into()
    );

    // Namespace is claimed.
    let account_id = client
        .version_control()
        .namespace(namespace)?
        .unwrap()
        .account_id;
    assert_eq!(account_id, AccountId::local(1));

    Ok(())
}

#[test]
fn can_get_publisher_from_namespace() -> anyhow::Result<()> {
    let client = AbstractClient::builder(Mock::new(&Addr::unchecked(OWNER))).build()?;

    let namespace = Namespace::new("namespace")?;
    let publisher: Publisher<Mock> = client.publisher_builder(namespace.clone()).build()?;

    let publisher_from_namespace: Publisher<Mock> = client.publisher_builder(namespace).build()?;

    assert_eq!(
        publisher.account().info()?,
        publisher_from_namespace.account().info()?
    );

    Ok(())
}

#[test]
fn can_publish_and_install_app() -> anyhow::Result<()> {
    let client = AbstractClient::builder(Mock::new(&Addr::unchecked(OWNER))).build()?;

    let publisher: Publisher<Mock> = client
        .publisher_builder(Namespace::new(TEST_NAMESPACE)?)
        .build()?;

    let publisher_account = publisher.account();
    let publisher_manager = publisher_account.manager()?;
    let publisher_proxy = publisher_account.proxy()?;

    publisher.publish_app::<MockAppI<Mock>>()?;

    // Install app on sub-account
    let my_app: Application<Mock, MockAppI<Mock>> =
        publisher_account.install_app::<MockAppI<Mock>>(&MockInitMsg {}, &[])?;

    my_app.call_as(&publisher_manager).do_something()?;

    let something = my_app.get_something()?;

    assert_eq!(MockQueryResponse {}, something);

    let sub_account_details = my_app.account().info()?;
    assert_eq!(
        AccountInfo {
            name: String::from("Sub Account"),
            chain_id: String::from("cosmos-testnet-14002"),
            description: None,
            governance_details: GovernanceDetails::SubAccount {
                manager: publisher_manager.clone(),
                proxy: publisher_proxy
            },
            link: None,
        },
        sub_account_details
    );

    // Install app on current account
    let publisher = client
        .publisher_builder(Namespace::new("tester")?)
        .install_on_sub_account(false)
        .build()?;
    let my_adapter: Application<Mock, MockAppI<Mock>> =
        publisher.account().install_app(&MockInitMsg {}, &[])?;

    my_adapter.call_as(&publisher_manager).do_something()?;
    let mock_query: MockQueryResponse = my_adapter.get_something()?;

    assert_eq!(MockQueryResponse {}, mock_query);

    let sub_account_details = my_adapter.account().info()?;
    assert_eq!(
        AccountInfo {
            name: String::from("Default Abstract Account"),
            chain_id: String::from("cosmos-testnet-14002"),
            description: None,
            governance_details: GovernanceDetails::Monarchy {
                monarch: client.sender()
            },
            link: None,
        },
        sub_account_details
    );

    Ok(())
}

#[test]
fn can_publish_and_install_adapter() -> anyhow::Result<()> {
    let client = AbstractClient::builder(Mock::new(&Addr::unchecked(OWNER))).build()?;

    let publisher: Publisher<Mock> = client
        .publisher_builder(Namespace::new("tester")?)
        .build()?;

    let publisher_manager = publisher.account().manager()?;
    let publisher_proxy = publisher.account().proxy()?;

    publisher.publish_adapter::<BootMockInitMsg, BootMockAdapter<Mock>>(BootMockInitMsg {})?;

    // Install adapter on sub-account
    let my_adapter: Application<Mock, BootMockAdapter<Mock>> =
        publisher.account().install_adapter(&[])?;

    my_adapter
        .call_as(&publisher_manager)
        .execute(&BootMockExecMsg {}.into(), None)?;
    let mock_query: String = my_adapter.query(&BootMockQueryMsg {}.into())?;

    assert_eq!(String::from("mock_query"), mock_query);

    let sub_account_details = my_adapter.account().info()?;
    assert_eq!(
        AccountInfo {
            name: String::from("Sub Account"),
            chain_id: String::from("cosmos-testnet-14002"),
            description: None,
            governance_details: GovernanceDetails::SubAccount {
                manager: publisher_manager.clone(),
                proxy: publisher_proxy
            },
            link: None,
        },
        sub_account_details
    );

    // Install adapter on current account
    let publisher = client
        .publisher_builder(Namespace::new("tester")?)
        .install_on_sub_account(false)
        .build()?;
    let my_adapter: Application<Mock, BootMockAdapter<Mock>> =
        publisher.account().install_adapter(&[])?;

    my_adapter
        .call_as(&publisher_manager)
        .execute(&BootMockExecMsg {}.into(), None)?;
    let mock_query: String = my_adapter.query(&BootMockQueryMsg {}.into())?;

    assert_eq!(String::from("mock_query"), mock_query);

    let sub_account_details = my_adapter.account().info()?;
    assert_eq!(
        AccountInfo {
            name: String::from("Default Abstract Account"),
            chain_id: String::from("cosmos-testnet-14002"),
            description: None,
            governance_details: GovernanceDetails::Monarchy {
                monarch: client.sender()
            },
            link: None,
        },
        sub_account_details
    );
    Ok(())
}

#[test]
fn cannot_create_same_account_twice_when_fetch_flag_is_disabled() -> anyhow::Result<()> {
    let client = AbstractClient::builder(Mock::new(&Addr::unchecked(OWNER))).build()?;

    let namespace = Namespace::new("namespace")?;

    // First call succeeds.
    client
        .account_builder()
        .namespace(namespace.clone())
        .build()?;

    // Second call fails
    let result = client
        .account_builder()
        .fetch_if_namespace_claimed(false)
        .namespace(namespace)
        .build();
    assert!(result.is_err());

    Ok(())
}

#[test]
fn can_create_same_account_twice_when_fetch_flag_is_enabled() -> anyhow::Result<()> {
    let client = AbstractClient::builder(Mock::new(&Addr::unchecked(OWNER))).build()?;

    let namespace = Namespace::new("namespace")?;

    let account1 = client
        .account_builder()
        .namespace(namespace.clone())
        .build()?;

    let account2 = client
        .account_builder()
        .namespace(namespace)
        .fetch_if_namespace_claimed(true)
        .build()?;

    assert_eq!(account1.info()?, account2.info()?);

    Ok(())
}

#[test]
fn can_install_module_with_dependencies() -> anyhow::Result<()> {
    let client = AbstractClient::builder(Mock::new(&Addr::unchecked(OWNER))).build()?;

    let app_publisher: Publisher<Mock> = client
        .publisher_builder(Namespace::new(TEST_WITH_DEP_NAMESPACE)?)
        .build()?;

    let app_dependency_publisher: Publisher<Mock> = client
        .publisher_builder(Namespace::new(TEST_NAMESPACE)?)
        .build()?;

    app_dependency_publisher.publish_app::<MockAppI<Mock>>()?;
    app_publisher.publish_app::<MockAppWithDepI<Mock>>()?;

    let my_app: Application<Mock, MockAppWithDepI<Mock>> = app_publisher
        .account()
        .install_app_with_dependencies::<MockAppWithDepI<Mock>>(
        &MockInitMsg {},
        Empty {},
        &[],
    )?;

    my_app
        .call_as(&app_publisher.account().manager()?)
        .do_something()?;

    let something = my_app.get_something()?;

    assert_eq!(MockQueryResponse {}, something);

    let module_infos_response: ModuleInfosResponse = my_app.account().module_infos()?;
    let module_addresses_response: ModuleAddressesResponse = my_app
        .account()
        .module_addresses(vec![TEST_MODULE_ID.to_owned(), TEST_MODULE_ID.to_owned()])?;

    let app_address: Addr = module_addresses_response
        .modules
        .iter()
        .find(|(module_id, _)| module_id == TEST_MODULE_ID)
        .unwrap()
        .clone()
        .1;

    let app_dependency_address: Addr = module_addresses_response
        .modules
        .iter()
        .find(|(module_id, _)| module_id == TEST_MODULE_ID)
        .unwrap()
        .clone()
        .1;

    assert!(module_infos_response
        .module_infos
        .contains(&ManagerModuleInfo {
            id: TEST_MODULE_ID.to_owned(),
            version: cw2::ContractVersion {
                contract: TEST_MODULE_ID.to_owned(),
                version: TEST_VERSION.to_owned()
            },
            address: app_dependency_address,
        }));

    assert!(module_infos_response
        .module_infos
        .contains(&ManagerModuleInfo {
            id: TEST_MODULE_ID.to_owned(),
            version: cw2::ContractVersion {
                contract: TEST_MODULE_ID.to_owned(),
                version: TEST_VERSION.to_owned()
            },
            address: app_address,
        }));

    Ok(())
}

#[test]
fn can_build_cw20_with_all_options() -> anyhow::Result<()> {
    let client = AbstractClient::builder(Mock::new(&Addr::unchecked(OWNER))).build()?;

    let name = "name";
    let symbol = "symbol";
    let decimals = 6;
    let description = "A test cw20 token";
    let logo = "link-to-logo";
    let project = "project";
    let marketing = "marketing";
    let cap = Uint128::from(100u128);
    let starting_balance = Uint128::from(100u128);
    let minter_response = cw20_builder::MinterResponse {
        minter: OWNER.to_owned(),
        cap: Some(cap),
    };

    let cw20: cw20_builder::Cw20Base<Mock> = client
        .cw20_builder(name, symbol, decimals)
        .initial_balance(cw20_builder::Cw20Coin {
            address: OWNER.to_owned(),
            amount: starting_balance,
        })
        .admin(OWNER)
        .mint(minter_response.clone())
        .marketing(cw20_builder::InstantiateMarketingInfo {
            description: Some(description.to_owned()),
            logo: Some(cw20_builder::Logo::Url(logo.to_owned())),
            project: Some(project.to_owned()),
            marketing: Some(marketing.to_owned()),
        })
        .instantiate_with_id("abstract:test_cw20")?;

    let actual_minter_response: cw20_builder::MinterResponse = cw20.minter()?;
    assert_eq!(minter_response, actual_minter_response);

    let marketing_info_response: cw20_builder::MarketingInfoResponse = cw20.marketing_info()?;
    assert_eq!(
        cw20_builder::MarketingInfoResponse {
            description: Some(description.to_owned()),
            logo: Some(cw20_builder::LogoInfo::Url(logo.to_owned())),
            project: Some(project.to_owned()),
            marketing: Some(Addr::unchecked(marketing)),
        },
        marketing_info_response
    );

    let owner_balance: cw20_builder::BalanceResponse = cw20.balance(OWNER.to_owned())?;
    assert_eq!(
        cw20_builder::BalanceResponse {
            balance: starting_balance
        },
        owner_balance
    );
    let transfer_amount = Uint128::from(50u128);
    let recipient = "user";
    cw20.transfer(transfer_amount, recipient.to_owned())?;

    let recipient_balance = cw20.balance(recipient.to_owned())?;
    assert_eq!(
        cw20_builder::BalanceResponse {
            balance: transfer_amount
        },
        recipient_balance
    );

    Ok(())
}

#[test]
fn can_build_cw20_with_minimum_options() -> anyhow::Result<()> {
    let client = AbstractClient::builder(Mock::new(&Addr::unchecked(OWNER))).build()?;

    let name = "name";
    let symbol = "symbol";
    let decimals = 6;

    let cw20: cw20_builder::Cw20Base<Mock> = client
        .cw20_builder(name, symbol, decimals)
        .instantiate_with_id("abstract:test_cw20")?;

    let minter_response = cw20.minter();
    assert!(minter_response.is_err());

    let marketing_info_response: cw20_builder::MarketingInfoResponse = cw20.marketing_info()?;
    assert_eq!(
        cw20_builder::MarketingInfoResponse {
            description: None,
            logo: None,
            project: None,
            marketing: None,
        },
        marketing_info_response
    );

    let owner_balance: cw20_builder::BalanceResponse = cw20.balance(OWNER.to_owned())?;
    assert_eq!(
        cw20_builder::BalanceResponse {
            balance: Uint128::zero(),
        },
        owner_balance
    );

    Ok(())
}

#[test]
fn can_modify_and_query_balance_on_account() -> anyhow::Result<()> {
    let client = AbstractClient::builder(Mock::new(&Addr::unchecked(OWNER))).build()?;
    let account = client.account_builder().build()?;

    let coin1 = Coin::new(50, "denom1");
    let coin2 = Coin::new(20, "denom2");
    let coin3 = Coin::new(10, "denom3");
    account.set_balance(&[coin1.clone(), coin2.clone()])?;
    account.add_balance(&[coin3.clone()])?;

    assert_eq!(coin1.amount, account.query_balance("denom1")?);
    assert_eq!(coin2.amount, account.query_balance("denom2")?);
    assert_eq!(coin3.amount, account.query_balance("denom3")?);
    assert_eq!(Uint128::zero(), account.query_balance("denom4")?);

    assert_eq!(vec![coin1, coin2, coin3], account.query_balances()?);
    Ok(())
}
#[test]
fn can_get_module_dependency() -> anyhow::Result<()> {
    let client = AbstractClient::builder(Mock::new(&Addr::unchecked(OWNER))).build()?;

    let app_publisher: Publisher<Mock> = client
        .publisher_builder(Namespace::new(TEST_WITH_DEP_NAMESPACE)?)
        .build()?;

    let app_dependency_publisher: Publisher<Mock> = client
        .publisher_builder(Namespace::new(TEST_NAMESPACE)?)
        .build()?;

    app_dependency_publisher.publish_app::<MockAppI<Mock>>()?;
    app_publisher.publish_app::<MockAppWithDepI<Mock>>()?;

    let my_app: Application<Mock, MockAppWithDepI<Mock>> = app_publisher
        .account()
        .install_app_with_dependencies(&MockInitMsg {}, Empty {}, &[])?;

    let dependency: MockAppI<Mock> = my_app.module()?;
    dependency.do_something()?;
    Ok(())
}

#[test]
fn can_set_and_query_balance_with_client() -> anyhow::Result<()> {
    let client = AbstractClient::builder(Mock::new(&Addr::unchecked(OWNER))).build()?;

    let user = Addr::unchecked("user");
    let coin1 = Coin::new(50, "denom1");
    let coin2 = Coin::new(20, "denom2");
    let coin3 = Coin::new(10, "denom3");
    client.set_balance(&user, &[coin1.clone(), coin2.clone()])?;
    client.add_balance(&user, &[coin3.clone()])?;

    assert_eq!(coin1.amount, client.query_balance(&user, "denom1")?);
    assert_eq!(coin2.amount, client.query_balance(&user, "denom2")?);
    assert_eq!(coin3.amount, client.query_balance(&user, "denom3")?);
    assert_eq!(Uint128::zero(), client.query_balance(&user, "denom4")?);

    assert_eq!(vec![coin1, coin2, coin3], client.query_balances(&user)?);
    Ok(())
}

#[test]
fn cannot_get_nonexisting_module_dependency() -> anyhow::Result<()> {
    let client = AbstractClient::builder(Mock::new(&Addr::unchecked(OWNER))).build()?;

    let publisher: Publisher<Mock> = client
        .publisher_builder(Namespace::new(TEST_NAMESPACE)?)
        .build()?;

    publisher.publish_app::<MockAppI<Mock>>()?;

    let my_app: Application<Mock, MockAppI<Mock>> = publisher
        .account()
        .install_app::<MockAppI<Mock>>(&MockInitMsg {}, &[])?;

    let dependency_res = my_app.module::<MockAppWithDepI<Mock>>();
    assert!(dependency_res.is_err());
    Ok(())
}

#[test]
fn can_execute_on_proxy() -> anyhow::Result<()> {
    let denom = "denom";
    let client = AbstractClient::builder(Mock::new(&Addr::unchecked(OWNER))).build()?;
    client.set_balances([(client.sender(), coins(100, denom).as_slice())])?;

    let user = String::from("user");

    let account: Account<Mock> = client.account_builder().build()?;

    let amount = 20;
    account.execute(
        vec![BankMsg::Send {
            to_address: user.clone(),
            amount: coins(20, denom),
        }],
        &coins(amount, denom),
    )?;

    assert_eq!(
        amount,
        client.query_balance(&Addr::unchecked(user), denom)?.into()
    );
    Ok(())
}

#[test]
fn resolve_works() -> anyhow::Result<()> {
    let denom = "test_denom";
    let entry = "denom";
    let client = AbstractClient::builder(Mock::new(&Addr::unchecked(OWNER)))
        .asset(entry, cw_asset::AssetInfoBase::Native(denom.to_owned()))
        .build()?;

    let name_service = client.name_service();
    let asset_entry = AssetEntry::new(entry);
    let asset = asset_entry.resolve(name_service)?;
    assert_eq!(asset, AssetInfo::Native(denom.to_owned()));

    // Or use it on AnsHost object
    let asset = name_service.resolve(&asset_entry)?;
    assert_eq!(asset, AssetInfo::Native(denom.to_owned()));
    Ok(())
}

#[test]
fn doc_example_test() -> anyhow::Result<()> {
    // ## ANCHOR: build_client
    // Create environment
    let sender: Addr = Addr::unchecked("sender");
    let env: Mock = Mock::new(&sender);

    // Build the client
    let client: AbstractClient<Mock> = AbstractClient::builder(env).build()?;
    // ## ANCHOR_END: build_client

    // ## ANCHOR: balances
    let coins = &[Coin::new(50, "eth"), Coin::new(20, "btc")];

    // Set a balance
    client.set_balance(&sender, coins)?;

    // Add to an address's balance
    client.add_balance(&sender, &[Coin::new(50, "eth")])?;

    // Query an address's balance
    let coin1_balance = client.query_balance(&sender, "eth")?;

    assert_eq!(coin1_balance.u128(), 100);
    // ## ANCHOR_END: balances

    // ## ANCHOR: publisher
    // Create a publisher
    let publisher: Publisher<Mock> = client
        .publisher_builder(Namespace::from_id(TEST_MODULE_ID)?)
        .build()?;

    // Publish an app
    publisher.publish_app::<MockAppI<Mock>>()?;
    // ## ANCHOR_END: publisher

    // ## ANCHOR: account
    let account: Account<Mock> = client.account_builder().build()?;

    // ## ANCHOR: app_interface
    // Install an app
    let app: Application<Mock, MockAppI<Mock>> =
        account.install_app::<MockAppI<Mock>>(&MockInitMsg {}, &[])?;
    // ## ANCHOR_END: account
    // Call a function on the app
    app.do_something()?;

    // Call as someone else
    let manager: Addr = account.manager()?;
    app.call_as(&manager).do_something()?;

    // Query the app
    let something: MockQueryResponse = app.get_something()?;
    // ## ANCHOR_END: app_interface

    // ## ANCHOR: account_helpers
    // Get account info
    let account_info: AccountInfo = account.info()?;
    // Get the owner
    let owner: Addr = account.owner()?;
    // Add or set balance
    account.add_balance(&[Coin::new(100, "btc")])?;
    // ...
    // ## ANCHOR_END: account_helpers

    assert_eq!(
        AccountInfo {
            name: String::from("Default Abstract Account"),
            chain_id: String::from("cosmos-testnet-14002"),
            description: None,
            governance_details: GovernanceDetails::Monarchy {
                monarch: sender.clone()
            },
            link: None,
        },
        account_info
    );

    assert_eq!(owner, sender);
    assert_eq!(something, MockQueryResponse {});

    Ok(())
}

#[test]
fn can_get_abstract_account_from_client_account() -> anyhow::Result<()> {
    let sender: Addr = Addr::unchecked("sender");
    let env: Mock = Mock::new(&sender);

    // Build the client
    let client: AbstractClient<Mock> = AbstractClient::builder(env).build()?;

    let account = client.account_builder().build()?;
    let abstract_account: &abstract_interface::AbstractAccount<Mock> = account.as_ref();
    assert_eq!(abstract_account.id()?, AccountId::local(1));
    Ok(())
}

#[test]
fn can_use_adapter_object_after_publishing() -> anyhow::Result<()> {
    let client: AbstractClient<Mock> =
        AbstractClient::builder(Mock::new(&Addr::unchecked(OWNER))).build()?;
    let publisher = client
        .publisher_builder(Namespace::new(TEST_NAMESPACE)?)
        .build()?;

    let adapter =
        publisher.publish_adapter::<BootMockInitMsg, BootMockAdapter<Mock>>(BootMockInitMsg {})?;
    let module_data: ModuleDataResponse =
        adapter.query(&abstract_core::adapter::QueryMsg::Base(
            abstract_core::adapter::BaseQueryMsg::ModuleData {},
        ))?;

    assert_eq!(
        module_data,
        ModuleDataResponse {
            module_id: abstract_adapter::mock::MOCK_ADAPTER.module_id().to_owned(),
            version: abstract_adapter::mock::MOCK_ADAPTER.version().to_owned(),
            dependencies: abstract_adapter::mock::MOCK_ADAPTER
                .dependencies()
                .into_iter()
                .map(Dependency::from)
                .map(Into::into)
                .collect(),
            metadata: Some(TEST_METADATA.to_owned())
        }
    );
    Ok(())
}
