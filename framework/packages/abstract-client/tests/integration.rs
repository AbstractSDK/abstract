mod app;
use abstract_client::{application::Application, client::AbstractClient, publisher::Publisher};
use abstract_core::objects::{gov_type::GovernanceDetails, AccountId};
use abstract_interface::{Abstract, AccountDetails, VCExecFns};
use app::{contract::APP_VERSION, AppInterface, AppQueryMsgFns};
use cosmwasm_std::Addr;
use cw_orch::{deploy::Deploy, prelude::Mock};

use crate::app::msg::ConfigResponse;

const ADMIN: &str = "admin";

fn deploy_abstract() -> anyhow::Result<(Mock, Abstract<Mock>)> {
    let admin = Addr::unchecked(ADMIN);
    let chain = Mock::new(&admin);
    let abstr = Abstract::deploy_on(chain.clone(), admin.to_string())?;
    Ok((chain, abstr))
}

fn create_account(
    monarch: String,
    namespace: String,
    abstr: &Abstract<Mock>,
) -> anyhow::Result<()> {
    abstr.account_factory.create_new_account(
        AccountDetails {
            name: String::from("test-account"),
            description: None,
            link: None,
            namespace: Some(namespace),
            base_asset: None,
            install_modules: vec![],
        },
        GovernanceDetails::Monarchy { monarch },
        &[],
    )?;
    Ok(())
}

// TODO:
// Figure out flow for creating account + claiming namespace (have different APIs or default
// behaviour when namespace is not claimed?)
// Add builders for nicer interaction.
// Allow using account-id instead of namespace to get publisher in the case where namespace is not
// claimed.
// Handle errors gracefully.
// Handle module dependencies.

#[test]
fn test() -> anyhow::Result<()> {
    // Set up.
    let (chain, abstr) = deploy_abstract()?;
    let account = abstr
        .account_factory
        .create_default_account(GovernanceDetails::Monarchy {
            monarch: ADMIN.to_string(),
        })?;
    // claim the namespace so app can be deployed
    abstr
        .version_control
        .claim_namespace(AccountId::local(1), "my-namespace".to_string())?;

    // Interaction with client begins.
    let client: AbstractClient<Mock> = AbstractClient::new(chain);

    // TODO: Also try with namespace that does not exist.
    let publisher: Publisher<Mock> = client.new_publisher(String::from("my-namespace"));

    publisher.deploy_module::<AppInterface<Mock>>(APP_VERSION.parse().unwrap());

    let my_app: Application<Mock, AppInterface<Mock>> = publisher
        .install_app::<AppInterface<Mock>, app::msg::AppInstantiateMsg>(
            &app::msg::AppInstantiateMsg {},
            &[],
        )
        .unwrap();
    let config = my_app.config()?;

    assert_eq!(ConfigResponse {}, config);
    Ok(())
}
