use abstract_client::{application::Application, client::AbstractClient};
use abstract_interface::Abstract;
use cosmwasm_std::Addr;
use cw_orch::{deploy::Deploy, prelude::Mock};

fn deploy_abstract() -> anyhow::Result<Mock> {
    let admin = Addr::unchecked("admin");
    let chain = Mock::new(&admin);
    Abstract::deploy_on(chain.clone(), admin.to_string())?;
    Ok(chain)
}

#[test]
fn test() -> anyhow::Result<()> {
    let chain = deploy_abstract()?;

    let client = AbstractClient::new(chain);

    //let my_app: Application<MyApp> = publisher.install_app<MyApp>();
    Ok(())
}
