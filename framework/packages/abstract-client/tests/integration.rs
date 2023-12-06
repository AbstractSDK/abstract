use abstract_app::mock::{
    interface::MockAppInterface, MockExecMsgFns, MockInitMsg, MockQueryMsgFns, MockQueryResponse,
};
use abstract_client::{
    account::Account, application::Application, client::AbstractClient, publisher::Publisher,
};
use abstract_core::{
    manager::state::AccountInfo,
    objects::{gov_type::GovernanceDetails, namespace::Namespace, AccountId, AssetEntry},
};
use abstract_interface::VCQueryFns;
use cosmwasm_std::Addr;
use cw_asset::AssetInfoUnchecked;
use cw_orch::prelude::{CallAs, Mock};

const ADMIN: &str = "admin";

#[test]
fn can_create_account_without_optional_parameters() -> anyhow::Result<()> {
    let client = AbstractClient::builder(ADMIN).build()?;

    let account: Account<Mock> = client.account_builder().build()?;

    let account_info = account.get_account_info()?;
    assert_eq!(
        AccountInfo {
            name: String::from("Default Abstract Account"),
            chain_id: String::from("cosmos-testnet-14002"),
            description: None,
            governance_details: GovernanceDetails::Monarchy {
                monarch: Addr::unchecked(ADMIN)
            },
            link: None,
        },
        account_info
    );

    Ok(())
}

#[test]
fn can_create_account_with_optional_parameters() -> anyhow::Result<()> {
    let asset = "asset";

    let client = AbstractClient::builder(ADMIN)
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
    let client = AbstractClient::builder(ADMIN).build()?;

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
    let client = AbstractClient::builder(ADMIN).build()?;

    let publisher: Publisher<Mock> = client.publisher_builder().build()?;

    let account_info = publisher.account().get_account_info()?;
    assert_eq!(
        AccountInfo {
            name: String::from("Default Abstract Account"),
            chain_id: String::from("cosmos-testnet-14002"),
            description: None,
            governance_details: GovernanceDetails::Monarchy {
                monarch: Addr::unchecked(ADMIN)
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
    let client = AbstractClient::builder(ADMIN)
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
    let client = AbstractClient::builder(ADMIN).build()?;

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
    let client = AbstractClient::builder(ADMIN).build()?;

    let publisher: Publisher<Mock> = client.publisher_builder().namespace("tester").build()?;

    let publisher_admin = publisher.admin()?;
    let publisher_proxy = publisher.proxy()?;

    publisher.publish_app::<MockAppInterface<Mock>>()?;

    let my_app: Application<Mock, MockAppInterface<Mock>> =
        publisher.install_app::<MockAppInterface<Mock>>(&MockInitMsg {}, &[])?;

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
    let client = AbstractClient::builder(ADMIN).build()?;

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
    let client = AbstractClient::builder(ADMIN).build()?;

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
