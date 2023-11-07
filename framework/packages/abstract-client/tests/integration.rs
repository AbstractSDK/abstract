use abstract_client::{application::Application, client::AbstractClient, publisher::Publisher};
use abstract_core::objects::gov_type::GovernanceDetails;
use abstract_interface::{Abstract, AccountDetails};
use cosmwasm_std::Addr;
use cw_orch::{deploy::Deploy, prelude::Mock};

fn deploy_abstract() -> anyhow::Result<(Mock, Abstract<Mock>)> {
    let admin = Addr::unchecked("admin");
    let chain = Mock::new(&admin);
    let abstr = Abstract::deploy_on(chain.clone(), admin.to_string())?;
    Ok((chain, abstr))
}

fn create_account(monarch: String, namespace: String, abstr: Abstract<Mock>) -> anyhow::Result<()> {
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

#[test]
fn test() -> anyhow::Result<()> {
    // Set up.
    let (chain, abstr) = deploy_abstract()?;
    let namespace = "namespace";
    let user = "user";
    create_account(user.to_owned(), namespace.to_owned(), abstr)?;

    // Interaction with client begins.
    let client: AbstractClient<Mock> = AbstractClient::new(chain);

    // TODO: Also try with namespace that does not exist.
    let publisher: Publisher<Mock> = client.new_publisher(namespace.to_owned());

    //let my_app: Application<MyApp> = publisher.install_app<MyApp>();
    Ok(())
}
