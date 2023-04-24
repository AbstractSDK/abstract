use crate::{
    commands::*,
    error::ManagerError,
    queries,
    queries::{handle_account_info_query, handle_config_query, handle_module_info_query},
    validation::{validate_description, validate_link, validate_name},
    versioning,
};
use abstract_core::manager::state::AccountInfo;
use abstract_core::objects::module_version::assert_contract_upgrade;
use abstract_sdk::core::{
    manager::{
        state::{Config, ACCOUNT_FACTORY, CONFIG, INFO, SUSPENSION_STATUS},
        CallbackMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
    },
    proxy::state::ACCOUNT_ID,
    MANAGER,
};
use cosmwasm_std::{
    ensure_eq, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};
use cw2::set_contract_version;
use semver::Version;

pub type ManagerResult<R = Response> = Result<R, ManagerError>;

pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
pub(crate) const MIN_DESC_LENGTH: usize = 1;
pub(crate) const MAX_DESC_LENGTH: usize = 1024;
/// Minimum link length is 11, because the shortest url could be http://a.be
pub(crate) const MIN_LINK_LENGTH: usize = 11;
pub(crate) const MAX_LINK_LENGTH: usize = 128;
pub(crate) const MIN_TITLE_LENGTH: usize = 1;
pub(crate) const MAX_TITLE_LENGTH: usize = 64;

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> ManagerResult {
    let version: Version = CONTRACT_VERSION.parse().unwrap();

    assert_contract_upgrade(deps.storage, MANAGER, version)?;
    set_contract_version(deps.storage, MANAGER, CONTRACT_VERSION)?;
    Ok(ManagerResponse::action("migrate"))
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ManagerResult {
    set_contract_version(deps.storage, MANAGER, CONTRACT_VERSION)?;

    ACCOUNT_ID.save(deps.storage, &msg.account_id)?;
    CONFIG.save(
        deps.storage,
        &Config {
            version_control_address: deps.api.addr_validate(&msg.version_control_address)?,
            module_factory_address: deps.api.addr_validate(&msg.module_factory_address)?,
        },
    )?;

    // Verify info
    validate_description(&msg.description)?;
    validate_link(&msg.link)?;
    validate_name(&msg.name)?;

    let governance_details = msg.owner.verify(deps.api)?;
    let owner = governance_details.owner_address();

    let account_info = AccountInfo {
        name: msg.name,
        governance_details,
        chain_id: env.block.chain_id,
        description: msg.description,
        link: msg.link,
    };

    INFO.save(deps.storage, &account_info)?;
    MIGRATE_CONTEXT.save(deps.storage, &vec![])?;

    // Set owner
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(owner.as_str()))?;
    SUSPENSION_STATUS.save(deps.storage, &false)?;
    ACCOUNT_FACTORY.set(deps, Some(info.sender))?;
    Ok(ManagerResponse::new(
        "instantiate",
        vec![
            ("account_id", msg.account_id.to_string()),
            ("owner", owner.to_string()),
        ],
    ))
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> ManagerResult {
    match msg {
        ExecuteMsg::UpdateStatus {
            is_suspended: suspension_status,
        } => update_account_status(deps, info, suspension_status),
        msg => {
            // Block actions if user is not subscribed
            let is_suspended = SUSPENSION_STATUS.load(deps.storage)?;
            if is_suspended {
                return Err(ManagerError::AccountSuspended {});
            }

            match msg {
                ExecuteMsg::SetOwner { owner } => set_owner(deps, env, info, owner),
                ExecuteMsg::UpdateModuleAddresses { to_add, to_remove } => {
                    // only Account Factory/Owner can add custom modules.
                    // required to add Proxy after init by Account Factory.
                    ACCOUNT_FACTORY
                        .assert_admin(deps.as_ref(), &info.sender)
                        .or_else(|_| cw_ownable::assert_owner(deps.storage, &info.sender))?;
                    update_module_addresses(deps, to_add, to_remove)
                }
                ExecuteMsg::InstallModule { module, init_msg } => {
                    install_module(deps, info, env, module, init_msg)
                }
                ExecuteMsg::UninstallModule { module_id } => {
                    uninstall_module(deps, info, module_id)
                }
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
                ExecuteMsg::UpdateSettings {
                    ibc_enabled: new_status,
                } => {
                    let mut response: Response = ManagerResponse::action("update_settings");

                    if let Some(ibc_enabled) = new_status {
                        response = update_ibc_status(deps, info, ibc_enabled, response)?;
                    } else {
                        return Err(ManagerError::NoUpdates {});
                    }

                    Ok(response)
                }
                ExecuteMsg::Callback(CallbackMsg {}) => handle_callback(deps, env, info),
                ExecuteMsg::UpdateOwnership(action) => match action {
                    // Disallow the user from using the TransferOwnership action
                    cw_ownable::Action::TransferOwnership { .. } => {
                        Err(ManagerError::MustUseSetOwner {})
                    }
                    _ => {
                        abstract_sdk::execute_update_ownership!(
                            ManagerResponse,
                            deps,
                            env,
                            info,
                            action
                        )
                    }
                },
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
        QueryMsg::Info {} => handle_account_info_query(deps),
        QueryMsg::Config {} => handle_config_query(deps),
        QueryMsg::Ownership {} => abstract_sdk::query_ownership!(deps),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract;
    use cosmwasm_std::testing::*;
    use speculoos::prelude::*;

    use crate::test_common::mock_init;

    mod migrate {
        use super::*;
        use abstract_core::AbstractError;
        use cw2::get_contract_version;

        #[test]
        fn disallow_same_version() -> ManagerResult<()> {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let res = contract::migrate(deps.as_mut(), mock_env(), MigrateMsg {});

            assert_that!(res)
                .is_err()
                .is_equal_to(ManagerError::Abstract(
                    AbstractError::CannotDowngradeContract {
                        contract: MANAGER.to_string(),
                        from: version.clone(),
                        to: version,
                    },
                ));

            Ok(())
        }

        #[test]
        fn disallow_downgrade() -> ManagerResult<()> {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let big_version = "999.999.999";
            set_contract_version(deps.as_mut().storage, MANAGER, big_version)?;

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let res = contract::migrate(deps.as_mut(), mock_env(), MigrateMsg {});

            assert_that!(res)
                .is_err()
                .is_equal_to(ManagerError::Abstract(
                    AbstractError::CannotDowngradeContract {
                        contract: MANAGER.to_string(),
                        from: big_version.parse().unwrap(),
                        to: version,
                    },
                ));

            Ok(())
        }

        #[test]
        fn disallow_name_change() -> ManagerResult<()> {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let old_version = "0.0.0";
            let old_name = "old:contract";
            set_contract_version(deps.as_mut().storage, old_name, old_version)?;

            let res = contract::migrate(deps.as_mut(), mock_env(), MigrateMsg {});

            assert_that!(res)
                .is_err()
                .is_equal_to(ManagerError::Abstract(
                    AbstractError::ContractNameMismatch {
                        from: old_name.parse().unwrap(),
                        to: MANAGER.parse().unwrap(),
                    },
                ));

            Ok(())
        }

        #[test]
        fn works() -> ManagerResult<()> {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let small_version = "0.0.0";
            set_contract_version(deps.as_mut().storage, MANAGER, small_version)?;

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let res = contract::migrate(deps.as_mut(), mock_env(), MigrateMsg {})?;
            assert_that!(res.messages).has_length(0);

            assert_that!(get_contract_version(&deps.storage)?.version)
                .is_equal_to(version.to_string());
            Ok(())
        }
    }
}
