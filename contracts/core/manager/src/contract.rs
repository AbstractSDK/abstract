use crate::{
    commands::*,
    error::ManagerError,
    queries,
    queries::{handle_config_query, handle_module_info_query, handle_os_info_query},
    validation::{validate_description, validate_link, validate_name_or_gov_type},
    versioning,
};
use abstract_sdk::os::{
    manager::{
        state::{Config, OsInfo, CONFIG, INFO, OS_FACTORY, ROOT, STATUS},
        CallbackMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
    },
    objects::module_version::{migrate_module_data, set_module_data},
    proxy::state::OS_ID,
    MANAGER,
};
use cosmwasm_std::{
    ensure_eq, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};
use cw2::{get_contract_version, set_contract_version};
use semver::Version;

pub type ManagerResult<R = Response> = Result<R, ManagerError>;

pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
pub(crate) const MIN_DESC_LENGTH: usize = 4;
pub(crate) const MAX_DESC_LENGTH: usize = 1024;
pub(crate) const MIN_LINK_LENGTH: usize = 11;
pub(crate) const MAX_LINK_LENGTH: usize = 128;
pub(crate) const MIN_TITLE_LENGTH: usize = 4;
pub(crate) const MAX_TITLE_LENGTH: usize = 64;

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> ManagerResult {
    let version: Version = CONTRACT_VERSION.parse().unwrap();
    let storage_version: Version = get_contract_version(deps.storage)?.version.parse().unwrap();
    if storage_version < version {
        set_contract_version(deps.storage, MANAGER, CONTRACT_VERSION)?;
        migrate_module_data(deps.storage, MANAGER, CONTRACT_VERSION, None::<String>)?;
    }
    Ok(ManagerResponse::action("migrate"))
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ManagerResult {
    set_contract_version(deps.storage, MANAGER, CONTRACT_VERSION)?;
    set_module_data(deps.storage, MANAGER, CONTRACT_VERSION, &[], None::<String>)?;

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
    MIGRATE_CONTEXT.save(deps.storage, &vec![])?;

    // Set root
    let root = deps.api.addr_validate(&msg.root_user)?;
    ROOT.set(deps.branch(), Some(root))?;
    STATUS.save(deps.storage, &true)?;
    OS_FACTORY.set(deps, Some(info.sender))?;
    Ok(ManagerResponse::new(
        "instantiate",
        vec![("os_id", msg.os_id.to_string()), ("owner", msg.root_user)],
    ))
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> ManagerResult {
    match msg {
        ExecuteMsg::SuspendOs { new_status } => update_subscription_status(deps, info, new_status),
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
                ExecuteMsg::InstallModule { module, init_msg } => {
                    install_module(deps, info, env, module, init_msg)
                }
                ExecuteMsg::RemoveModule { module_id } => uninstall_module(deps, info, module_id),
                ExecuteMsg::RegisterModule {
                    module,
                    module_addr,
                } => register_module(deps, info, env, module, module_addr),
                ExecuteMsg::ExecOnModule {
                    module_id,
                    exec_msg,
                } => exec_on_module(deps, info, module_id, exec_msg),
                ExecuteMsg::Upgrade { modules } => upgrade_modules(deps, env, info, modules),
                ExecuteMsg::UpdateInfo {
                    name,
                    description,
                    link,
                } => update_info(deps, info, name, description, link),
                ExecuteMsg::EnableIBC { new_status } => enable_ibc(deps, info, new_status),
                ExecuteMsg::Callback(CallbackMsg {}) => handle_callback(deps, env, info),
                _ => panic!(),
            }
        }
    }
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::ModuleVersions { ids } => queries::handle_contract_versions_query(deps, env, ids),
        QueryMsg::ModuleAddresses { ids } => queries::handle_module_address_query(deps, env, ids),
        QueryMsg::ModuleInfos { start_after, limit } => {
            handle_module_info_query(deps, start_after, limit)
        }
        QueryMsg::Info {} => handle_os_info_query(deps),
        QueryMsg::Config {} => handle_config_query(deps),
    }
}

pub fn handle_callback(mut deps: DepsMut, env: Env, info: MessageInfo) -> ManagerResult {
    ensure_eq!(
        info.sender,
        env.contract.address,
        StdError::generic_err("Callback must be called by contract")
    );
    let migrated_modules = MIGRATE_CONTEXT.load(deps.storage)?;

    for (migrated_module_id, old_deps) in migrated_modules {
        versioning::maybe_remove_old_deps(deps.branch(), &migrated_module_id, &old_deps)?;
        let new_deps =
            versioning::maybe_add_new_deps(deps.branch(), &migrated_module_id, &old_deps)?;
        versioning::assert_dependency_requirements(deps.as_ref(), &new_deps, &migrated_module_id)?;
    }

    MIGRATE_CONTEXT.save(deps.storage, &vec![])?;
    Ok(Response::new())
}
