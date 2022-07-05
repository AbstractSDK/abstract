use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    Uint64,
};

use crate::{commands::*, error::ManagerError, queries};
use abstract_os::manager::state::{Config, ADMIN, CONFIG, ROOT, STATUS};
use abstract_os::MANAGER;
use abstract_os::{
    manager::{ConfigQueryResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    modules::*,
    proxy::state::OS_ID,
};
use cw2::set_contract_version;

pub type ManagerResult = Result<Response, ManagerError>;

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> ManagerResult {
    // let version: Version = CONTRACT_VERSION.parse()?;
    // let storage_version: Version = get_contract_version(deps.storage)?.version.parse()?;
    // if storage_version < version {
    //     set_contract_version(deps.storage, MANAGER, CONTRACT_VERSION)?;
    // }
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ManagerResult {
    set_contract_version(deps.storage, MANAGER, CONTRACT_VERSION)?;

    let subscription_address = if let Some(addr) = msg.subscription_address {
        deps.api.addr_validate(&addr)?
    } else if msg.os_id == 0 {
        Addr::unchecked("".to_string())
    } else {
        return Err(ManagerError::NoSubscriptionAddrProvided {});
    };

    OS_ID.save(deps.storage, &msg.os_id)?;
    CONFIG.save(
        deps.storage,
        &Config {
            version_control_address: deps.api.addr_validate(&msg.version_control_address)?,
            module_factory_address: deps.api.addr_validate(&msg.module_factory_address)?,
            subscription_address,
        },
    )?;
    // Set root
    let root = deps.api.addr_validate(&msg.root_user)?;
    ROOT.set(deps.branch(), Some(root))?;
    STATUS.save(deps.storage, &true)?;
    // Setup the admin as the creator of the contract
    ADMIN.set(deps, Some(info.sender))?;
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> ManagerResult {
    match msg {
        ExecuteMsg::SuspendOs { new_status } => update_os_status(deps, info, new_status),
        msg => {
            // Block actions if user is not subscribed
            let is_subscribed = STATUS.load(deps.storage)?;
            if !is_subscribed {
                return Err(ManagerError::NotSubscribed {});
            }

            match msg {
                ExecuteMsg::SetAdmin { admin } => set_admin(deps, info, admin),
                ExecuteMsg::UpdateConfig { vc_addr, root } => {
                    execute_update_config(deps, info, vc_addr, root)
                }
                ExecuteMsg::UpdateModuleAddresses { to_add, to_remove } => {
                    // Only Admin can call this method
                    // Todo: Admin is currently defaulted to Os Factory.
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
                ExecuteMsg::ExecOnModule {
                    module_name,
                    exec_msg,
                } => exec_on_module(deps, info, module_name, exec_msg),
                ExecuteMsg::Upgrade {
                    module,
                    migrate_msg,
                } => _upgrade_module(deps, env, info, module, migrate_msg),
                ExecuteMsg::RemoveModule { module_name } => remove_module(deps, info, module_name),
                _ => panic!(),
            }
        }
    }
}

fn _upgrade_module(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    module: Module,
    migrate_msg: Option<Binary>,
) -> ManagerResult {
    ROOT.assert_admin(deps.as_ref(), &info.sender)?;
    match module.kind {
        ModuleKind::API => replace_api(deps, module.info),
        _ => match migrate_msg {
            Some(msg) => migrate_module(deps, env, module.info, msg),
            None => Err(ManagerError::MsgRequired {}),
        },
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
