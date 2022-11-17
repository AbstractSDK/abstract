use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use semver::Version;

use crate::queries::{handle_config_query, handle_module_info_query, handle_os_info_query};
use crate::validators::{validate_description, validate_link, validate_name_or_gov_type};
use crate::{commands::*, error::ManagerError, queries};
use abstract_sdk::os::manager::state::{Config, OsInfo, CONFIG, INFO, OS_FACTORY, ROOT, STATUS};
use abstract_sdk::os::MANAGER;
use abstract_sdk::os::{
    manager::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    proxy::state::OS_ID,
};
use cw2::{get_contract_version, set_contract_version};

pub type ManagerResult = Result<Response, ManagerError>;

pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
pub(crate) const MIN_DESC_LENGTH: usize = 4;
pub(crate) const MAX_DESC_LENGTH: usize = 1024;
pub(crate) const MIN_LINK_LENGTH: usize = 12;
pub(crate) const MAX_LINK_LENGTH: usize = 128;
pub(crate) const MIN_TITLE_LENGTH: usize = 4;
pub(crate) const MAX_TITLE_LENGTH: usize = 64;
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> ManagerResult {
    let version: Version = CONTRACT_VERSION.parse().unwrap();
    let storage_version: Version = get_contract_version(deps.storage)?.version.parse().unwrap();
    if storage_version < version {
        set_contract_version(deps.storage, MANAGER, CONTRACT_VERSION)?;
    }
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ManagerResult {
    set_contract_version(deps.storage, MANAGER, CONTRACT_VERSION)?;

    let subscription_address = msg
        .subscription_address
        .map(|a| deps.api.addr_validate(&a))
        .transpose()?;

    OS_ID.save(deps.storage, &msg.os_id)?;
    CONFIG.save(
        deps.storage,
        &Config {
            version_control_address: deps.api.addr_validate(&msg.version_control_address)?,
            module_factory_address: deps.api.addr_validate(&msg.module_factory_address)?,
            subscription_address,
        },
    )?;

    // Verify info
    validate_description(&msg.description)?;
    validate_link(&msg.link)?;
    validate_name_or_gov_type(&msg.name)?;

    let os_info = OsInfo {
        name: msg.name,
        governance_type: msg.governance_type,
        chain_id: env.block.chain_id,
        description: msg.description,
        link: msg.link,
    };

    INFO.save(deps.storage, &os_info)?;
    // Set root
    let root = deps.api.addr_validate(&msg.root_user)?;
    ROOT.set(deps.branch(), Some(root))?;
    STATUS.save(deps.storage, &true)?;
    OS_FACTORY.set(deps, Some(info.sender))?;
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
                ExecuteMsg::SetRoot {
                    root,
                    governance_type,
                } => set_root_and_gov_type(deps, info, root, governance_type),
                ExecuteMsg::UpdateModuleAddresses { to_add, to_remove } => {
                    // only factory/root can add custom modules.
                    // required to add Proxy after init by os factory.
                    OS_FACTORY
                        .assert_admin(deps.as_ref(), &info.sender)
                        .or_else(|_| ROOT.assert_admin(deps.as_ref(), &info.sender))?;

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
                    module_id,
                    exec_msg,
                } => exec_on_module(deps, info, module_id, exec_msg),
                ExecuteMsg::Upgrade {
                    module,
                    migrate_msg,
                } => upgrade_module(deps, env, info, module, migrate_msg),
                ExecuteMsg::RemoveModule { module_id } => remove_module(deps, info, module_id),
                ExecuteMsg::UpdateInfo {
                    name,
                    description,
                    link,
                } => update_info(deps, info, name, description, link),
                ExecuteMsg::EnableIBC { new_status } => enable_ibc(deps, info, new_status),
                _ => panic!(),
            }
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::ModuleVersions { ids } => queries::handle_contract_versions_query(deps, env, ids),
        QueryMsg::ModuleAddresses { ids } => queries::handle_module_address_query(deps, env, ids),
        QueryMsg::ModuleInfos {
            page_token,
            page_size,
        } => handle_module_info_query(deps, page_token, page_size),
        QueryMsg::Info {} => handle_os_info_query(deps),
        QueryMsg::Config {} => handle_config_query(deps),
    }
}
