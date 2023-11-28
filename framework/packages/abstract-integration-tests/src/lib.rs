//! # Testing Functions
//!
//! This module contains testing functions that can be used in different environments.

pub mod manager;
pub mod mock_modules;
// pub mod proxy;
// pub mod account_factory;

use abstract_adapter::mock::MockInitMsg;
use abstract_core::objects::module::ModuleVersion;
use abstract_interface::*;
use abstract_sdk::core::objects::gov_type::GovernanceDetails;
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
        Some(&MockInitMsg),
        None,
    )?;

    Ok(manager.module_info(module)?.unwrap().address.to_string())
}
