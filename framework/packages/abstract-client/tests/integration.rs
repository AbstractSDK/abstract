use abstract_account::error::AccountError;
use abstract_adapter::mock::{
    interface::MockAdapterI, MockExecMsg as AdapterMockExecMsg, MockInitMsg as AdapterMockInitMsg,
    MockQueryMsg as AdapterMockQueryMsg, TEST_METADATA,
};
use abstract_app::{
    mock::{
        interface::MockAppWithDepI, mock_app_dependency::interface::MockAppI, MockExecMsgFns,
        MockInitMsg, MockQueryMsgFns, MockQueryResponse,
    },
    objects::module::ModuleInfo,
    sdk::base::Handler,
    traits::ModuleIdentification,
};
use abstract_client::{
    builder::cw20_builder::{self, ExecuteMsgInterfaceFns, QueryMsgInterfaceFns},
    AbstractClient, AbstractClientError, Account, AccountSource, Application, Environment,
    Publisher,
};
use abstract_interface::{
    ClientResolve, IbcClient, InstallConfig, RegisteredModule, RegistryExecFns, RegistryQueryFns,
};
use abstract_std::{
    account::{
        state::AccountInfo, AccountModuleInfo, ModuleAddressesResponse, ModuleInfosResponse,
    },
    adapter::AuthorizedAddressesResponse,
    ans_host::QueryMsgFns,
    objects::{
        dependency::Dependency, fee::FixedFee, gov_type::GovernanceDetails,
        module_version::ModuleDataResponse, namespace::Namespace, AccountId, AssetEntry,
    },
    IBC_CLIENT,
};
use abstract_testing::prelude::*;
use cosmwasm_std::{coins, BankMsg, Uint128};
use cw_asset::{AssetInfo, AssetInfoUnchecked};
use cw_orch::prelude::*;
use mock_service::{MockMsg, MockService};
use registry::error::RegistryError;

mod mock_service;

#[test]
fn can_create_account_without_optional_parameters() -> anyhow::Result<()> {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let client = AbstractClient::builder(chain.clone()).build_mock()?;

    let account: Account<MockBech32> = client.account_builder().build()?;

    let account_info = account.info()?;
    assert_eq!(
        AccountInfo {
            name: Some(String::from("Default Abstract Account")),
            description: None,
            link: None,
        },
        account_info
    );

    let owner = account.owner()?;
    assert_eq!(owner, sender);

    Ok(())
}

#[test]
fn can_create_account_with_optional_parameters() -> anyhow::Result<()> {
    let asset = "asset";

    let chain = MockBech32::new("mock");
    let client = AbstractClient::builder(chain.clone())
        .asset(asset, AssetInfoUnchecked::native(asset))
        .build_mock()?;

    let name = "test-account";
    let description = "description";
    let link = "https://abstract.money";
    let governance_details = GovernanceDetails::Monarchy {
        monarch: chain.addr_make("monarch").to_string(),
    };
    let namespace = Namespace::new("test-namespace")?;
    let account: Account<MockBech32> = client
        .account_builder()
        .name(name)
        .link(link)
        .description(description)
        .ownership(governance_details.clone())
        .namespace(namespace.clone())
        .build()?;

    let account_info = account.info()?;
    assert_eq!(
        AccountInfo {
            name: Some(String::from(name)),
            description: Some(String::from(description)),
            link: Some(String::from(link)),
        },
        account_info
    );

    // Namespace is claimed.
    let account_id = client.registry().namespace(namespace)?.unwrap().account_id;
    assert_eq!(account_id, AccountId::local(1));

    Ok(())
}

#[test]
fn can_get_account_from_namespace() -> anyhow::Result<()> {
    let chain = MockBech32::new("mock");
    let client = AbstractClient::builder(chain.clone()).build_mock()?;

    let namespace = Namespace::new("namespace")?;
    let account: Account<MockBech32> = client
        .account_builder()
        .namespace(namespace.clone())
        .build()?;

    // From namespace directly
    let account_from_namespace: Account<MockBech32> = client.account_from(namespace)?;

    assert_eq!(account.info()?, account_from_namespace.info()?);
    Ok(())
}

#[test]
fn err_fetching_unclaimed_namespace() -> anyhow::Result<()> {
    let chain = MockBech32::new("mock");
    let client = AbstractClient::builder(chain.clone()).build_mock()?;

    let namespace = Namespace::new("namespace")?;

    let account_from_namespace_no_claim_res: Result<
        Account<MockBech32>,
        abstract_client::AbstractClientError,
    > = client.account_from(namespace);

    assert!(matches!(
        account_from_namespace_no_claim_res.unwrap_err(),
        abstract_client::AbstractClientError::NamespaceNotClaimed { .. }
    ));

    Ok(())
}

#[test]
fn can_create_publisher_without_optional_parameters() -> anyhow::Result<()> {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let client = AbstractClient::builder(chain.clone()).build_mock()?;

    let publisher: Publisher<MockBech32> = client
        .account_builder()
        .namespace(Namespace::new(TEST_NAMESPACE)?)
        .build()?
        .publisher()?;

    let account_info = publisher.account().info()?;
    assert_eq!(
        AccountInfo {
            name: Some(String::from("Default Abstract Account")),
            description: None,
            link: None,
        },
        account_info
    );
    let owner = publisher.account().owner()?;
    assert_eq!(owner, sender);

    Ok(())
}

#[test]
fn can_create_publisher_with_optional_parameters() -> anyhow::Result<()> {
    let asset = "asset";
    let chain = MockBech32::new("mock");
    let client = AbstractClient::builder(chain.clone())
        .asset(asset, AssetInfoUnchecked::native(asset))
        .build_mock()?;

    let name = "test-account";
    let description = "description";
    let link = "https://abstract.money";
    let governance_details = GovernanceDetails::Monarchy {
        monarch: chain.addr_make("monarch").to_string(),
    };
    let namespace = Namespace::new("test-namespace")?;
    let publisher: Publisher<MockBech32> = client
        .account_builder()
        .namespace(namespace.clone())
        .name(name)
        .link(link)
        .description(description)
        .ownership(governance_details.clone())
        .build()?
        .publisher()?;

    let account_info = publisher.account().info()?;
    assert_eq!(
        AccountInfo {
            name: Some(String::from(name)),
            description: Some(String::from(description)),
            link: Some(String::from(link)),
        },
        account_info
    );

    let ownership = publisher.account().ownership()?;
    assert_eq!(ownership.owner, governance_details);

    // Namespace is claimed.
    let account_id = client.registry().namespace(namespace)?.unwrap().account_id;
    assert_eq!(account_id, AccountId::local(1));

    Ok(())
}

