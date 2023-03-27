use abstract_boot::{
    Abstract, AbstractAccount, AccountFactory, AnsHost, Manager, ModuleFactory, Proxy,
    VersionControl,
};
use abstract_core::{ACCOUNT_FACTORY, ANS_HOST, MANAGER, MODULE_FACTORY, PROXY, VERSION_CONTROL};
use boot_core::ContractWrapper;
use boot_core::{ContractInstance, Mock};

pub const OWNER: &str = "owner";

pub fn init_test_env<'a>(chain: Mock) -> anyhow::Result<(Abstract<Mock>, AbstractAccount<Mock>)> {
    let mut ans_host = AnsHost::new(ANS_HOST, chain.clone());
    let mut account_factory = AccountFactory::new(ACCOUNT_FACTORY, chain.clone());
    let mut version_control = VersionControl::new(VERSION_CONTROL, chain.clone());
    let mut module_factory = ModuleFactory::new(MODULE_FACTORY, chain.clone());
    let mut manager = Manager::new(MANAGER, chain.clone());
    let mut proxy = Proxy::new(PROXY, chain.clone());

    ans_host
        .as_instance_mut()
        .set_mock(Box::new(ContractWrapper::new_with_empty(
            ::ans_host::contract::execute,
            ::ans_host::contract::instantiate,
            ::ans_host::contract::query,
        )));

    account_factory.as_instance_mut().set_mock(Box::new(
        ContractWrapper::new_with_empty(
            ::account_factory::contract::execute,
            ::account_factory::contract::instantiate,
            ::account_factory::contract::query,
        )
        .with_reply_empty(::account_factory::contract::reply),
    ));

    module_factory.as_instance_mut().set_mock(Box::new(
        boot_core::ContractWrapper::new_with_empty(
            ::abstract_module_factory::contract::execute,
            ::abstract_module_factory::contract::instantiate,
            ::abstract_module_factory::contract::query,
        )
        .with_reply_empty(::abstract_module_factory::contract::reply),
    ));

    version_control.as_instance_mut().set_mock(Box::new(
        boot_core::ContractWrapper::new_with_empty(
            ::version_control::contract::execute,
            ::version_control::contract::instantiate,
            ::version_control::contract::query,
        ),
    ));

    manager
        .as_instance_mut()
        .set_mock(Box::new(boot_core::ContractWrapper::new_with_empty(
            ::manager::contract::execute,
            ::manager::contract::instantiate,
            ::manager::contract::query,
        )));

    proxy
        .as_instance_mut()
        .set_mock(Box::new(boot_core::ContractWrapper::new_with_empty(
            ::proxy::contract::execute,
            ::proxy::contract::instantiate,
            ::proxy::contract::query,
        )));

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
