//! # Testing Functions
//!
//! This module contains testing functions that can be used in different environments.

pub mod account_factory;
pub mod account;
pub mod mock_modules;
// pub mod proxy;

use abstract_adapter::mock::{interface::MockAdapterI, MockInitMsg};
use abstract_interface::*;
use abstract_sdk::std::objects::gov_type::GovernanceDetails;
use abstract_std::objects::{
    module::{ModuleVersion, Monetization},
    namespace::Namespace,
    AccountId,
};
use abstract_testing::prelude::*;
use cw_orch::prelude::*;
pub type AResult = anyhow::Result<()>; // alias for Result<(), anyhow::Error>

pub fn create_default_account<T: CwEnv>(
    factory: &AccountFactory<T>,
) -> anyhow::Result<AccountI<T>> {
    let sender = factory.as_instance().environment().sender_addr();

    let account = factory.create_default_account(GovernanceDetails::Monarchy {
        monarch: sender.to_string(),
    })?;
    Ok(account)
}

pub fn install_module_version<T: CwEnv>(
    account: &AccountI<T>,
    module: &str,
    version: &str,
) -> anyhow::Result<String> {
    account.install_module_version(
        module,
        ModuleVersion::Version(version.to_string()),
        Some(&MockInitMsg {}),
        &[],
    )?;

    Ok(account.module_info(module)?.unwrap().address.to_string())
}

pub fn init_mock_adapter<T: CwEnv>(
    chain: T,
    deployment: &Abstract<T>,
    version: Option<String>,
    account_id: AccountId,
) -> anyhow::Result<MockAdapterI<T>> {
    deployment
        .version_control
        .claim_namespace(account_id, "tester".to_string())?;
    let mock_adapter = MockAdapterI::new(TEST_MODULE_ID, chain);
    let version: semver::Version = version
        .unwrap_or_else(|| TEST_VERSION.to_string())
        .parse()?;
    MockAdapterI::deploy(&mock_adapter, version, MockInitMsg {}, DeployStrategy::Try)?;
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
        abstract_std::version_control::UpdateModule::Versioned {
            version,
            metadata: None,
            monetization: Some(monetization),
            instantiation_funds: None,
        },
    )?;
    Ok(())
}

pub fn install_adapter_with_funds<T: CwEnv>(
    account: &AccountI<T>,
    adapter_id: &str,
    funds: &[Coin],
) -> AResult {
    account.install_module::<Empty>(adapter_id, None, funds)?;
    Ok(())
}

pub fn install_adapter<T: CwEnv>(account: &AccountI<T>, adapter_id: &str) -> AResult {
    install_adapter_with_funds(account, adapter_id, &[])
}
