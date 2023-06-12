#![allow(unused)]
pub mod mock_modules;

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
    Abstract, AccountFactory, AnsHost, Manager, ModuleFactory, Proxy, VCExecFns, VersionControl,
};
use abstract_interface::{AbstractAccount, AdapterDeployer};
use abstract_testing::prelude::{TEST_MODULE_NAME, TEST_NAMESPACE};
use cosmwasm_std::Addr;
use cw_orch::prelude::*;
use semver::Version;

pub(crate) type AResult = anyhow::Result<()>; // alias for Result<(), anyhow::Error>

pub(crate) fn create_default_account(
    factory: &AccountFactory<Mock>,
) -> anyhow::Result<AbstractAccount<Mock>> {
    let account = factory.create_default_account(GovernanceDetails::Monarchy {
        monarch: Addr::unchecked(OWNER).to_string(),
    })?;
    Ok(account)
}

use abstract_testing::addresses::{TEST_ACCOUNT_ID, TEST_MODULE_ID};

pub(crate) fn init_mock_adapter(
    chain: Mock,
    deployment: &Abstract<Mock>,
    version: Option<String>,
) -> anyhow::Result<BootMockAdapter<Mock>> {
    deployment
        .version_control
        .claim_namespaces(TEST_ACCOUNT_ID, vec!["tester".to_string()]);
    let mut staking_adapter = BootMockAdapter::new(TEST_MODULE_ID, chain);
    let version: Version = version
        .unwrap_or_else(|| CONTRACT_VERSION.to_string())
        .parse()?;
    staking_adapter.deploy(version, MockInitMsg)?;
    Ok(staking_adapter)
}

pub(crate) fn add_mock_adapter_install_fee(
    chain: Mock,
    deployment: &Abstract<Mock>,
    monetization: Monetization,
    version: Option<String>,
) -> anyhow::Result<()> {
    let version: Version = version
        .unwrap_or_else(|| CONTRACT_VERSION.to_string())
        .parse()?;
    deployment.version_control.set_module_monetization(
        TEST_MODULE_NAME.to_string(),
        monetization,
        Namespace::new(TEST_NAMESPACE).unwrap(),
    )?;
    Ok(())
}
