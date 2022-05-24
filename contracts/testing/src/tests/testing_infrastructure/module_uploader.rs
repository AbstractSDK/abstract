use crate::tests::common::DEFAULT_VERSION;
use abstract_os::core::modules::ModuleInfo;
use abstract_os::native::version_control::msg as VCMsg;
use anyhow::Result as AnyResult;
use cosmwasm_std::{Addr, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response};

use terra_cosmwasm::TerraMsgWrapper;
use terra_multi_test::{Contract, Executor, TerraApp};

pub fn register_module(
    app: &mut TerraApp,
    sender: &Addr,
    version_control: &Addr,
    module: ModuleInfo,
    contract: Box<dyn Contract<TerraMsgWrapper>>,
) -> AnyResult<()> {
    let code_id = app.store_code(contract);
    let msg = VCMsg::ExecuteMsg::AddCodeId {
        module: module.name,
        version: module.version.unwrap_or(DEFAULT_VERSION.to_string()),
        code_id,
    };
    app.execute_contract(sender.clone(), version_control.clone(), &msg, &[])?;
    Ok(())
}
