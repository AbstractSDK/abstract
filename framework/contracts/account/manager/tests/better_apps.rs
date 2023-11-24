mod common;
use abstract_core::objects::account::TEST_ACCOUNT_ID;
use abstract_interface::*;

use common::{create_default_account, AResult};
use cosmwasm_std::Addr;
use cw_orch::deploy::Deploy;
use cw_orch::prelude::*;
use mock_app::MockApp;
use speculoos::prelude::*;

pub mod mock_app {
    pub const APP_ID: &str = "tester:app";
    pub const APP_VERSION: &str = "1.0.0";
    abstract_app::gen_app_better_mock!(MockApp, APP_ID, APP_VERSION, &[]);
}
use crate::mock_app::APP_ID;
use crate::mock_app::APP_VERSION;

#[test]
fn account_install_app() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;

    deployment
        .version_control
        .claim_namespace(TEST_ACCOUNT_ID, "tester".to_owned())?;

    let app = MockApp::new_test(chain);
    app.deploy(APP_VERSION.parse().unwrap(), DeployStrategy::Try)?;
    let app_addr = account.install_app(app, &crate::mock_app::sv::ImplInstantiateMsg {}, None)?;
    let module_addr = account.manager.module_info(APP_ID)?.unwrap().address;
    assert_that!(app_addr).is_equal_to(module_addr);
    Ok(())
}
