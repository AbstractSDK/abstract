use anyhow::Result as AnyResult;
use cosmwasm_std::Addr;
use pandora_os::{core::modules::ModuleInfo, registery::SUBSCRIPTION};
use terra_multi_test::{ContractWrapper, TerraApp};

use super::testing_infrastructure::module_uploader::register_module;

pub fn register_subscription(
    app: &mut TerraApp,
    sender: &Addr,
    version_control: &Addr,
) -> AnyResult<()> {
    let module = ModuleInfo {
        name: SUBSCRIPTION.into(),
        version: None,
    };

    let contract = Box::new(
        ContractWrapper::new_with_empty(
            subscription::contract::execute,
            subscription::contract::instantiate,
            subscription::contract::query,
        )
        .with_migrate_empty(subscription::contract::migrate),
    );
    register_module(app, &sender, &version_control, module, contract).unwrap();
    Ok(())
}