#[test]
fn can_get_publisher_from_namespace() -> anyhow::Result<()> {
    let chain = MockBech32::new("mock");
    let client = AbstractClient::builder(chain.clone()).build_mock()?;

    let namespace = Namespace::new("namespace")?;
    let publisher: Publisher<MockBech32> = client
        .account_builder()
        .namespace(namespace.clone())
        .build()?
        .publisher()?;

    let publisher_from_namespace: Publisher<MockBech32> = client
        .account_builder()
        .namespace(namespace)
        .build()?
        .publisher()?;

    assert_eq!(
        publisher.account().info()?,
        publisher_from_namespace.account().info()?
    );

    Ok(())
}

#[test]
fn can_publish_and_install_app() -> anyhow::Result<()> {
    let chain = MockBech32::new("mock");
    let client = AbstractClient::builder(chain.clone()).build_mock()?;

    let publisher: Publisher<MockBech32> = client
        .account_builder()
        .namespace(Namespace::new(TEST_NAMESPACE)?)
        .install_on_sub_account(true)
        .build()?
        .publisher()?;

    let publisher_account = publisher.account();
    let publisher_account_address = publisher_account.address()?;

    publisher.publish_app::<MockAppI<MockBech32>>()?;

    // Install app on sub-account
    let my_app: Application<_, MockAppI<_>> =
        publisher_account.install_app::<MockAppI<MockBech32>>(&MockInitMsg {}, &[])?;

    my_app.call_as(&publisher_account_address).do_something()?;

    let something = my_app.get_something()?;

    assert_eq!(MockQueryResponse {}, something);

    // Can get installed application of the account
    let my_app: Application<_, MockAppI<_>> = my_app.account().application()?;
    let something = my_app.get_something()?;
    assert_eq!(MockQueryResponse {}, something);

    let sub_account_details = my_app.account().info()?;
    assert_eq!(
        AccountInfo {
            name: Some(String::from("Sub Account")),
            description: None,
            link: None,
        },
        sub_account_details
    );
    let sub_account_ownership = my_app.account().ownership()?;
    assert_eq!(
        sub_account_ownership.owner,
        GovernanceDetails::SubAccount {
            account: publisher_account_address.to_string(),
        }
    );

    let sub_accounts = publisher.account().sub_accounts()?;
    assert_eq!(sub_accounts.len(), 1);
    assert_eq!(sub_accounts[0].id()?, my_app.account().id()?);

    // Install app on current account
    let publisher = client
        .account_builder()
        .namespace(Namespace::new(TEST_NAMESPACE)?)
        .install_on_sub_account(false)
        .build()?
        .publisher()?;
    let my_adapter: Application<_, MockAppI<_>> =
        publisher.account().install_app(&MockInitMsg {}, &[])?;

    my_adapter
        .call_as(&publisher_account_address)
        .do_something()?;
    let mock_query: MockQueryResponse = my_adapter.get_something()?;

    assert_eq!(MockQueryResponse {}, mock_query);

    let sub_account_details = my_adapter.account().info()?;
    assert_eq!(
        AccountInfo {
            name: Some(String::from("Default Abstract Account")),
            description: None,
            link: None,
        },
        sub_account_details
    );
    let sub_account_ownership = my_adapter.account().ownership()?;
    assert_eq!(
        sub_account_ownership.owner,
        GovernanceDetails::Monarchy {
            monarch: client.sender().to_string()
        }
    );

    Ok(())
}

#[test]
fn can_publish_and_install_adapter() -> anyhow::Result<()> {
    let chain = MockBech32::new("mock");
    let client = AbstractClient::builder(chain.clone()).build_mock()?;

    let account = client
        .account_builder()
        .namespace(Namespace::new(TEST_NAMESPACE)?)
        .build()?;
    let publisher = Publisher::new(&account)?;

    let publisher_account_address = account.address()?;

    publisher.publish_adapter::<AdapterMockInitMsg, MockAdapterI<_>>(AdapterMockInitMsg {})?;

    // Install adapter on sub-account
    let my_adapter: Application<_, MockAdapterI<_>> = account.install_adapter(&[])?;

    my_adapter
        .call_as(&publisher_account_address)
        .execute(&AdapterMockExecMsg {}.into(), &[])?;
    let mock_query: String = my_adapter.query(&AdapterMockQueryMsg::GetSomething {}.into())?;

    assert_eq!(String::from("mock_query"), mock_query);

    let sub_account_details = my_adapter.account().info()?;
    assert_eq!(
        AccountInfo {
            name: Some(String::from("Sub Account")),
            description: None,
            link: None,
        },
        sub_account_details
    );
    let sub_account_ownership = my_adapter.account().ownership()?;
    assert_eq!(
        sub_account_ownership.owner,
        GovernanceDetails::SubAccount {
            account: publisher_account_address.to_string(),
        }
    );

    // Install adapter on current account
    let publisher = client.fetch_publisher(Namespace::new(TEST_NAMESPACE)?)?;
    let my_adapter: Application<_, MockAdapterI<_>> = publisher.account().install_adapter(&[])?;

    my_adapter
        .call_as(&publisher_account_address)
        .execute(&AdapterMockExecMsg {}.into(), &[])?;
    let mock_query: String = my_adapter.query(&AdapterMockQueryMsg::GetSomething {}.into())?;

    assert_eq!(String::from("mock_query"), mock_query);

    let sub_account_details = my_adapter.account().info()?;
    assert_eq!(
        AccountInfo {
            name: Some(String::from("Default Abstract Account")),
            description: None,
            link: None,
        },
        sub_account_details
    );
    let sub_account_ownership = my_adapter.account().ownership()?;
    assert_eq!(
        sub_account_ownership.owner,
        GovernanceDetails::Monarchy {
            monarch: client.sender().to_string()
        }
    );

    Ok(())
}

#[test]
fn can_fetch_account_from_id() -> anyhow::Result<()> {
    let chain = MockBech32::new("mock");
    let client = AbstractClient::builder(chain.clone()).build_mock()?;

    let account1 = client.account_builder().build()?;

    let account2 = client.account_from(account1.id()?)?;

    assert_eq!(account1.info()?, account2.info()?);

    Ok(())
}

