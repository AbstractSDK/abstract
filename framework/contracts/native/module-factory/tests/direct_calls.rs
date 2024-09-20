use abstract_interface::*;
use abstract_std::{
    module_factory, module_factory::FactoryModuleInstallConfig, objects::module::ModuleInfo,
};
use abstract_testing::prelude::mock_bech32_sender;
use cosmwasm_std::Binary;
use cw_orch::prelude::*;
use speculoos::prelude::*;

type AResult = anyhow::Result<()>; // alias for Result<(), anyhow::Error>

#[test]
fn instantiate() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on(chain.clone(), mock_bech32_sender(&chain))?;

    let factory = deployment.module_factory;
    let factory_config = factory.config()?;
    let expected = module_factory::ConfigResponse {
        ans_host_address: deployment.ans_host.address()?,
        version_control_address: deployment.version_control.address()?,
    };

    assert_that!(&factory_config).is_equal_to(&expected);
    Ok(())
}

#[test]
fn caller_must_be_manager() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let _not_owner = chain.addr_make("not_owner");
    let deployment = Abstract::deploy_on(chain.clone(), mock_bech32_sender(&chain))?;

    let factory = &deployment.module_factory;
    let test_module = ModuleInfo::from_id(
        "publisher:test",
        abstract_std::objects::module::ModuleVersion::Latest,
    )?;

    let res = factory
        .install_modules(
            vec![FactoryModuleInstallConfig::new(test_module, None)],
            Binary::default(),
        )
        .unwrap_err();
    assert_that(&res.root().to_string())
        .contains("ensure that the contract is a Manager or Proxy contract");

    Ok(())
}
