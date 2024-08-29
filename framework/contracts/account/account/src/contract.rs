use abstract_macros::abstract_response;
use abstract_sdk::std::{
    objects::validation::{validate_description, validate_link, validate_name},
    proxy::state::ACCOUNT_ID,
    MANAGER,
};
use abstract_std::{
    account::{ExecuteMsg, InstantiateMsg, QueryMsg},
    manager::{
        state::{AccountInfo, Config, ACCOUNT_MODULES, CONFIG, INFO, SUSPENSION_STATUS},
        UpdateSubAccountAction,
    },
    objects::{gov_type::GovernanceDetails, ownership, AccountId},
    proxy::state::STATE,
    ACCOUNT, PROXY,
};
use cosmwasm_std::{
    ensure_eq, wasm_execute, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdError,
    StdResult,
};
use cw2::set_contract_version;

use manager::commands::*;
use proxy::commands::*;

use crate::error::AccountError;

#[abstract_response(ACCOUNT)]
pub struct AccountResponse;

pub type AccountResult<R = Response> = Result<R, AccountError>;

pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    InstantiateMsg {
        account_id,
        owner,
        install_modules,
        name,
        description,
        link,
        module_factory_address,
        version_control_address,
        ans_host_address,
    }: InstantiateMsg,
) -> AccountResult {
    // ## Proxy ##
    // Use CW2 to set the contract version, this is needed for migrations
    cw2::set_contract_version(deps.storage, PROXY, CONTRACT_VERSION)?;

    let account_id =
        account_id.unwrap_or_else(|| /*  TODO: Query VC for Sequence*/ AccountId::local(0));

    ACCOUNT_ID.save(deps.storage, &account_id)?;
    STATE.save(
        deps.storage,
        &abstract_std::proxy::state::State { modules: vec![] },
    )?;

    // ## Manage ##
    let module_factory_address = deps.api.addr_validate(&module_factory_address)?;
    let version_control_address = deps.api.addr_validate(&version_control_address)?;

    // Save config
    let config = Config {
        version_control_address: version_control_address.clone(),
        module_factory_address: module_factory_address.clone(),
    };
    abstract_std::manager::state::CONFIG.save(deps.storage, &config)?;

    // Verify info
    validate_description(description.as_deref())?;
    validate_link(link.as_deref())?;
    validate_name(&name)?;

    let account_info = AccountInfo {
        name,
        chain_id: env.block.chain_id,
        description,
        link,
    };

    INFO.save(deps.storage, &account_info)?;
    MIGRATE_CONTEXT.save(deps.storage, &vec![])?;

    // Set owner
    let cw_gov_owner = ownership::initialize_owner(
        deps.branch(),
        // TODO: support no owner here (ownership handled in SUDO)
        owner,
        config.version_control_address.clone(),
    )?;

    SUSPENSION_STATUS.save(deps.storage, &false)?;

    let mut response = ManagerResponse::new(
        "instantiate",
        vec![
            ("account_id".to_owned(), account_id.to_string()),
            ("owner".to_owned(), cw_gov_owner.owner.to_string()),
        ],
    );

    if !install_modules.is_empty() {
        // Install modules
        let (install_msgs, install_attribute) = _install_modules(
            deps.branch(),
            install_modules,
            config.module_factory_address,
            config.version_control_address,
            info.funds,
        )?;
        response = response
            .add_submessages(install_msgs)
            .add_attribute(install_attribute.key, install_attribute.value);
    }

    // Register on manager if it's sub-account
    if let GovernanceDetails::SubAccount { manager, .. } = cw_gov_owner.owner {
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
pub fn execute(mut deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> AccountResult {
    match msg {
        ExecuteMsg::UpdateStatus {
            is_suspended: suspension_status,
        } => update_account_status(deps, info, suspension_status).map_err(AccountError::from),
        msg => {
            // Block actions if user is not subscribed
            let is_suspended = SUSPENSION_STATUS.load(deps.storage)?;
            if is_suspended {
                return Err(AccountError::AccountSuspended {});
            }

            match msg {
                ExecuteMsg::UpdateInternalConfig(config) => {
                    update_internal_config(deps, info, config).map_err(AccountError::from)
                }
                ExecuteMsg::InstallModules { modules } => {
                    install_modules(deps, info, modules).map_err(AccountError::from)
                }
                ExecuteMsg::UninstallModule { module_id } => {
                    uninstall_module(deps, info, module_id).map_err(AccountError::from)
                }
                ExecuteMsg::ExecOnModule {
                    module_id,
                    exec_msg,
                } => exec_on_module(deps, info, module_id, exec_msg).map_err(AccountError::from),
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
                )
                .map_err(AccountError::from),
                ExecuteMsg::Upgrade { modules } => {
                    upgrade_modules(deps, env, info, modules).map_err(AccountError::from)
                }
                ExecuteMsg::UpdateInfo {
                    name,
                    description,
                    link,
                } => update_info(deps, info, name, description, link).map_err(AccountError::from),
                ExecuteMsg::UpdateSubAccount(action) => {
                    handle_sub_account_action(deps, info, action).map_err(AccountError::from)
                }
                // TODO: Update module migrate logic to not use callback!
                // ExecuteMsg::Callback(CallbackMsg {}) => handle_callback(deps, env, info),
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

                    let config = CONFIG.load(deps.storage)?;
                    let new_owner_attributes = ownership::update_ownership(
                        deps,
                        &env.block,
                        &info.sender,
                        config.version_control_address,
                        action,
                    )?
                    .into_attributes();
                    Ok(
                        AccountResponse::new("update_ownership", new_owner_attributes)
                            .add_messages(msgs),
                    )
                }
                ExecuteMsg::ModuleAction { msgs } => {
                    execute_module_action(deps, info, msgs).map_err(AccountError::from)
                }
                ExecuteMsg::ModuleActionWithData { msg } => {
                    execute_module_action_response(deps, info, msg).map_err(AccountError::from)
                }
                ExecuteMsg::IbcAction { msg } => {
                    execute_ibc_action(deps, info, msg).map_err(AccountError::from)
                }
                ExecuteMsg::IcaAction { action_query_msg } => {
                    ica_action(deps, info, action_query_msg).map_err(AccountError::from)
                }
                _ => panic!(),
            }
        }
    };
    Ok(Response::new())
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => todo!(),
        QueryMsg::ModuleVersions { ids } => todo!(),
        QueryMsg::ModuleAddresses { ids } => todo!(),
        QueryMsg::ModuleInfos { start_after, limit } => todo!(),
        QueryMsg::Info {} => todo!(),
        QueryMsg::SubAccountIds { start_after, limit } => todo!(),
        QueryMsg::TopLevelOwner {} => todo!(),
        QueryMsg::Ownership {} => todo!(),
    };

    Ok(Binary::default())
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
        fn disallow_same_version() -> AccountResult<()> {
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
        fn disallow_downgrade() -> AccountResult<()> {
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
        fn disallow_name_change() -> AccountResult<()> {
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
        fn works() -> AccountResult<()> {
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
