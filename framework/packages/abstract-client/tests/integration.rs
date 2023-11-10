mod app;
use abstract_client::{
    account::Account, application::Application, client::AbstractClient, publisher::Publisher,
};
use abstract_core::objects::gov_type::GovernanceDetails;
use abstract_interface::Abstract;
use app::{AppInterface, AppQueryMsgFns};
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

// TODO:
// Allow using account-id instead of namespace to get publisher in the case where namespace is not
// claimed.
// Handle module dependencies.

#[test]
fn test() -> anyhow::Result<()> {
    // Set up.
    let (chain, _abstr) = deploy_abstract()?;

    // Interaction with client begins.
    let client: AbstractClient<Mock> = AbstractClient::new(chain)?;

    let publisher: Publisher<Mock> = client
        .new_publisher()
        .name("test-account")
        .namespace("my-namespace")
        .governance_details(GovernanceDetails::Monarchy {
            monarch: ADMIN.to_string(),
        })
        .build()?;

    publisher.deploy_module::<AppInterface<Mock>>()?;

    let my_app: Application<Mock, AppInterface<Mock>> = publisher
        .install_app::<AppInterface<Mock>, app::msg::AppInstantiateMsg>(
            &app::msg::AppInstantiateMsg {},
            &[],
        )?;

    let config = my_app.config()?;

    assert_eq!(ConfigResponse {}, config);

    let _account: Account<Mock> = client.new_account().build()?;
    Ok(())
}
