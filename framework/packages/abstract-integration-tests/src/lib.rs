//! # Testing Functions
//!
//! This module contains testing functions that can be used in different environments.

pub mod account_factory;
pub mod manager;
pub mod mock_modules;
// pub mod proxy;

use abstract_adapter::mock::{BootMockAdapter, MockInitMsg};
use abstract_core::objects::{
    module::{ModuleVersion, Monetization},
    namespace::Namespace,
    AccountId,
};
use abstract_interface::*;
use abstract_sdk::core::objects::gov_type::GovernanceDetails;
use abstract_testing::prelude::*;
use cw_orch::prelude::*;
pub type AResult = anyhow::Result<()>; // alias for Result<(), anyhow::Error>

pub fn create_default_account<T: CwEnv>(
    factory: &AccountFactory<T>,
) -> anyhow::Result<AbstractAccount<T>> {
    let sender = factory.as_instance().get_chain().sender();

    let account = factory.create_default_account(GovernanceDetails::Monarchy {
        monarch: sender.to_string(),
    })?;
    Ok(account)
}

pub fn install_module_version<T: CwEnv>(
    manager: &Manager<T>,
    module: &str,
    version: &str,
) -> anyhow::Result<String> {
    manager.install_module_version(
        module,
        ModuleVersion::Version(version.to_string()),
        Some(&MockInitMsg {}),
        None,
    )?;

    Ok(manager.module_info(module)?.unwrap().address.to_string())
}

pub fn init_mock_adapter<T: CwEnv>(
    chain: T,
    deployment: &Abstract<T>,
    version: Option<String>,
    account_id: AccountId,
) -> anyhow::Result<BootMockAdapter<T>> {
    deployment
        .version_control
        .claim_namespace(account_id, "tester".to_string())?;
    let mock_adapter = BootMockAdapter::new(TEST_MODULE_ID, chain);
    let version: semver::Version = version
        .unwrap_or_else(|| TEST_VERSION.to_string())
        .parse()?;
    BootMockAdapter::deploy(&mock_adapter, version, MockInitMsg {}, DeployStrategy::Try)?;
    Ok(mock_adapter)
}

pub fn add_mock_adapter_install_fee<T: CwEnv>(
    deployment: &Abstract<T>,
    monetization: Monetization,
    version: Option<String>,
) -> anyhow::Result<()> {
    let version = version.unwrap_or(TEST_VERSION.to_string());
    deployment.version_control.update_module_configuration(
        "test-module-id".to_string(),
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

pub fn install_adapter_with_funds<T: CwEnv>(
    manager: &Manager<T>,
    adapter_id: &str,
    funds: &[Coin],
) -> AResult {
    manager.install_module::<Empty>(adapter_id, None, Some(funds))?;
    Ok(())
}

pub fn install_adapter<T: CwEnv>(manager: &Manager<T>, adapter_id: &str) -> AResult {
    install_adapter_with_funds(manager, adapter_id, &[])
}
