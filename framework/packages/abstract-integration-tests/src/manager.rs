use abstract_app::gen_app_mock;
use abstract_core::objects::account::TEST_ACCOUNT_ID;
use abstract_core::PROXY;
use abstract_interface::*;

use cosmwasm_std::{coin, Addr, Coin, CosmosMsg};
use cw_orch::deploy::Deploy;
use cw_orch::prelude::*;
use speculoos::prelude::*;
use crate::AResult;

const APP_ID: &str = "tester:app";
const APP_VERSION: &str = "1.0.0";
gen_app_mock!(MockApp, APP_ID, APP_VERSION, &[]);

/// Test installing an app on an account
pub fn account_install_app<T: CwEnv>(chain: T, sender: Addr) -> AResult {
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = crate::create_default_account(&deployment.account_factory)?;

    deployment
        .version_control
        .claim_namespace(TEST_ACCOUNT_ID, "tester".to_owned())?;

    let app = MockApp::new_test(chain.clone());
    MockApp::deploy(&app, APP_VERSION.parse().unwrap(), DeployStrategy::Try)?;
    let app_addr = account.install_app(&app, &MockInitMsg, None)?;
    let module_addr = account.manager.module_info(APP_ID)?.unwrap().address;
    assert_that!(app_addr).is_equal_to(module_addr);
    Ok(())
}