use abstract_core::{app::BaseInstantiateMsg, objects::gov_type::GovernanceDetails};
use cw_orch::{
    anyhow,
    deploy::Deploy,
    prelude::{networks::parse_network, ContractInstance, DaemonBuilder, TxHandler},
    tokio::runtime::Runtime,
};

use abstract_interface::*;
use croncat_app::{
    contract::{interface::CroncatApp, CRONCAT_ID},
    msg::AppInstantiateMsg,
};
use dotenv::dotenv;
use semver::Version;

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> anyhow::Result<()> {
    dotenv().ok();
    env_logger::init();
    let chain = parse_network("uni-6");
    let version: Version = CONTRACT_VERSION.parse().unwrap();
    let rt = Runtime::new()?;
    let chain = DaemonBuilder::default()
        .chain(chain)
        .handle(rt.handle())
        .build()?;
    let app = CroncatApp::new(CRONCAT_ID, chain.clone());

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
    abstract_deployment.version_control.claim_namespaces(
        account_config.account_id.u64() as u32,
        vec!["croncat".to_owned()],
    )?;

    // Deploy
    app.deploy(version)?;

    // Install app
    account.install_module(
        CRONCAT_ID,
        &croncat_app::msg::InstantiateMsg {
            base: BaseInstantiateMsg {
                ans_host_address: abstract_deployment.ans_host.addr_str()?,
            },
            module: AppInstantiateMsg {},
        },
        None,
    )?;
    Ok(())
}
