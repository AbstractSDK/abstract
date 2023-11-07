mod app;
use abstract_client::{application::Application, client::AbstractClient, publisher::Publisher};
use abstract_core::objects::{gov_type::GovernanceDetails, AccountId};
use abstract_interface::{Abstract, AccountDetails, AppDeployer, DeployStrategy, VCExecFns};
use app::{
    contract::{APP_ID, APP_VERSION},
    AppInterface, AppQueryMsgFns,
};
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

fn deploy_app(abstr: &Abstract<Mock>, chain: Mock) -> anyhow::Result<()> {
    let account = abstr
        .account_factory
        .create_default_account(GovernanceDetails::Monarchy {
            monarch: ADMIN.to_string(),
        })?;
    // claim the namespace so app can be deployed
    abstr
        .version_control
        .claim_namespace(AccountId::local(1), "my-namespace".to_string())?;
    let app = AppInterface::new(APP_ID, chain);
    app.deploy(APP_VERSION.parse()?, DeployStrategy::Try)?;
    Ok(())
}

#[test]
fn test() -> anyhow::Result<()> {
    // Set up.
    let (chain, abstr) = deploy_abstract()?;
    //let namespace = "namespace";
    //let user = "user";
    deploy_app(&abstr, chain.clone())?;
    //create_account(user.to_owned(), namespace.to_owned(), &abstr)?;

    // Interaction with client begins.
    let client: AbstractClient<Mock> = AbstractClient::new(chain);

    // TODO: Also try with namespace that does not exist.
    let publisher: Publisher<Mock> = client.new_publisher(String::from("my-namespace"));

    let my_app: Application<Mock, AppInterface<Mock>> = publisher
        .account()
        .install_app::<AppInterface<Mock>, app::msg::AppInstantiateMsg>(
            &app::msg::AppInstantiateMsg {},
            &[],
        )
        .unwrap();
    let config = my_app.config()?;

    assert_eq!(ConfigResponse {}, config);
    Ok(())
}
