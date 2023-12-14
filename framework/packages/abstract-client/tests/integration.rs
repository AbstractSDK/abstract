use abstract_app::mock::{
    interface::MockAppInterface, mock_app_dependency::interface::MockAppDependencyInterface,
    MockExecMsgFns, MockInitMsg, MockQueryMsgFns, MockQueryResponse,
};
use abstract_client::{
    account::Account,
    application::Application,
    client::test_utils::cw20_builder::{self, Cw20ExecuteMsgFns, Cw20QueryMsgFns},
    client::AbstractClient,
    publisher::Publisher,
};
use abstract_core::{
    manager::{
        state::AccountInfo, ManagerModuleInfo, ModuleAddressesResponse, ModuleInfosResponse,
    },
    objects::{gov_type::GovernanceDetails, namespace::Namespace, AccountId, AssetEntry},
};
use abstract_interface::VCQueryFns;
use abstract_testing::{
    prelude::{
        TEST_DEPENDENCY_MODULE_ID, TEST_DEPENDENCY_NAMESPACE, TEST_MODULE_ID, TEST_NAMESPACE,
        TEST_VERSION,
    },
    OWNER,
};
use cosmwasm_std::{Addr, Empty, Uint128};
use cw_asset::AssetInfoUnchecked;
use cw_orch::prelude::{CallAs, Mock};
use cw_ownable::Ownership;

#[test]
fn can_create_account_without_optional_parameters() -> anyhow::Result<()> {
    let client = AbstractClient::builder(OWNER).build()?;

    let account: Account<Mock> = client.account_builder().build()?;

    let account_info = account.get_account_info()?;
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

    let client = AbstractClient::builder(OWNER)
        .asset(asset, AssetInfoUnchecked::native(asset))
        .build()?;

    let name = "test-account";
    let description = "description";
    let link = "https://abstract.money";
    let governance_details = GovernanceDetails::Monarchy {
        monarch: String::from("monarch"),
    };
    let namespace = "test-namespace";
    let base_asset = AssetEntry::new(asset);
    let account: Account<Mock> = client
        .account_builder()
        .name(name)
        .link(link)
        .description(description)
        .governance_details(governance_details.clone())
        .namespace(namespace)
        .base_asset(base_asset)
        .build()?;

    let account_info = account.get_account_info()?;
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
        .namespace(Namespace::new(namespace)?)?
        .account_id;
    assert_eq!(account_id, AccountId::local(1));

    Ok(())
}

#[test]
fn can_get_account_from_namespace() -> anyhow::Result<()> {
    let client = AbstractClient::builder(OWNER).build()?;

    let namespace = "namespace";
    let account: Account<Mock> = client.account_builder().namespace(namespace).build()?;

    let account_from_namespace: Account<Mock> = client.get_account_from_namespace(namespace)?;

    assert_eq!(
        account.get_account_info()?,
        account_from_namespace.get_account_info()?
    );

    Ok(())
}

#[test]
fn can_create_publisher_without_optional_parameters() -> anyhow::Result<()> {
    let client = AbstractClient::builder(OWNER).build()?;

    let publisher: Publisher<Mock> = client.publisher_builder().build()?;

    let account_info = publisher.account().get_account_info()?;
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
    let client = AbstractClient::builder(OWNER)
        .asset(asset, AssetInfoUnchecked::native(asset))
        .build()?;

    let name = "test-account";
    let description = "description";
    let link = "https://abstract.money";
    let governance_details = GovernanceDetails::Monarchy {
        monarch: String::from("monarch"),
    };
    let namespace = "test-namespace";
    let base_asset = AssetEntry::new(asset);
    let publisher: Publisher<Mock> = client
        .publisher_builder()
        .name(name)
        .link(link)
        .description(description)
        .governance_details(governance_details.clone())
        .namespace(namespace)
        .base_asset(base_asset)
        .build()?;

    let account_info = publisher.account().get_account_info()?;
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
        .namespace(Namespace::new(namespace)?)?
        .account_id;
    assert_eq!(account_id, AccountId::local(1));

    Ok(())
}

#[test]
fn can_get_publisher_from_namespace() -> anyhow::Result<()> {
    let client = AbstractClient::builder(OWNER).build()?;

    let namespace = "namespace";
    let publisher: Publisher<Mock> = client.publisher_builder().namespace(namespace).build()?;

    let publisher_from_namespace: Publisher<Mock> =
        client.get_publisher_from_namespace(namespace)?;

    assert_eq!(
        publisher.account().get_account_info()?,
        publisher_from_namespace.account().get_account_info()?
    );

    Ok(())
}

