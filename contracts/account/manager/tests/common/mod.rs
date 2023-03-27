#![allow(unused)]
pub mod mock_modules;

pub const OWNER: &str = "owner";
pub const TEST_COIN: &str = "ucoin";

use ::abstract_manager::contract::CONTRACT_VERSION;
use abstract_api::mock::{BootMockApi, MockInitMsg};
use abstract_boot::{
    Abstract, AccountFactory, AnsHost, Manager, ModuleFactory, Proxy, VersionControl,
};
use abstract_boot::{AbstractAccount, ApiDeployer};
use abstract_core::{objects::gov_type::GovernanceDetails, PROXY};
use abstract_core::{ACCOUNT_FACTORY, ANS_HOST, MANAGER, MODULE_FACTORY, VERSION_CONTROL};
use boot_core::ContractWrapper;
use boot_core::{ContractInstance, Mock};
use cosmwasm_std::Addr;
use semver::Version;

pub fn init_abstract_env(chain: Mock) -> anyhow::Result<(Abstract<Mock>, AbstractAccount<Mock>)> {
    let mut ans_host = AnsHost::new(ANS_HOST, chain.clone());
    let mut account_factory = AccountFactory::new(ACCOUNT_FACTORY, chain.clone());
    let mut version_control = VersionControl::new(VERSION_CONTROL, chain.clone());
    let mut module_factory = ModuleFactory::new(MODULE_FACTORY, chain.clone());
    let mut manager = Manager::new(MANAGER, chain.clone());
    let mut proxy = Proxy::new(PROXY, chain.clone());

    ans_host.as_instance_mut().set_mock(Box::new(
        ContractWrapper::new_with_empty(
            ::ans_host::contract::execute,
            ::ans_host::contract::instantiate,
            ::ans_host::contract::query,
        )
        .with_migrate_empty(::ans_host::contract::migrate),
    ));

    account_factory.as_instance_mut().set_mock(Box::new(
        ContractWrapper::new_with_empty(
            ::account_factory::contract::execute,
            ::account_factory::contract::instantiate,
            ::account_factory::contract::query,
        )
        .with_migrate_empty(::account_factory::contract::migrate)
        .with_reply_empty(::account_factory::contract::reply),
    ));

    module_factory.as_instance_mut().set_mock(Box::new(
        boot_core::ContractWrapper::new_with_empty(
            ::module_factory::contract::execute,
            ::module_factory::contract::instantiate,
            ::module_factory::contract::query,
        )
        .with_migrate_empty(::module_factory::contract::migrate)
        .with_reply_empty(::module_factory::contract::reply),
    ));

    version_control.as_instance_mut().set_mock(Box::new(
        boot_core::ContractWrapper::new_with_empty(
            ::version_control::contract::execute,
            ::version_control::contract::instantiate,
            ::version_control::contract::query,
        )
        .with_migrate_empty(::version_control::contract::migrate),
    ));

    manager.as_instance_mut().set_mock(Box::new(
        boot_core::ContractWrapper::new_with_empty(
            ::abstract_manager::contract::execute,
            ::abstract_manager::contract::instantiate,
            ::abstract_manager::contract::query,
        )
        .with_migrate_empty(::abstract_manager::contract::migrate),
    ));

    proxy.as_instance_mut().set_mock(Box::new(
        boot_core::ContractWrapper::new_with_empty(
            ::proxy::contract::execute,
            ::proxy::contract::instantiate,
            ::proxy::contract::query,
        )
        .with_migrate_empty(::proxy::contract::migrate),
    ));

    // do as above for the rest of the contracts

    let deployment = Abstract {
        chain,
        version: "1.0.0".parse()?,
        ans_host,
        account_factory,
        version_control,
        module_factory,
    };

    let account = AbstractAccount { manager, proxy };

    Ok((deployment, account))
}

pub(crate) type AResult = anyhow::Result<()>; // alias for Result<(), anyhow::Error>

pub(crate) fn create_default_account(
    factory: &AccountFactory<Mock>,
) -> anyhow::Result<AbstractAccount<Mock>> {
    let account = factory.create_default_account(GovernanceDetails::Monarchy {
        monarch: Addr::unchecked(OWNER).to_string(),
    })?;
    Ok(account)
}

use abstract_testing::addresses::TEST_MODULE_ID;

pub(crate) fn init_mock_api(
    chain: Mock,
    _deployment: &Abstract<Mock>,
    version: Option<String>,
) -> anyhow::Result<BootMockApi<Mock>> {
    let mut staking_api = BootMockApi::new(TEST_MODULE_ID, chain);
    let version: Version = version
        .unwrap_or_else(|| CONTRACT_VERSION.to_string())
        .parse()?;
    staking_api.deploy(version, MockInitMsg)?;
    Ok(staking_api)
}
