mod common;

use common::init_test_env;
use abstract_boot::{os_factory::OsFactoryQueryFns, OsFactoryExecFns, VCQueryFns, OS, *};
use abstract_os::{objects::{gov_type::GovernanceDetails, module::ModuleInfo}, module_factory, version_control::Core};
use boot_core::{
    prelude::{instantiate_default_mock_env, ContractInstance},
    IndexResponse,
};
use cosmwasm_std::{Addr, Uint64};
use speculoos::prelude::*;

type AResult = anyhow::Result<()>; // alias for Result<(), anyhow::Error>

#[test]
fn instantiate() -> AResult {
    let sender = Addr::unchecked(common::ROOT_USER);
    let (_state, chain) = instantiate_default_mock_env(&sender)?;
    let (mut deployment, mut core) = init_test_env(&chain)?;
    deployment.deploy(&mut core)?;

    let factory = deployment.module_factory;
    let factory_config = factory.config()?;
    let expected = module_factory::ConfigResponse {
        owner: sender.clone().into_string(),
        ans_host_address: deployment.ans_host.address()?.into(),
        version_control_address: deployment.version_control.address()?.into_string(),
    };

    assert_that!(&factory_config).is_equal_to(&expected);
    Ok(())
}

#[test]
fn caller_must_be_manager () -> AResult {
    let _not_owner = Addr::unchecked("not_owner");
    let sender = Addr::unchecked(common::ROOT_USER);
    let (_, chain) = instantiate_default_mock_env(&sender)?;
    let (mut deployment, mut core) = init_test_env(&chain)?;
    deployment.deploy(&mut core)?;

    let factory = &deployment.module_factory;
    let test_module = ModuleInfo::from_id("publisher:test", abstract_os::objects::module::ModuleVersion::Latest {  })?;

    let res = factory.install_module(test_module, None).unwrap_err();
    assert_that(&res.to_string()).contains("Must be as OS manager");

    Ok(())
}