#[test]
fn can_publish_and_install_app() -> anyhow::Result<()> {
    let client = AbstractClient::builder(OWNER).build()?;

    let publisher: Publisher<Mock> = client
        .publisher_builder()
        .namespace(TEST_DEPENDENCY_NAMESPACE)
        .build()?;

    let publisher_admin = publisher.admin()?;
    let publisher_proxy = publisher.proxy()?;

    publisher.publish_app::<MockAppDependencyInterface<Mock>>()?;

    let my_app: Application<Mock, MockAppDependencyInterface<Mock>> =
        publisher.install_app::<MockAppDependencyInterface<Mock>>(&MockInitMsg {}, &[])?;

    my_app.call_as(&publisher.admin()?).do_something()?;

    let something = my_app.get_something()?;

    assert_eq!(MockQueryResponse {}, something);

    let sub_account_details = my_app.account().get_account_info()?;
    assert_eq!(
        AccountInfo {
            name: String::from("Sub Account"),
            chain_id: String::from("cosmos-testnet-14002"),
            description: None,
            governance_details: GovernanceDetails::SubAccount {
                manager: publisher_admin,
                proxy: publisher_proxy
            },
            link: None,
        },
        sub_account_details
    );

    Ok(())
}

#[test]
fn cannot_create_same_account_twice_when_fetch_flag_is_disabled() -> anyhow::Result<()> {
    let client = AbstractClient::builder(OWNER).build()?;

    let namespace = "namespace";

    // First call succeeds.
    client.account_builder().namespace(namespace).build()?;

    // Second call fails
    let result = client.account_builder().namespace(namespace).build();
    assert!(result.is_err());

    Ok(())
}

#[test]
fn can_create_same_account_twice_when_fetch_flag_is_enabled() -> anyhow::Result<()> {
    let client = AbstractClient::builder(OWNER).build()?;

    let namespace = "namespace";

    let account1 = client.account_builder().namespace(namespace).build()?;

    let account2 = client
        .account_builder()
        .namespace(namespace)
        .fetch_if_namespace_claimed(true)
        .build()?;

    assert_eq!(account1.get_account_info()?, account2.get_account_info()?);

    Ok(())
}

#[test]
fn can_install_module_with_dependencies() -> anyhow::Result<()> {
    let client = AbstractClient::builder(OWNER).build()?;

    let app_publisher: Publisher<Mock> = client
        .publisher_builder()
        .namespace(TEST_NAMESPACE)
        .build()?;

    let app_dependency_publisher: Publisher<Mock> = client
        .publisher_builder()
        .namespace(TEST_DEPENDENCY_NAMESPACE)
        .build()?;

    app_dependency_publisher.publish_app::<MockAppDependencyInterface<Mock>>()?;
    app_publisher.publish_app::<MockAppInterface<Mock>>()?;

    let my_app: Application<Mock, MockAppInterface<Mock>> = app_publisher
        .install_app_with_dependencies::<MockAppInterface<Mock>>(&MockInitMsg {}, Empty {}, &[])?;

    my_app.call_as(&app_publisher.admin()?).do_something()?;

    let something = my_app.get_something()?;

    assert_eq!(MockQueryResponse {}, something);

    let module_infos_response: ModuleInfosResponse = my_app.account().module_infos()?;
    let module_addresses_response: ModuleAddressesResponse =
        my_app.account().module_addresses(vec![
            TEST_MODULE_ID.to_owned(),
            TEST_DEPENDENCY_MODULE_ID.to_owned(),
        ])?;

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
        .find(|(module_id, _)| module_id == TEST_DEPENDENCY_MODULE_ID)
        .unwrap()
        .clone()
        .1;

    assert!(module_infos_response
        .module_infos
        .contains(&ManagerModuleInfo {
            id: TEST_DEPENDENCY_MODULE_ID.to_owned(),
            version: cw2::ContractVersion {
                contract: TEST_DEPENDENCY_MODULE_ID.to_owned(),
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
    let client = AbstractClient::builder(OWNER).build()?;

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
    let client = AbstractClient::builder(OWNER).build()?;

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
