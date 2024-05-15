use abstract_sdk::{
    cw_helpers::AbstractAttributes,
    std::{
        manager::{
            state::{AccountInfo, Config, CONFIG, INFO, SUSPENSION_STATUS},
            CallbackMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
        },
        objects::{
            module_version::assert_contract_upgrade,
            validation::{validate_description, validate_link, validate_name},
        },
        proxy::state::ACCOUNT_ID,
        MANAGER,
    },
};
use abstract_std::{
    manager::{
        state::{ACCOUNT_MODULES, PENDING_GOVERNANCE},
        UpdateSubAccountAction,
    },
    objects::gov_type::GovernanceDetails,
    PROXY,
};
use cosmwasm_std::{
    ensure_eq, wasm_execute, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdError,
    StdResult,
};
use cw2::set_contract_version;
use semver::Version;

use crate::{
    commands::{self, *},
    error::ManagerError,
    queries, versioning,
};

pub type ManagerResult<R = Response> = Result<R, ManagerError>;

pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> ManagerResult {
    let version: Version = CONTRACT_VERSION.parse().unwrap();

    assert_contract_upgrade(deps.storage, MANAGER, version)?;
    set_contract_version(deps.storage, MANAGER, CONTRACT_VERSION)?;
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
    let module_factory_address = deps.api.addr_validate(&msg.module_factory_address)?;
    let version_control_address = deps.api.addr_validate(&msg.version_control_address)?;

    // Save account id
    ACCOUNT_ID.save(deps.storage, &msg.account_id)?;

    // Save config
    let config = Config {
        version_control_address: version_control_address.clone(),
        module_factory_address: module_factory_address.clone(),
    };
    CONFIG.save(deps.storage, &config)?;

    // Verify info
    validate_description(msg.description.as_deref())?;
    validate_link(msg.link.as_deref())?;
    validate_name(&msg.name)?;

    let governance_details = msg.owner.verify(deps.as_ref(), version_control_address)?;
    let owner = governance_details
        .owner_address()
        .ok_or(ManagerError::InitRenounced {})?;

    let account_info = AccountInfo {
        name: msg.name,
        governance_details,
        chain_id: env.block.chain_id,
        description: msg.description,
        link: msg.link,
    };

    INFO.save(deps.storage, &account_info)?;
    MIGRATE_CONTEXT.save(deps.storage, &vec![])?;

    // Add proxy to modules
    ACCOUNT_MODULES.save(
        deps.storage,
        PROXY,
        &deps.api.addr_validate(&msg.proxy_addr)?,
    )?;

    // Set owner
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(owner.as_str()))?;
    SUSPENSION_STATUS.save(deps.storage, &false)?;

    let mut response = ManagerResponse::new(
        "instantiate",
        vec![
            ("account_id".to_owned(), msg.account_id.to_string()),
            ("owner".to_owned(), owner.to_string()),
        ],
    );

    if !msg.install_modules.is_empty() {
        // Install modules
        let (install_msgs, install_attributes) = install_modules_internal(
            deps.branch(),
            msg.install_modules,
            config.module_factory_address,
            config.version_control_address,
            info.funds,
        )?;
        response = response
            .add_submessages(install_msgs)
            .add_abstract_attributes(install_attributes)
    }

    // Register on manager if it's sub-account
    if let GovernanceDetails::SubAccount { manager, .. } = account_info.governance_details {
        response = response.add_message(wasm_execute(
            manager,
            &ExecuteMsg::UpdateSubAccount(UpdateSubAccountAction::RegisterSubAccount {
                id: ACCOUNT_ID.load(deps.storage)?.seq(),
            }),
            vec![],
        )?);
    }

    Ok(response)
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
                ExecuteMsg::UpdateInternalConfig(config) => {
                    update_internal_config(deps, info, config)
                }
                ExecuteMsg::ProposeOwner { owner } => propose_owner(deps, env, info, owner),
                ExecuteMsg::InstallModules { modules } => install_modules(deps, info, modules),
                ExecuteMsg::UninstallModule { module_id } => {
                    uninstall_module(deps, info, module_id)
                }
                ExecuteMsg::ExecOnModule {
                    module_id,
                    exec_msg,
                } => exec_on_module(deps, info, module_id, exec_msg),
                ExecuteMsg::CreateSubAccount {
                    name,
                    description,
                    link,
                    base_asset,
                    namespace,
                    install_modules,
                    account_id,
                } => create_sub_account(
                    deps,
                    info,
                    env,
                    name,
                    description,
                    link,
                    base_asset,
                    namespace,
                    install_modules,
                    account_id,
                ),
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

                    // only owner can update IBC status
                    assert_admin_right(deps.as_ref(), &info.sender)?;
                    if let Some(ibc_enabled) = new_status {
                        let (proxy_msg, attributes) =
                            update_ibc_status_internal(deps, ibc_enabled)?;

                        response = response
                            .add_abstract_attributes(std::iter::once(attributes))
                            .add_message(proxy_msg);
                    } else {
                        return Err(ManagerError::NoUpdates {});
                    }

                    Ok(response)
                }
                ExecuteMsg::UpdateSubAccount(action) => {
                    handle_sub_account_action(deps, info, action)
                }
                ExecuteMsg::Callback(CallbackMsg {}) => handle_callback(deps, env, info),
                // Used to claim or renounce an ownership change.
                ExecuteMsg::UpdateOwnership(action) => {
                    let mut info = info;
                    let mut deps = deps;
                    let msgs = match action {
                        // Disallow the user from using the TransferOwnership action.
                        cw_ownable::Action::TransferOwnership { .. } => {
                            return Err(ManagerError::MustUseProposeOwner {});
                        }
                        cw_ownable::Action::AcceptOwnership => {
                            update_governance(deps.branch(), &mut info.sender)?
                        }
                        cw_ownable::Action::RenounceOwnership => renounce_governance(
                            deps.branch(),
                            env.contract.address,
                            &mut info.sender,
                        )?,
                    };
                    // Clear pending governance for either renounced or accepted ownership
                    PENDING_GOVERNANCE.remove(deps.storage);

                    let result: ManagerResult = abstract_sdk::execute_update_ownership!(
                        ManagerResponse,
                        deps,
                        env,
                        info,
                        action
                    );
                    Ok(result?.add_messages(msgs))
                }
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
            queries::handle_module_info_query(deps, start_after, limit)
        }
        QueryMsg::Info {} => queries::handle_account_info_query(deps),
        QueryMsg::Config {} => queries::handle_config_query(deps),
        QueryMsg::Ownership {} => abstract_sdk::query_ownership!(deps),
        QueryMsg::SubAccountIds { start_after, limit } => {
            queries::handle_sub_accounts_query(deps, start_after, limit)
        }
        QueryMsg::TopLevelOwner {} => queries::handle_top_level_owner_query(deps, env),
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

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> ManagerResult {
    match msg.id {
        commands::REGISTER_MODULES_DEPENDENCIES => {
            commands::register_dependencies(deps, msg.result)
        }
        commands::HANDLE_ADAPTER_AUTHORIZED_REMOVE => {
            commands::adapter_authorized_remove(deps, msg.result)
        }
        _ => Err(ManagerError::UnexpectedReply {}),
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::*;
    use speculoos::prelude::*;

    use super::*;
    use crate::{contract, test_common::mock_init};

    mod migrate {
        use abstract_std::AbstractError;
        use cw2::get_contract_version;

        use super::*;

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

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let small_version = Version {
                minor: version.minor - 1,
                ..version.clone()
            }
            .to_string();

            set_contract_version(deps.as_mut().storage, MANAGER, small_version)?;

            let res = contract::migrate(deps.as_mut(), mock_env(), MigrateMsg {})?;
            assert_that!(res.messages).has_length(0);

            assert_that!(get_contract_version(&deps.storage)?.version)
                .is_equal_to(version.to_string());
            Ok(())
        }
    }
}
