#![allow(unused)]

use abstract_core::objects::module::{ModuleInfo, ModuleVersion, Monetization};
use abstract_core::objects::namespace::Namespace;
pub use abstract_testing::addresses::TEST_OWNER;

pub const OWNER: &str = TEST_OWNER;
pub const TEST_COIN: &str = "ucoin";

use ::abstract_manager::contract::CONTRACT_VERSION;
use abstract_adapter::mock::{BootMockAdapter, MockInitMsg};
use abstract_core::version_control::AccountBase;
use abstract_core::{objects::gov_type::GovernanceDetails, PROXY};
use abstract_core::{ACCOUNT_FACTORY, ANS_HOST, MANAGER, MODULE_FACTORY, VERSION_CONTROL};
use abstract_interface::{
    Abstract, AccountFactory, AnsHost, DeployStrategy, Manager, ManagerExecFns, ModuleFactory,
    Proxy, VCExecFns, VersionControl,
};
use abstract_interface::{AbstractAccount, AdapterDeployer};
use abstract_testing::prelude::{TEST_MODULE_NAME, TEST_NAMESPACE};
use cosmwasm_std::Addr;
use cw_orch::prelude::*;
use semver::Version;

pub use abstract_integration_tests::{create_default_account, mock_modules, AResult};
use abstract_testing::addresses::{TEST_ACCOUNT_ID, TEST_MODULE_ID};

pub(crate) fn init_mock_adapter(
    chain: Mock,
    deployment: &Abstract<Mock>,
    version: Option<String>,
) -> anyhow::Result<BootMockAdapter<Mock>> {
    deployment
        .version_control
        .claim_namespace(TEST_ACCOUNT_ID, "tester".to_string());
    let mut staking_adapter = BootMockAdapter::new(TEST_MODULE_ID, chain);
    let version: Version = version
        .unwrap_or_else(|| CONTRACT_VERSION.to_string())
        .parse()?;
    staking_adapter.deploy(version, MockInitMsg, DeployStrategy::Try)?;
    Ok(staking_adapter)
}

pub(crate) fn add_mock_adapter_install_fee(
    chain: Mock,
    deployment: &Abstract<Mock>,
    monetization: Monetization,
    version: Option<String>,
) -> anyhow::Result<()> {
    let version = version.unwrap_or(CONTRACT_VERSION.to_string());
    deployment.version_control.update_module_configuration(
        TEST_MODULE_NAME.to_string(),
        Namespace::new(TEST_NAMESPACE).unwrap(),
        abstract_core::version_control::UpdateModule::Versioned {
            version,
            metadata: None,
            monetization: Some(monetization),
            instantiation_funds: None,
        },
    )?;
    Ok(())
}

pub fn install_adapter(manager: &Manager<Mock>, adapter_id: &str) -> AResult {
    manager.install_module::<Empty>(adapter_id, None, None)?;
    Ok(())
}

pub fn install_adapter_with_funds(
    manager: &Manager<Mock>,
    adapter_id: &str,
    funds: &[Coin],
) -> AResult {
    manager.install_module::<Empty>(adapter_id, None, Some(funds))?;
    Ok(())
}

pub fn uninstall_module(manager: &Manager<Mock>, module_id: &str) -> AResult {
    manager
        .uninstall_module(module_id.to_string())
        .map_err(Into::<CwOrchError>::into)?;
    Ok(())
}
