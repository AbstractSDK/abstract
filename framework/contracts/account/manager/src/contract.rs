use abstract_sdk::{
    feature_objects::VersionControlContract,
    std::{
        manager::{
            state::{AccountInfo, Config, ACCOUNT_MODULES, CONFIG, INFO, SUSPENSION_STATUS},
            CallbackMsg, ExecuteMsg, InstantiateMsg, QueryMsg, UpdateSubAccountAction,
        },
        objects::{
            gov_type::GovernanceDetails,
            ownership,
            validation::{validate_description, validate_link, validate_name},
        },
        proxy::state::ACCOUNT_ID,
        ACCOUNT,
    },
};

use cosmwasm_std::{
    ensure_eq, wasm_execute, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdError,
    StdResult,
};
use cw2::set_contract_version;

use crate::{
    commands::{self, *},
    error::ManagerError,
    queries, versioning,
};

pub type ManagerResult<R = Response> = Result<R, ManagerError>;

pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
pub use crate::migrate::migrate;

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ManagerResult {
    set_contract_version(deps.storage, ACCOUNT, CONTRACT_VERSION)?;
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

    let account_info = AccountInfo {
        name: msg.name,
        description: msg.description,
        link: msg.link,
    };

    INFO.save(deps.storage, &account_info)?;
    MIGRATE_CONTEXT.save(deps.storage, &vec![])?;

    // Add proxy to modules
    ACCOUNT_MODULES.save(
        deps.storage,
        ACCOUNT,
        &deps.api.addr_validate(&msg.proxy_addr)?,
    )?;

    // Set owner
    let cw_gov_owner = ownership::initialize_owner(
        deps.branch(),
        msg.owner,
        config.version_control_address.clone(),
    )?;

    SUSPENSION_STATUS.save(deps.storage, &false)?;

    let mut response = ManagerResponse::new(
        "instantiate",
        vec![
            ("account_id".to_owned(), msg.account_id.to_string()),
            ("owner".to_owned(), cw_gov_owner.owner.to_string()),
        ],
    );

    if !msg.install_modules.is_empty() {
        // Install modules
        let (install_msgs, install_attribute) =
            _install_modules(deps.branch(), msg.install_modules, info.funds)?;
        response = response
            .add_submessages(install_msgs)
            .add_attribute(install_attribute.key, install_attribute.value);
    }

    // Register on manager if it's sub-account
    if let GovernanceDetails::SubAccount { account } = cw_gov_owner.owner {
        response = response.add_message(wasm_execute(
            account,
            &ExecuteMsg::UpdateSubAccount(UpdateSubAccountAction::RegisterSubAccount {
                id: ACCOUNT_ID.load(deps.storage)?.seq(),
            }),
            vec![],
        )?);
    }

    Ok(response)
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn execute(mut deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> ManagerResult {
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
                ExecuteMsg::UpdateSubAccount(action) => {
                    handle_sub_account_action(deps, info, action)
                }
                ExecuteMsg::Callback(CallbackMsg {}) => handle_callback(deps, env, info),
                // Used to claim or renounce an ownership change.
                ExecuteMsg::UpdateOwnership(action) => {
                    // If sub-account related it may require some messages to be constructed beforehand
                    let msgs = match &action {
                        ownership::GovAction::TransferOwnership { .. } => vec![],
                        ownership::GovAction::AcceptOwnership => {
                            maybe_update_sub_account_governance(deps.branch())?
                        }
                        ownership::GovAction::RenounceOwnership => {
                            remove_account_from_contracts(deps.branch())?
                        }
                    };

                    let version_control = VersionControlContract::new(deps.api)?;
                    let new_owner_attributes = ownership::update_ownership(
                        deps,
                        &env.block,
                        &info.sender,
                        version_control.address,
                        action,
                    )?
                    .into_attributes();
                    Ok(
                        ManagerResponse::new("update_ownership", new_owner_attributes)
                            .add_messages(msgs),
                    )
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
        QueryMsg::Ownership {} => {
            cosmwasm_std::to_json_binary(&ownership::get_ownership(deps.storage)?)
        }
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
        _ => Err(ManagerError::UnexpectedReply {}),
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::*;
    use semver::Version;
    use speculoos::prelude::*;

    use super::*;
    use crate::{contract, test_common::mock_init};

    mod migrate {
        use abstract_std::{manager::MigrateMsg, AbstractError};
        use cw2::get_contract_version;

        use super::*;

        #[test]
        fn disallow_same_version() -> ManagerResult<()> {
            let mut deps = mock_dependencies();
            mock_init(&mut deps)?;

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let res = contract::migrate(deps.as_mut(), mock_env(), MigrateMsg {});

            assert_that!(res)
                .is_err()
                .is_equal_to(ManagerError::Abstract(
                    AbstractError::CannotDowngradeContract {
                        contract: ACCOUNT.to_string(),
                        from: version.clone(),
                        to: version,
                    },
                ));

            Ok(())
        }

        #[test]
        fn disallow_downgrade() -> ManagerResult<()> {
            let mut deps = mock_dependencies();
            mock_init(&mut deps)?;

            let big_version = "999.999.999";
            set_contract_version(deps.as_mut().storage, ACCOUNT, big_version)?;

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let res = contract::migrate(deps.as_mut(), mock_env(), MigrateMsg {});

            assert_that!(res)
                .is_err()
                .is_equal_to(ManagerError::Abstract(
                    AbstractError::CannotDowngradeContract {
                        contract: ACCOUNT.to_string(),
                        from: big_version.parse().unwrap(),
                        to: version,
                    },
                ));

            Ok(())
        }

        #[test]
        fn disallow_name_change() -> ManagerResult<()> {
            let mut deps = mock_dependencies();
            mock_init(&mut deps)?;

            let old_version = "0.0.0";
            let old_name = "old:contract";
            set_contract_version(deps.as_mut().storage, old_name, old_version)?;

            let res = contract::migrate(deps.as_mut(), mock_env(), MigrateMsg {});

            assert_that!(res)
                .is_err()
                .is_equal_to(ManagerError::Abstract(
                    AbstractError::ContractNameMismatch {
                        from: old_name.parse().unwrap(),
                        to: ACCOUNT.parse().unwrap(),
                    },
                ));

            Ok(())
        }

        #[test]
        fn works() -> ManagerResult<()> {
            let mut deps = mock_dependencies();
            mock_init(&mut deps)?;

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let small_version = Version {
                minor: version.minor - 1,
                ..version.clone()
            }
            .to_string();

            set_contract_version(deps.as_mut().storage, ACCOUNT, small_version)?;

            let res = contract::migrate(deps.as_mut(), mock_env(), MigrateMsg {})?;
            assert_that!(res.messages).has_length(0);

            assert_that!(get_contract_version(&deps.storage)?.version)
                .is_equal_to(version.to_string());
            Ok(())
        }
    }
}
