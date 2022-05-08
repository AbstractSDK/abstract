use crate::tests::common::DEFAULT_VERSION;
use anyhow::Result as AnyResult;
use cosmwasm_std::{Addr, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response};
use pandora_os::core::modules::ModuleInfo;
use pandora_os::native::version_control::msg as VCMsg;

use terra_cosmwasm::TerraMsgWrapper;
use terra_multi_test::{Contract, Executor, TerraApp};

type ContractFn<T, C, E> =
    fn(deps: DepsMut, env: Env, info: MessageInfo, msg: T) -> Result<Response<C>, E>;
type PermissionedFn<T, C, E> = fn(deps: DepsMut, env: Env, msg: T) -> Result<Response<C>, E>;
type ReplyFn<C, E> = fn(deps: DepsMut, env: Env, msg: Reply) -> Result<Response<C>, E>;
type QueryFn<T, E> = fn(deps: Deps, env: Env, msg: T) -> Result<Binary, E>;

type ContractClosure<T, C, E> = Box<dyn Fn(DepsMut, Env, MessageInfo, T) -> Result<Response<C>, E>>;
type PermissionedClosure<T, C, E> = Box<dyn Fn(DepsMut, Env, T) -> Result<Response<C>, E>>;
type ReplyClosure<C, E> = Box<dyn Fn(DepsMut, Env, Reply) -> Result<Response<C>, E>>;
type QueryClosure<T, E> = Box<dyn Fn(Deps, Env, T) -> Result<Binary, E>>;

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
