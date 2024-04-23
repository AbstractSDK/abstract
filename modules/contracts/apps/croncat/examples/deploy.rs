use abstract_app::std::objects::gov_type::GovernanceDetails;
use abstract_interface::*;
use croncat_app::{
    contract::{interface::Croncat, CRONCAT_ID},
    msg::AppInstantiateMsg,
};
use cw_orch::{
    anyhow,
    prelude::{networks::parse_network, DaemonBuilder, Deploy, TxHandler},
    tokio::runtime::Runtime,
};
use dotenv::dotenv;
use semver::Version;

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> anyhow::Result<()> {
    dotenv().ok();
    env_logger::init();
    let chain = parse_network("uni-6").unwrap();
    let version: Version = CONTRACT_VERSION.parse().unwrap();
    let rt = Runtime::new()?;
    let chain = DaemonBuilder::default()
        .chain(chain)
        .handle(rt.handle())
        .build()?;
    let app = Croncat::new(CRONCAT_ID, chain.clone());

    // Create account
    let abstract_deployment = Abstract::load_from(chain.clone())?;
    let account = abstract_deployment.account_factory.create_default_account(
        GovernanceDetails::Monarchy {
            monarch: chain.sender().into_string(),
        },
    )?;

    // In case account already created

    // let account_base = abstract_deployment.version_control.get_account(7)?;
    // let account = AbstractAccount::new(chain.clone(), None);
    // account.manager.set_address(&account_base.manager);
    // account.proxy.set_address(&account_base.proxy);

    // Claim namespace
    let account_config = account.manager.config()?;
    abstract_deployment
        .version_control
        .claim_namespace(account_config.account_id, "croncat".to_owned())?;

    // Deploy
    app.deploy(version, DeployStrategy::Try)?;

    // Install app
    account.install_app(&app, &AppInstantiateMsg {}, None)?;
    Ok(())
}