#[test]
fn can_fetch_account_from_app() -> anyhow::Result<()> {
    let chain = MockBech32::new("mock");
    let client = AbstractClient::builder(chain.clone()).build_mock()?;

    let app_publisher: Publisher<MockBech32> = client
        .account_builder()
        .namespace(Namespace::new(TEST_NAMESPACE)?)
        .build()?
        .publisher()?;

    app_publisher.publish_app::<MockAppI<MockBech32>>()?;

    let account1 = client
        .account_builder()
        // Install apps in this account
        .install_on_sub_account(false)
        .build()?;

    let app = account1.install_app::<MockAppI<MockBech32>>(&MockInitMsg {}, &[])?;

    let account2 = client.account_from(AccountSource::App(app.address()?))?;

    assert_eq!(account1.info()?, account2.info()?);

    Ok(())
}

#[test]
fn can_install_module_with_dependencies() -> anyhow::Result<()> {
    let chain = MockBech32::new("mock");
    let client = AbstractClient::builder(chain.clone()).build_mock()?;

    let app_publisher: Publisher<MockBech32> = client
        .account_builder()
        .namespace(Namespace::new(TEST_WITH_DEP_NAMESPACE)?)
        .build()?
        .publisher()?;

    let app_dependency_publisher: Publisher<MockBech32> = client
        .account_builder()
        .namespace(Namespace::new(TEST_NAMESPACE)?)
        .build()?
        .publisher()?;

    app_dependency_publisher.publish_app::<MockAppI<_>>()?;
    app_publisher.publish_app::<MockAppWithDepI<_>>()?;

    let my_app: Application<_, MockAppWithDepI<_>> = app_publisher
        .account()
        .install_app_with_dependencies::<MockAppWithDepI<MockBech32>>(
            &MockInitMsg {},
            Empty {},
            &[],
        )?;

    my_app
        .call_as(&app_publisher.account().address()?)
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
        .contains(&AccountModuleInfo {
            id: TEST_MODULE_ID.to_owned(),
            version: cw2::ContractVersion {
                contract: TEST_MODULE_ID.to_owned(),
                version: TEST_VERSION.to_owned()
            },
            address: app_dependency_address,
        }));

    assert!(module_infos_response
        .module_infos
        .contains(&AccountModuleInfo {
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
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let client = AbstractClient::builder(chain.clone()).build_mock()?;

    let name = "name";
    let symbol = "symbol";
    let decimals = 6;
    let description = "A test cw20 token";
    let logo = "link-to-logo";
    let project = "project";
    let marketing = chain.addr_make("marketing");
    let cap = Uint128::from(100u128);
    let starting_balance = Uint128::from(100u128);
    let minter_response = cw20_builder::MinterResponse {
        minter: sender.to_string(),
        cap: Some(cap),
    };

    let cw20: cw20_builder::Cw20Base<MockBech32> = client
        .cw20_builder(name, symbol, decimals)
        .initial_balance(cw20_builder::Cw20Coin {
            address: sender.to_string(),
            amount: starting_balance,
        })
        .admin(sender.to_string())
        .mint(minter_response.clone())
        .marketing(cw20_builder::InstantiateMarketingInfo {
            description: Some(description.to_owned()),
            logo: Some(cw20_builder::Logo::Url(logo.to_owned())),
            project: Some(project.to_owned()),
            marketing: Some(marketing.to_string()),
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
            marketing: Some(marketing),
        },
        marketing_info_response
    );

    let owner_balance: cw20_builder::BalanceResponse = cw20.balance(sender.to_string())?;
    assert_eq!(
        cw20_builder::BalanceResponse {
            balance: starting_balance
        },
        owner_balance
    );
    let transfer_amount = Uint128::from(50u128);
    let recipient = chain.addr_make("user");
    cw20.transfer(transfer_amount, recipient.to_string())?;

    let recipient_balance = cw20.balance(recipient.to_string())?;
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
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let client = AbstractClient::builder(chain.clone()).build_mock()?;

    let name = "name";
    let symbol = "symbol";
    let decimals = 6;

    let cw20: cw20_builder::Cw20Base<MockBech32> = client
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

    let owner_balance: cw20_builder::BalanceResponse = cw20.balance(sender.to_string())?;
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
    let chain = MockBech32::new("mock");
    let client = AbstractClient::builder(chain.clone()).build_mock()?;
    let account = client.account_builder().build()?;

    let coin1 = Coin::new(50u128, "denom1");
    let coin2 = Coin::new(20u128, "denom2");
    let coin3 = Coin::new(10u128, "denom3");
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
    let chain = MockBech32::new("mock");
    let client = AbstractClient::builder(chain.clone()).build_mock()?;

    let app_publisher: Publisher<MockBech32> = client
        .account_builder()
        .namespace(Namespace::new(TEST_WITH_DEP_NAMESPACE)?)
        .build()?
        .publisher()?;

    let app_dependency_publisher: Publisher<MockBech32> = client
        .account_builder()
        .namespace(Namespace::new(TEST_NAMESPACE)?)
        .build()?
        .publisher()?;

    app_dependency_publisher.publish_app::<MockAppI<MockBech32>>()?;
    app_publisher.publish_app::<MockAppWithDepI<MockBech32>>()?;

    let my_app: Application<MockBech32, MockAppWithDepI<MockBech32>> = app_publisher
        .account()
        .install_app_with_dependencies(&MockInitMsg {}, Empty {}, &[])?;

    let dependency: MockAppI<MockBech32> = my_app.module()?;
    dependency.do_something()?;
    Ok(())
}

#[test]
fn can_set_and_query_balance_with_client() -> anyhow::Result<()> {
    let chain = MockBech32::new("mock");
    let client = AbstractClient::builder(chain.clone()).build_mock()?;

    let user = chain.addr_make("user");
    let coin1 = Coin::new(50u128, "denom1");
    let coin2 = Coin::new(20u128, "denom2");
    let coin3 = Coin::new(10u128, "denom3");
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
    let chain = MockBech32::new("mock");
    let client = AbstractClient::builder(chain.clone()).build_mock()?;

    let publisher: Publisher<MockBech32> = client
        .account_builder()
        .namespace(Namespace::new(TEST_NAMESPACE)?)
        .build()?
        .publisher()?;

    publisher.publish_app::<MockAppI<MockBech32>>()?;

    let my_app: Application<MockBech32, MockAppI<MockBech32>> = publisher
        .account()
        .install_app::<MockAppI<MockBech32>>(&MockInitMsg {}, &[])?;

    let dependency_res = my_app.module::<MockAppWithDepI<MockBech32>>();
    assert!(dependency_res.is_err());
    Ok(())
}

/// ANCHOR: mock_integration_test
#[test]
fn can_execute_on_account() -> anyhow::Result<()> {
    let denom = "denom";
    let chain = MockBech32::new("mock");
    let client = AbstractClient::builder(chain.clone()).build_mock()?;
    client.set_balances([(&client.sender(), coins(100, denom).as_slice())])?;

    let user = chain.addr_make("user");

    let account: Account<MockBech32> = client.account_builder().build()?;

    let amount = 20;
    account.execute(
        vec![BankMsg::Send {
            to_address: user.to_string(),
            amount: coins(20, denom),
        }],
        &coins(amount, denom),
    )?;

    assert_eq!(amount, client.query_balance(&user, denom)?.into());
    Ok(())
}
/// ANCHOR_END: mock_integration_test

#[test]
fn resolve_works() -> anyhow::Result<()> {
    let denom = "test_denom";
    let entry = "denom";
    let chain = MockBech32::new("mock");
    let client = AbstractClient::builder(chain.clone())
        .asset(entry, cw_asset::AssetInfoBase::Native(denom.to_owned()))
        .build_mock()?;

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
    let env: MockBech32 = MockBech32::new("mock");
    let sender: Addr = env.sender_addr();

    // Build the client
    let client: AbstractClient<MockBech32> = AbstractClient::builder(env.clone()).build_mock()?;
    // ## ANCHOR_END: build_client

    // ## ANCHOR: balances
    let coins = &[Coin::new(50u128, "eth"), Coin::new(20u128, "btc")];

    // Set a balance
    client.set_balance(&sender, coins)?;

    // Add to an address's balance
    client.add_balance(&sender, &[Coin::new(50u128, "eth")])?;

    // Query an address's balance
    let coin1_balance = client.query_balance(&sender, "eth")?;

    assert_eq!(coin1_balance.u128(), 100);
    // ## ANCHOR_END: balances

    // ## ANCHOR: publisher
    // Create a publisher
    let publisher: Publisher<MockBech32> = client
        .account_builder()
        .namespace(Namespace::from_id(TEST_MODULE_ID)?)
        .build()?
        .publisher()?;

    // Publish an app
    publisher.publish_app::<MockAppI<MockBech32>>()?;
    // ## ANCHOR_END: publisher

    // ## ANCHOR: account
    let accounti: Account<MockBech32> = client.account_builder().build()?;

    // ## ANCHOR: app_interface
    // Install an app
    let app: Application<MockBech32, MockAppI<MockBech32>> =
        accounti.install_app::<MockAppI<MockBech32>>(&MockInitMsg {}, &[])?;
    // ## ANCHOR_END: account
    // Call a function on the app
    app.do_something()?;

    // Call as someone else
    let account: Addr = accounti.address()?;
    app.call_as(&account).do_something()?;

    // Query the app
    let something: MockQueryResponse = app.get_something()?;
    // ## ANCHOR_END: app_interface

    // ## ANCHOR: account_helpers
    // Get account info
    let account_info: AccountInfo = accounti.info()?;
    // Get the owner
    let owner: Addr = accounti.owner()?;
    // Add or set balance
    accounti.add_balance(&[Coin::new(100u128, "btc")])?;
    // ...
    // ## ANCHOR_END: account_helpers

    assert_eq!(
        AccountInfo {
            name: Some(String::from("Default Abstract Account")),
            description: None,
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
    let chain = MockBech32::new("mock");

    // Build the client
    let client: AbstractClient<MockBech32> = AbstractClient::builder(chain.clone()).build_mock()?;

    let account = client.account_builder().build()?;
    let abstract_account: &abstract_interface::AccountI<MockBech32> = account.as_ref();
    assert_eq!(abstract_account.id()?, AccountId::local(1));
    Ok(())
}

#[test]
fn can_use_adapter_object_after_publishing() -> anyhow::Result<()> {
    let chain = MockBech32::new("mock");
    let client: AbstractClient<MockBech32> = AbstractClient::builder(chain.clone()).build_mock()?;
    let publisher = client
        .account_builder()
        .namespace(Namespace::new(TEST_NAMESPACE)?)
        .build()?
        .publisher()?;

    let adapter = publisher
        .publish_adapter::<AdapterMockInitMsg, MockAdapterI<MockBech32>>(AdapterMockInitMsg {})?;
    let module_data: ModuleDataResponse = adapter.query(&abstract_std::adapter::QueryMsg::Base(
        abstract_std::adapter::BaseQueryMsg::ModuleData {},
    ))?;

    assert_eq!(
        module_data,
        ModuleDataResponse {
            module_id: abstract_adapter::mock::MOCK_ADAPTER.module_id().to_owned(),
            version: abstract_adapter::mock::MOCK_ADAPTER.version().to_owned(),
            dependencies: abstract_adapter::mock::MOCK_ADAPTER
                .dependencies()
                .iter()
                .map(Dependency::from)
                .map(Into::into)
                .collect(),
            metadata: Some(TEST_METADATA.to_owned())
        }
    );
    Ok(())
}

#[test]
fn can_register_dex_with_client() -> anyhow::Result<()> {
    let dexes = vec!["foo".to_owned(), "bar".to_owned()];
    let chain = MockBech32::new("mock");

    let client: AbstractClient<MockBech32> = AbstractClient::builder(chain.clone())
        .dexes(dexes.clone())
        .build_mock()?;

    let dexes_response = client.name_service().registered_dexes()?;
    assert_eq!(
        dexes_response,
        abstract_std::ans_host::RegisteredDexesResponse { dexes }
    );
    Ok(())
}

#[test]
fn can_customize_sub_account() -> anyhow::Result<()> {
    let chain = MockBech32::new("mock");
    let client = AbstractClient::builder(chain.clone()).build_mock()?;
    let account = client.account_builder().build()?;
    let sub_account = client
        .account_builder()
        .name("foo-bar")
        .sub_account(&account)
        .build()?;

    let info = sub_account.info()?;
    assert_eq!(info.name.unwrap(), "foo-bar");

    // Account aware of sub account
    let sub_accounts = account.sub_accounts()?;
    assert_eq!(sub_accounts.len(), 1);
    assert_eq!(sub_accounts[0].id()?, sub_account.id()?);
    Ok(())
}

#[test]
fn cant_create_sub_accounts_for_another_user() -> anyhow::Result<()> {
    let chain = MockBech32::new("mock");
    let client = AbstractClient::builder(chain.clone()).build_mock()?;
    let account = client.account_builder().build()?;
    let result = client
        .account_builder()
        .name("foo-bar")
        .ownership(GovernanceDetails::SubAccount {
            account: account.address()?.into_string(),
        })
        .build();

    // No debug on `Account<Chain>`
    let Err(AbstractClientError::Interface(abstract_interface::AbstractInterfaceError::Orch(err))) =
        result
    else {
        panic!("Expected cw-orch error")
    };
    let err: AccountError = err.downcast().unwrap();
    assert!(matches!(
        err,
        AccountError::SubAccountCreatorNotAccount { .. }
    ));
    Ok(())
}

#[test]
fn install_adapter_on_account_builder() -> anyhow::Result<()> {
    let chain = MockBech32::new("mock");
    let client = AbstractClient::builder(chain.clone()).build_mock()?;

    let publisher: Publisher<MockBech32> = client
        .account_builder()
        .namespace(Namespace::new(TEST_NAMESPACE)?)
        .build()?
        .publisher()?;

    // Publish adapter
    let adapter: MockAdapterI<_> = publisher.publish_adapter(AdapterMockInitMsg {})?;

    let account = client
        .account_builder()
        .install_adapter::<MockAdapterI<MockBech32>>()?
        .build()?;
    let modules = account.module_infos()?.module_infos;
    let adapter_info = modules
        .iter()
        .find(|module| module.id == MockAdapterI::<MockBech32>::module_id())
        .expect("Adapter not found");

    assert_eq!(
        *adapter_info,
        AccountModuleInfo {
            id: MockAdapterI::<MockBech32>::module_id().to_owned(),
            version: cw2::ContractVersion {
                contract: MockAdapterI::<MockBech32>::module_id().to_owned(),
                version: MockAdapterI::<MockBech32>::module_version().to_owned()
            },
            address: adapter.address()?,
        }
    );

    Ok(())
}

#[test]
fn install_application_on_account_builder() -> anyhow::Result<()> {
    let chain = MockBech32::new("mock");
    let client = AbstractClient::builder(chain.clone()).build_mock()?;

    let publisher: Publisher<MockBech32> = client
        .account_builder()
        .namespace(Namespace::new(TEST_NAMESPACE)?)
        .build()?
        .publisher()?;

    // Publish app
    publisher.publish_app::<MockAppI<MockBech32>>()?;

    let account = client
        .account_builder()
        .install_app::<MockAppI<MockBech32>>(&MockInitMsg {})?
        .build()?;

    let my_app = account.application::<MockAppI<_>>()?;

    let something = my_app.get_something()?;
    assert_eq!(MockQueryResponse {}, something);

    let modules = account.module_infos()?.module_infos;
    let app_info = modules
        .iter()
        .find(|module| module.id == MockAppI::<MockBech32>::module_id())
        .expect("Application not found");

    assert_eq!(
        *app_info,
        AccountModuleInfo {
            id: MockAppI::<MockBech32>::module_id().to_owned(),
            version: cw2::ContractVersion {
                contract: MockAppI::<MockBech32>::module_id().to_owned(),
                version: MockAppI::<MockBech32>::module_version().to_owned()
            },
            address: my_app.address()?,
        }
    );
    Ok(())
}

#[test]
fn auto_funds_work() -> anyhow::Result<()> {
    // Give enough tokens for the owner
    let chain = MockBech32::new("mock");
    let owner = chain.sender_addr();
    chain.set_balance(&owner, coins(50, TTOKEN))?;

    let client = AbstractClient::builder(chain.clone()).build_mock()?;

    let publisher: Publisher<MockBech32> = client
        .account_builder()
        .namespace(Namespace::new(TEST_NAMESPACE)?)
        .build()?
        .publisher()?;
    let _: MockAdapterI<_> = publisher.publish_adapter(AdapterMockInitMsg {})?;

    client.registry().update_module_configuration(
        TEST_MODULE_NAME.to_owned(),
        Namespace::new(TEST_NAMESPACE)?,
        abstract_std::registry::UpdateModule::Versioned {
            version: MockAdapterI::<MockBech32>::module_version().to_owned(),
            metadata: None,
            monetization: Some(abstract_std::objects::module::Monetization::InstallFee(
                FixedFee::new(&Coin {
                    denom: TTOKEN.to_owned(),
                    amount: Uint128::new(50),
                }),
            )),
            instantiation_funds: None,
        },
    )?;
    let mut account_builder = client.account_builder();

    // User can guard his funds
    account_builder
        .name("bob")
        .install_adapter::<MockAdapterI<MockBech32>>()?
        .auto_fund_assert(|c| c[0].amount < Uint128::new(50));
    let e = account_builder.build().unwrap_err();
    assert!(matches!(e, AbstractClientError::AutoFundsAssertFailed(_)));

    // Or can enable auto_fund and create account if have enough funds
    let account = account_builder.auto_fund().build()?;
    let info = account.info()?;
    assert_eq!(info.name.unwrap(), "bob");

    // funds used
    let balance = client.environment().query_balance(&owner, TTOKEN)?;
    assert!(balance.is_zero());
    Ok(())
}

#[test]
fn install_application_with_deps_on_account_builder() -> anyhow::Result<()> {
    let chain = MockBech32::new("mock");
    let client = AbstractClient::builder(chain.clone()).build_mock()?;

    let app_publisher: Publisher<MockBech32> = client
        .account_builder()
        .namespace(Namespace::new(TEST_WITH_DEP_NAMESPACE)?)
        .build()?
        .publisher()?;

    let app_dependency_publisher: Publisher<MockBech32> = client
        .account_builder()
        .namespace(Namespace::new(TEST_NAMESPACE)?)
        .build()?
        .publisher()?;

    // Publish apps
    app_dependency_publisher.publish_app::<MockAppI<MockBech32>>()?;
    app_publisher.publish_app::<MockAppWithDepI<MockBech32>>()?;

    let account = client
        .account_builder()
        .install_app_with_dependencies::<MockAppWithDepI<MockBech32>>(&MockInitMsg {}, Empty {})?
        .build()?;

    let modules = account.module_infos()?.module_infos;

    // Check dependency
    let dep_app = account.application::<MockAppI<_>>()?;
    let something = dep_app.get_something()?;
    assert_eq!(MockQueryResponse {}, something);

    let app_info = modules
        .iter()
        .find(|module| module.id == MockAppI::<MockBech32>::module_id())
        .expect("Dependency of an application not found");

    assert_eq!(
        *app_info,
        AccountModuleInfo {
            id: MockAppI::<MockBech32>::module_id().to_owned(),
            version: cw2::ContractVersion {
                contract: MockAppI::<MockBech32>::module_id().to_owned(),
                version: MockAppI::<MockBech32>::module_version().to_owned()
            },
            address: dep_app.address()?,
        }
    );

    // Check app itself
    let my_app = account.application::<MockAppWithDepI<_>>()?;

    my_app
        .call_as(&app_publisher.account().address()?)
        .do_something()?;

    let something = my_app.get_something()?;

    assert_eq!(MockQueryResponse {}, something);

    let app_info = modules
        .iter()
        .find(|module| module.id == MockAppWithDepI::<MockBech32>::module_id())
        .expect("Application not found");

    assert_eq!(
        *app_info,
        AccountModuleInfo {
            id: MockAppWithDepI::<MockBech32>::module_id().to_owned(),
            version: cw2::ContractVersion {
                contract: MockAppWithDepI::<MockBech32>::module_id().to_owned(),
                version: MockAppWithDepI::<MockBech32>::module_version().to_owned()
            },
            address: my_app.address()?,
        }
    );
    Ok(())
}

#[test]
fn authorize_app_on_adapters() -> anyhow::Result<()> {
    let chain = MockBech32::new("mock");
    let client = AbstractClient::builder(chain.clone()).build_mock()?;

    let publisher: Publisher<MockBech32> = client
        .account_builder()
        .namespace(Namespace::new(TEST_NAMESPACE)?)
        .build()?
        .publisher()?;
    let app_publisher: Publisher<MockBech32> = client
        .account_builder()
        .namespace(Namespace::new(TEST_WITH_DEP_NAMESPACE)?)
        .build()?
        .publisher()?;

    // Publish adapter and app
    let adapter =
        publisher.publish_adapter::<_, MockAdapterI<MockBech32>>(AdapterMockInitMsg {})?;
    app_publisher.publish_app::<MockAppWithDepI<MockBech32>>()?;

    let account = client
        .account_builder()
        .install_app_with_dependencies::<MockAppWithDepI<MockBech32>>(&MockInitMsg {}, Empty {})?
        .build()?;

    // Authorize app on adapter
    let app: Application<MockBech32, MockAppWithDepI<MockBech32>> = account.application()?;
    app.authorize_on_adapters(&[abstract_adapter::mock::MOCK_ADAPTER.module_id()])?;

    // Check it authorized
    let authorized_addrs_resp: AuthorizedAddressesResponse = adapter.query(
        &abstract_std::adapter::BaseQueryMsg::AuthorizedAddresses {
            account_address: app.account().address()?.to_string(),
        }
        .into(),
    )?;
    assert_eq!(authorized_addrs_resp.addresses, vec![app.address()?]);
    Ok(())
}

#[test]
fn create_account_with_expected_account_id() -> anyhow::Result<()> {
    let chain = MockBech32::new("mock");
    let client = AbstractClient::builder(chain.clone()).build_mock()?;

    // Check it fails on wrong account_id
    let next_id = client.random_account_id()?;
    let err = client
        .account_builder()
        .expected_account_id(10)
        .build()
        .unwrap_err();
    let AbstractClientError::Interface(abstract_interface::AbstractInterfaceError::Orch(err)) = err
    else {
        panic!("Expected cw-orch error")
    };
    let err: RegistryError = err.downcast().unwrap();
    assert_eq!(
        err,
        RegistryError::InvalidAccountSequence {
            expected: 1,
            actual: 10,
        }
    );

    // Can create if right id
    let account = client
        .account_builder()
        .expected_account_id(next_id)
        .build()?;

    // Check it fails on wrong account_id for sub-accounts
    let next_id = client.random_account_id()?;
    let err = client
        .account_builder()
        .sub_account(&account)
        .expected_account_id(0)
        .build()
        .unwrap_err();
    let AbstractClientError::Interface(abstract_interface::AbstractInterfaceError::Orch(err)) = err
    else {
        panic!("Expected cw-orch error")
    };
    let err: RegistryError = err.downcast().unwrap();
    assert_eq!(
        err,
        RegistryError::AccountAlreadyExists(AccountId::local(0))
    );

    // Can create sub-account if right id
    let sub_account = client
        .account_builder()
        .sub_account(&account)
        .expected_account_id(next_id)
        .build()?;
    let sub_accounts = account.sub_accounts()?;
    assert_eq!(sub_accounts[0].id()?, sub_account.id()?);
    Ok(())
}

#[test]
fn instantiate2_addr() -> anyhow::Result<()> {
    let chain = MockBech32::new("mock");
    let client = AbstractClient::builder(chain.clone()).build_mock()?;

    let publisher: Publisher<MockBech32> = client
        .account_builder()
        .namespace(Namespace::new(TEST_NAMESPACE)?)
        .build()?
        .publisher()?;

    publisher.publish_app::<MockAppI<MockBech32>>()?;

    let account_id = AccountId::local(client.random_account_id()?);
    let expected_addr = client.module_instantiate2_address::<MockAppI<MockBech32>>(&account_id)?;

    let sub_account = client
        .account_builder()
        .sub_account(publisher.account())
        .expected_account_id(account_id.seq())
        .install_app::<MockAppI<MockBech32>>(&MockInitMsg {})?
        .build()?;
    let application = sub_account.application::<MockAppI<_>>()?;

    assert_eq!(application.address()?, expected_addr);
    Ok(())
}

#[test]
fn instantiate2_raw_addr() -> anyhow::Result<()> {
    let chain = MockBech32::new("mock");
    let client = AbstractClient::builder(chain.clone()).build_mock()?;

    let next_seq = client.random_account_id()?;
    let account_id = AccountId::local(next_seq);

    let account_addr = client.module_instantiate2_address_raw(
        &account_id,
        ModuleInfo::from_id_latest(abstract_std::ACCOUNT)?,
    )?;
    let account = client
        .account_builder()
        .expected_account_id(next_seq)
        .build()?;

    assert_eq!(account.address()?, account_addr);
    Ok(())
}

#[test]
fn instantiate2_random_seq() -> anyhow::Result<()> {
    let chain = MockBech32::new("mock");
    let client = AbstractClient::builder(chain.clone()).build_mock()?;

    let next_seq = client.random_account_id()?;
    let account_id = AccountId::local(next_seq);

    let account_addr = client.module_instantiate2_address_raw(
        &account_id,
        ModuleInfo::from_id_latest(abstract_std::ACCOUNT)?,
    )?;
    let account = client
        .account_builder()
        .expected_account_id(next_seq)
        .build()?;

    assert_eq!(account.address()?, account_addr);
    Ok(())
}

#[test]
fn install_same_app_on_different_accounts() -> anyhow::Result<()> {
    let chain = MockBech32::new("mock");
    let client = AbstractClient::builder(chain.clone()).build_mock()?;

    let app_publisher: Publisher<MockBech32> = client
        .account_builder()
        .namespace(Namespace::new(TEST_NAMESPACE)?)
        .build()?
        .publisher()?;

    app_publisher.publish_app::<MockAppI<MockBech32>>()?;

    let account1 = client
        .account_builder()
        .install_app::<MockAppI<MockBech32>>(&MockInitMsg {})?
        .build()?;

    let account2 = client
        .account_builder()
        .install_app::<MockAppI<MockBech32>>(&MockInitMsg {})?
        .build()?;

    let account3 = client
        .account_builder()
        .sub_account(&account1)
        .install_app::<MockAppI<MockBech32>>(&MockInitMsg {})?
        .build()?;

    let mock_app1 = account1.application::<MockAppI<MockBech32>>()?;
    let mock_app2 = account2.application::<MockAppI<MockBech32>>()?;
    let mock_app3 = account3.application::<MockAppI<MockBech32>>()?;

    assert_ne!(mock_app1.id(), mock_app2.id());
    assert_ne!(mock_app1.id(), mock_app3.id());
    assert_ne!(mock_app2.id(), mock_app3.id());

    Ok(())
}

#[test]
fn install_ibc_client_on_creation() -> anyhow::Result<()> {
    let chain = MockBech32::new("mock");
    let client = AbstractClient::builder(chain.clone()).build_mock()?;

    let account = client
        .account_builder()
        .install_adapter::<IbcClient<MockBech32>>()?
        .build()?;
    let ibc_module_addr = account.module_addresses(vec![IBC_CLIENT.to_owned()])?;
    assert_eq!(ibc_module_addr.modules[0].0, IBC_CLIENT);
    Ok(())
}

#[test]
fn module_installed() -> anyhow::Result<()> {
    let chain = MockBech32::new("mock");
    let client = AbstractClient::builder(chain.clone()).build_mock()?;

    let account = client
        .account_builder()
        .install_adapter::<IbcClient<MockBech32>>()?
        .build()?;
    let installed = account.module_installed(IBC_CLIENT)?;
    assert!(installed);
    let installed = account.module_installed(TEST_MODULE_ID)?;
    assert!(!installed);
    Ok(())
}

#[test]
fn module_version_installed() -> anyhow::Result<()> {
    let chain = MockBech32::new("mock");
    let client = AbstractClient::builder(chain.clone()).build_mock()?;

    let account = client
        .account_builder()
        .install_adapter::<IbcClient<MockBech32>>()?
        .build()?;

    let installed = account.module_version_installed(ModuleInfo::from_id_latest(IBC_CLIENT)?)?;
    assert!(installed);
    let installed = account.module_version_installed(ModuleInfo::from_id(
        IBC_CLIENT,
        abstract_std::objects::module::ModuleVersion::Version(TEST_VERSION.to_string()),
    )?)?;
    assert!(installed);
    let installed =
        account.module_version_installed(ModuleInfo::from_id_latest(TEST_MODULE_ID)?)?;
    assert!(!installed);
    let installed = account.module_version_installed(ModuleInfo::from_id(
        IBC_CLIENT,
        abstract_std::objects::module::ModuleVersion::Version("0.1.0".to_string()),
    )?)?;
    assert!(!installed);
    Ok(())
}

#[test]
fn retrieve_account_builder_install_missing_modules() -> anyhow::Result<()> {
    let chain = MockBech32::new("mock");
    let client = AbstractClient::builder(chain.clone()).build_mock()?;

    let app_publisher: Publisher<MockBech32> = client
        .account_builder()
        .namespace(Namespace::new(TEST_NAMESPACE)?)
        .build()?
        .publisher()?;

    app_publisher.publish_app::<MockAppI<MockBech32>>()?;

    assert!(!app_publisher.account().module_installed(TEST_MODULE_ID)?);
    let account = client
        .account_builder()
        .namespace(Namespace::new(TEST_NAMESPACE)?)
        .install_app::<MockAppI<MockBech32>>(&MockInitMsg {})?
        .build()?;
    // Same account
    assert_eq!(app_publisher.account().id()?, account.id()?);
    // Installed from builder after account was created
    assert!(account.module_installed(TEST_MODULE_ID)?);
    Ok(())
}

#[test]
fn module_status() -> anyhow::Result<()> {
    let chain = MockBech32::new("mock");
    let client = AbstractClient::builder(chain.clone()).build_mock()?;
    let admin = AbstractClient::mock_admin(&chain);
    client
        .registry()
        .call_as(&admin)
        .update_config(None, Some(false))?;

    let app_publisher: Publisher<MockBech32> = client
        .account_builder()
        .namespace(Namespace::new(TEST_NAMESPACE)?)
        .build()?
        .publisher()?;

    let module_info = MockAppI::<MockBech32>::module_info().unwrap();
    let module_status = client.module_status(module_info.clone())?;
    assert!(module_status.is_none());

    app_publisher.publish_app::<MockAppI<MockBech32>>()?;
    let module_status = client.module_status(module_info.clone())?;
    assert_eq!(
        module_status,
        Some(abstract_std::objects::module::ModuleStatus::Pending)
    );

    client
        .registry()
        .call_as(&admin)
        .approve_or_reject_modules(vec![module_info.clone()], vec![])?;
    let module_status = client.module_status(module_info.clone())?;
    assert_eq!(
        module_status,
        Some(abstract_std::objects::module::ModuleStatus::Registered)
    );

    client.registry().yank_module(module_info.clone())?;
    let module_status = client.module_status(module_info)?;
    assert_eq!(
        module_status,
        Some(abstract_std::objects::module::ModuleStatus::Yanked)
    );
    Ok(())
}

#[test]
fn cant_upload_module_with_non_deployed_deps() -> anyhow::Result<()> {
    let chain = MockBech32::new("mock");
    let client = AbstractClient::builder(chain.clone()).build_mock()?;

    let app_publisher: Publisher<MockBech32> = client
        .account_builder()
        .namespace(Namespace::new(TEST_WITH_DEP_NAMESPACE)?)
        .build()?
        .publisher()?;

    let app_dependency_publisher: Publisher<MockBech32> = client
        .account_builder()
        .namespace(Namespace::new(TEST_NAMESPACE)?)
        .build()?
        .publisher()?;

    // Matching dep not uploaded - can't upload app
    let res = app_publisher.publish_app::<MockAppWithDepI<_>>();
    assert!(matches!(
        res,
        Err(AbstractClientError::Interface(
            abstract_interface::AbstractInterfaceError::NoMatchingModule(_)
        ))
    ));

    // Now publish dep and we can upload app then
    app_dependency_publisher.publish_app::<MockAppI<_>>()?;
    let res = app_publisher.publish_app::<MockAppWithDepI<_>>();
    assert!(res.is_ok());
    Ok(())
}

#[test]
fn register_service() -> anyhow::Result<()> {
    let chain = MockBech32::new("mock");
    let client = AbstractClient::builder(chain.clone()).build_mock()?;

    let service_publisher: Publisher<MockBech32> = client
        .account_builder()
        .namespace(Namespace::new(TEST_NAMESPACE)?)
        .build()?
        .publisher()?;

    service_publisher.publish_service::<MockService<MockBech32>>(&MockMsg {})?;

    // Can get service without account
    let service = client.service::<MockService<MockBech32>>()?;
    let res: String = service.query(&MockMsg {})?;
    assert_eq!(res, "test");

    // Or from an account with Application if installed
    let account = client
        .account_builder()
        .namespace(Namespace::new(TEST_NAMESPACE)?)
        .install_service::<MockService<MockBech32>>(&MockMsg {})?
        .build()?;

    let service = account.application::<MockService<MockBech32>>()?;
    let res: String = service.query(&MockMsg {})?;
    assert_eq!(res, "test");
    Ok(())
}

#[test]
fn ans_balance() -> anyhow::Result<()> {
    let chain = MockBech32::new("mock");
    let client = AbstractClient::builder(chain.clone())
        .asset(
            "mock",
            cw_asset::AssetInfoBase::Native("mockdenom".to_owned()),
        )
        .build_mock()?;

    chain.add_balance(&chain.sender_addr(), coins(101, "mockdenom"))?;

    let balance = client
        .name_service()
        .balance(&chain.sender_addr(), &AssetEntry::new("mock"))?;
    assert_eq!(balance, Uint128::new(101));
    Ok(())
}

// Tests wether using the Account builder with install_on_subaccount on an existing account installs given apps
#[test]
fn account_fetcher_shouldnt_install_module_on_existing_account() -> anyhow::Result<()> {
    let chain = MockBech32::new("mock");
    let client = AbstractClient::builder(chain.clone()).build_mock()?;

    let app_publisher: Publisher<MockBech32> = client
        .account_builder()
        .namespace(Namespace::new(TEST_WITH_DEP_NAMESPACE)?)
        .build()?
        .publisher()?;

    let app_dependency_publisher: Publisher<MockBech32> = client
        .account_builder()
        .namespace(Namespace::new(TEST_NAMESPACE)?)
        .build()?
        .publisher()?;

    app_dependency_publisher.publish_app::<MockAppI<_>>()?;
    let res = app_publisher.publish_app::<MockAppWithDepI<_>>();
    assert!(res.is_ok());

    const NEW_NAMESPACE: &str = "new-namespace";

    // We create an account
    let account = client
        .account_builder()
        .namespace(NEW_NAMESPACE.try_into()?)
        .build()?;

    client.fetch_or_build(NEW_NAMESPACE.try_into()?, |builder| {
        builder
            .install_on_sub_account(true)
            .install_app::<MockAppWithDepI<MockBech32>>(&MockInitMsg {})
            .unwrap()
    })?;
    assert!(!account.module_installed(TEST_MODULE_ID)?);
    Ok(())
}

// Tests wether using the Account builder with install_on_subaccount on an existing account installs given apps
#[test]
fn account_builder_should_install_on_subaccount() -> anyhow::Result<()> {
    let chain = MockBech32::new("mock");
    let client = AbstractClient::builder(chain.clone()).build_mock()?;

    let app_publisher: Publisher<MockBech32> = client
        .account_builder()
        .namespace(Namespace::new(TEST_WITH_DEP_NAMESPACE)?)
        .build()?
        .publisher()?;

    let app_dependency_publisher: Publisher<MockBech32> = client
        .account_builder()
        .namespace(Namespace::new(TEST_NAMESPACE)?)
        .build()?
        .publisher()?;

    app_dependency_publisher.publish_app::<MockAppI<_>>()?;
    let res = app_publisher.publish_app::<MockAppWithDepI<_>>();
    assert!(res.is_ok());

    const NEW_NAMESPACE: &str = "new-namespace";

    // We create an account
    let account = client
        .account_builder()
        .install_on_sub_account(true)
        .install_service::<IbcClient<MockBech32>>(&Empty {})?
        .install_app::<MockAppI<MockBech32>>(&MockInitMsg {})?
        .install_app::<MockAppWithDepI<MockBech32>>(&MockInitMsg {})?
        .namespace(NEW_NAMESPACE.try_into()?)
        .build()?;

    assert!(!account.module_installed(TEST_MODULE_ID)?);

    // Fetch the subaccount
    let subaccounts = account.sub_accounts()?;
    assert_eq!(subaccounts.len(), 1);
    Ok(())
}
