use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    Uint64,
};
use cw2::set_contract_version;

use crate::commands::*;
use crate::error::ManagerError;
use crate::queries;
use crate::state::{Config, ADMIN, CONFIG, OS_ID, ROOT};
use pandora_os::core::manager::msg::{ConfigQueryResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use pandora_os::registery::MANAGER;

pub type ManagerResult = Result<Response, ManagerError>;

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ManagerResult {
    set_contract_version(deps.storage, MANAGER, CONTRACT_VERSION)?;

    OS_ID.save(deps.storage, &msg.os_id)?;
    CONFIG.save(
        deps.storage,
        &Config {
            version_control_address: deps.api.addr_validate(&msg.version_control_address)?,
            module_factory_address: deps.api.addr_validate(&msg.module_factory_address)?,
        },
    )?;
    // Set root
    let root = deps.api.addr_validate(&msg.root_user)?;
    ROOT.set(deps.branch(), Some(root))?;
    // Setup the admin as the creator of the contract
    ADMIN.set(deps, Some(info.sender))?;
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> ManagerResult {
    match msg {
        ExecuteMsg::SetAdmin { admin } => set_admin(deps, info, admin),
        ExecuteMsg::UpdateConfig { vc_addr, root } => {
            execute_update_config(deps, info, vc_addr, root)
        }
        ExecuteMsg::UpdateModuleAddresses { to_add, to_remove } => {
            // Only Admin can call this method
            // TODO: do we want Root here too?
            ADMIN.assert_admin(deps.as_ref(), &info.sender)?;
            update_module_addresses(deps, to_add, to_remove)
        }
        ExecuteMsg::CreateModule { module, init_msg } => {
            create_module(deps, info, env, module, init_msg)
        }
        ExecuteMsg::RegisterModule {
            module,
            module_addr,
        } => register_module(deps, info, env, module, module_addr),
        ExecuteMsg::ConfigureModule {
            module_name,
            config_msg,
        } => configure_module(deps, info, module_name, config_msg),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryVersions { names } => {
            queries::handle_contract_versions_query(deps, env, names)
        }
        QueryMsg::QueryModules { names } => {
            queries::handle_module_addresses_query(deps, env, names)
        }
        QueryMsg::QueryEnabledModules {} => queries::handle_enabled_modules_query(deps),

        QueryMsg::QueryOsConfig {} => {
            let os_id = Uint64::from(OS_ID.load(deps.storage)?);
            let root = ROOT
                .get(deps)?
                .unwrap_or_else(|| Addr::unchecked(""))
                .to_string();

            let config = CONFIG.load(deps.storage)?;

            to_binary(&ConfigQueryResponse {
                root,
                os_id,
                version_control_address: config.version_control_address.to_string(),
                module_factory_address: config.module_factory_address.into_string(),
            })
        }
    }
}
