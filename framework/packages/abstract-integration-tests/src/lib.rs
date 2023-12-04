//! # Testing Functions
//!
//! This module contains testing functions that can be used in different environments.

pub mod account_factory;
pub mod manager;
pub mod mock_modules;
// pub mod proxy;

use abstract_adapter::mock::MockInitMsg;
use abstract_core::objects::{
    account::TEST_ACCOUNT_ID,
    module::{ModuleVersion, Monetization},
    namespace::Namespace,
};
use abstract_interface::*;
use abstract_sdk::core::objects::gov_type::GovernanceDetails;
use abstract_testing::addresses::TEST_NAMESPACE;
use cw_orch::prelude::*;
use mock_modules::{adapter_1::BootMockAdapter1V1, V1};
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
        Some(&MockInitMsg),
        None,
    )?;

    Ok(manager.module_info(module)?.unwrap().address.to_string())
}

pub fn init_mock_adapter<T: CwEnv>(
    chain: T,
    deployment: &Abstract<T>,
    version: Option<String>,
) -> anyhow::Result<BootMockAdapter1V1<T>> {
    deployment
        .version_control
        .claim_namespace(TEST_ACCOUNT_ID, "tester".to_string())?;
    let staking_adapter = BootMockAdapter1V1::new_test(chain);
    let version: semver::Version = version.unwrap_or_else(|| V1.to_string()).parse()?;
    BootMockAdapter1V1::deploy(&staking_adapter, version, MockInitMsg, DeployStrategy::Try)?;
    Ok(staking_adapter)
}

pub fn add_mock_adapter_install_fee<T: CwEnv>(
    deployment: &Abstract<T>,
    monetization: Monetization,
    version: Option<String>,
) -> anyhow::Result<()> {
    let version = version.unwrap_or(V1.to_string());
    deployment.version_control.update_module_configuration(
        "mock-adapter1".to_string(),
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
