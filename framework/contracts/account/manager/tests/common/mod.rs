#![allow(unused)]

use abstract_core::objects::module::{ModuleInfo, ModuleVersion, Monetization};
use abstract_core::objects::namespace::Namespace;

use abstract_adapter::mock::{BootMockAdapter, MockInitMsg};
use abstract_core::version_control::AccountBase;
use abstract_core::{objects::gov_type::GovernanceDetails, PROXY};
use abstract_core::{ACCOUNT_FACTORY, ANS_HOST, MANAGER, MODULE_FACTORY, VERSION_CONTROL};
use abstract_interface::{
    Abstract, AccountFactory, AnsHost, DeployStrategy, Manager, ManagerExecFns, ModuleFactory,
    Proxy, VCExecFns, VersionControl,
};
use abstract_interface::{AbstractAccount, AdapterDeployer};
use abstract_manager::contract::CONTRACT_VERSION;
use abstract_testing::prelude::*;
use cosmwasm_std::Addr;
use cw_orch::prelude::*;
use semver::Version;

pub use abstract_integration_tests::{create_default_account, mock_modules, AResult};
use abstract_testing::addresses::{TEST_ACCOUNT_ID, TEST_MODULE_ID};

pub fn uninstall_module(manager: &Manager<Mock>, module_id: &str) -> AResult {
    manager
        .uninstall_module(module_id.to_string())
        .map_err(Into::<CwOrchError>::into)?;
    Ok(())
}
