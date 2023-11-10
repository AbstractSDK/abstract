mod app;
use abstract_client::{
    account::Account, application::Application, client::AbstractClient, publisher::Publisher,
};
use abstract_core::{
    manager::state::AccountInfo,
    objects::{gov_type::GovernanceDetails, namespace::Namespace, AccountId, AssetEntry},
};
use abstract_interface::{Abstract, VCQueryFns};
use app::{AppInterface, AppQueryMsgFns};
use cosmwasm_std::Addr;
use cw_asset::AssetInfo;
use cw_orch::{
    deploy::Deploy,
    prelude::{CallAs, CwOrchExecute, Mock},
};

use crate::app::{msg::ConfigResponse, AppExecuteMsgFns};

const ADMIN: &str = "admin";

fn deploy_abstract() -> anyhow::Result<(Mock, Abstract<Mock>)> {
    let admin = Addr::unchecked(ADMIN);
    let chain = Mock::new(&admin);
    let abstr = Abstract::deploy_on(chain.clone(), admin.to_string())?;
    Ok((chain, abstr))
}

// TODO:
// Allow using account-id instead of namespace to get publisher in the case where namespace is not
// claimed.
// Handle module dependencies.

#[test]
fn can_create_account_without_optional_parameters() -> anyhow::Result<()> {
    // Set up.
    let (chain, _abstr) = deploy_abstract()?;

    let client: AbstractClient<Mock> = AbstractClient::new(chain.clone())?;

    let account: Account<Mock> = client.new_account().build()?;

    let account_info = account.get_account_info()?;
    assert_eq!(
        AccountInfo {
            name: String::from("Default Abstract Account"),
            chain_id: String::from("cosmos-testnet-14002"),
            description: None,
            governance_details: GovernanceDetails::Monarchy {
                monarch: chain.sender
            },
            link: None,
        },
        account_info
    );

    Ok(())
}

#[test]
fn can_create_account_with_optional_parameters() -> anyhow::Result<()> {
    // Set up.
    let (chain, abstr) = deploy_abstract()?;

    let asset = "asset";

    abstr.ans_host.execute(
        &abstract_core::ans_host::ExecuteMsg::UpdateAssetAddresses {
            to_add: vec![(asset.to_owned(), AssetInfo::native(asset).into())],
            to_remove: vec![],
        },
        None,
    )?;

    let client: AbstractClient<Mock> = AbstractClient::new(chain.clone())?;

    let name = "test-account";
    let description = "description";
    let link = "https://abstract.money";
    let governance_details = GovernanceDetails::Monarchy {
        monarch: String::from("monarch"),
    };
    let namespace = "test-namespace";
    let base_asset = AssetEntry::new(asset);
    let account: Account<Mock> = client
        .new_account()
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
    let account_id = abstr
        .version_control
        .namespace(Namespace::new(namespace)?)?
        .account_id;
    assert_eq!(account_id, AccountId::local(1));

    Ok(())
}

#[test]
fn can_get_account_from_namespace() -> anyhow::Result<()> {
    // Set up.
    let (chain, _abstr) = deploy_abstract()?;

    let client: AbstractClient<Mock> = AbstractClient::new(chain.clone())?;

    let namespace = "namespace";
    let account: Account<Mock> = client.new_account().namespace(namespace).build()?;

    let account_from_namespace: Account<Mock> =
        client.get_account_from_namespace(namespace.to_owned())?;

    assert_eq!(
        account.get_account_info()?,
        account_from_namespace.get_account_info()?
    );

    Ok(())
}

#[test]
fn can_create_publisher_without_optional_parameters() -> anyhow::Result<()> {
    // Set up.
    let (chain, _abstr) = deploy_abstract()?;

    let client: AbstractClient<Mock> = AbstractClient::new(chain.clone())?;

    let publisher: Publisher<Mock> = client.new_publisher().build()?;

    let account_info = publisher.account().get_account_info()?;
    assert_eq!(
        AccountInfo {
            name: String::from("Default Abstract Account"),
            chain_id: String::from("cosmos-testnet-14002"),
            description: None,
            governance_details: GovernanceDetails::Monarchy {
                monarch: chain.sender
            },
            link: None,
        },
        account_info
    );

    Ok(())
}

#[test]
fn can_create_publisher_with_optional_parameters() -> anyhow::Result<()> {
    // Set up.
    let (chain, abstr) = deploy_abstract()?;

    let asset = "asset";

    abstr.ans_host.execute(
        &abstract_core::ans_host::ExecuteMsg::UpdateAssetAddresses {
            to_add: vec![(asset.to_owned(), AssetInfo::native(asset).into())],
            to_remove: vec![],
        },
        None,
    )?;

    let client: AbstractClient<Mock> = AbstractClient::new(chain.clone())?;

    let name = "test-account";
    let description = "description";
    let link = "https://abstract.money";
    let governance_details = GovernanceDetails::Monarchy {
        monarch: String::from("monarch"),
    };
    let namespace = "test-namespace";
    let base_asset = AssetEntry::new(asset);
    let publisher: Publisher<Mock> = client
        .new_publisher()
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
    let account_id = abstr
        .version_control
        .namespace(Namespace::new(namespace)?)?
        .account_id;
    assert_eq!(account_id, AccountId::local(1));

    Ok(())
}

#[test]
fn can_get_publisher_from_namespace() -> anyhow::Result<()> {
    // Set up.
    let (chain, _abstr) = deploy_abstract()?;

    let client: AbstractClient<Mock> = AbstractClient::new(chain.clone())?;

    let namespace = "namespace";
    let publisher: Publisher<Mock> = client.new_publisher().namespace(namespace).build()?;

    let publisher_from_namespace: Publisher<Mock> =
        client.get_publisher_from_namespace(namespace.to_owned())?;

    assert_eq!(
        publisher.account().get_account_info()?,
        publisher_from_namespace.account().get_account_info()?
    );

    Ok(())
}

#[test]
fn can_publish_and_install_app() -> anyhow::Result<()> {
    // Set up.
    let (chain, _abstr) = deploy_abstract()?;

    let client: AbstractClient<Mock> = AbstractClient::new(chain)?;

    let publisher: Publisher<Mock> = client.new_publisher().namespace("my-namespace").build()?;

    publisher.deploy_module::<AppInterface<Mock>>()?;

    let my_app: Application<Mock, AppInterface<Mock>> = publisher
        .install_app::<AppInterface<Mock>, app::msg::AppInstantiateMsg>(
            &app::msg::AppInstantiateMsg {},
            &[],
        )?;

    my_app.call_as(&publisher.admin()?).update_config()?;

    let config = my_app.config()?;

    assert_eq!(ConfigResponse {}, config);
    Ok(())
}
