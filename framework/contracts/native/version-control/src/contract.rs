use abstract_macros::abstract_response;
use abstract_sdk::{
    core::{
        objects::{module_version::assert_cw_contract_upgrade, ABSTRACT_ACCOUNT_ID},
        version_control::{
            state::CONFIG, ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
        },
        VERSION_CONTROL,
    },
    execute_update_ownership, query_ownership,
};
pub(crate) use abstract_std::objects::namespace::ABSTRACT_NAMESPACE;
use abstract_std::{
    objects::namespace::Namespace,
    version_control::{state::NAMESPACES_INFO, Config},
};
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response};
use cw_semver::Version;

use crate::{commands::*, error::VCError, queries};

pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type VCResult<T = Response> = Result<T, VCError>;

#[abstract_response(VERSION_CONTROL)]
pub struct VcResponse;

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> VCResult {
    let to_version: Version = CONTRACT_VERSION.parse()?;

    let vc_addr_raw = deps.storage.get(b"fac");
    if let Some(vc_addr) = vc_addr_raw {
        let vc_addr: Option<cosmwasm_std::Addr> = cosmwasm_std::from_json(vc_addr)?;

        CONFIG.update(deps.storage, |mut cfg| {
            // Save factory address to a new place
            cfg.account_factory_address = vc_addr;
            // Check if fee requires in migration
            if cfg
                .namespace_registration_fee
                .as_ref()
                .map(|fee| fee.amount.is_zero())
                .unwrap_or(false)
            {
                // registration_fee is Option now, but was 0 in previous version
                cfg.namespace_registration_fee = None;
            }
            VCResult::Ok(cfg)
        })?;
    }
    // Remove old factory
    deps.storage.remove(b"fac");

    assert_cw_contract_upgrade(deps.storage, VERSION_CONTROL, to_version)?;
    cw2::set_contract_version(deps.storage, VERSION_CONTROL, CONTRACT_VERSION)?;
    Ok(VcResponse::action("migrate"))
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn instantiate(deps: DepsMut, _env: Env, _info: MessageInfo, msg: InstantiateMsg) -> VCResult {
    cw2::set_contract_version(deps.storage, VERSION_CONTROL, CONTRACT_VERSION)?;

    let InstantiateMsg {
        admin,
        allow_direct_module_registration_and_updates,
        namespace_registration_fee,
    } = msg;

    CONFIG.save(
        deps.storage,
        &Config {
            // Account factory should be set by `update_config`
            account_factory_address: None,
            allow_direct_module_registration_and_updates:
                allow_direct_module_registration_and_updates.unwrap_or(false),
            namespace_registration_fee,
        },
    )?;

    // Set up the admin
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(&admin))?;

    // Save the abstract namespace to the Abstract admin account
    NAMESPACES_INFO.save(
        deps.storage,
        &Namespace::new(ABSTRACT_NAMESPACE)?,
        &ABSTRACT_ACCOUNT_ID,
    )?;

    Ok(VcResponse::action("instantiate"))
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> VCResult {
    match msg {
        ExecuteMsg::ProposeModules { modules } => propose_modules(deps, info, modules),
        ExecuteMsg::ApproveOrRejectModules { approves, rejects } => {
            approve_or_reject_modules(deps, info, approves, rejects)
        }
        ExecuteMsg::RemoveModule { module } => remove_module(deps, info, module),
        ExecuteMsg::YankModule { module } => yank_module(deps, info, module),
        ExecuteMsg::UpdateModuleConfiguration {
            module_name,
            namespace,
            update_module,
        } => update_module_config(deps, info, module_name, namespace, update_module),
        ExecuteMsg::ClaimNamespace {
            namespace,
            account_id,
        } => claim_namespace(deps, info, account_id, namespace),
        ExecuteMsg::RemoveNamespaces { namespaces } => remove_namespaces(deps, info, namespaces),
        ExecuteMsg::AddAccount {
            account_id,
            account_base: base,
            namespace,
        } => add_account(deps, info, account_id, base, namespace),
        ExecuteMsg::UpdateConfig {
            account_factory_address,
            allow_direct_module_registration_and_updates,
            namespace_registration_fee,
        } => update_config(
            deps,
            info,
            account_factory_address,
            allow_direct_module_registration_and_updates,
            namespace_registration_fee,
        ),
        ExecuteMsg::UpdateOwnership(action) => {
            execute_update_ownership!(VcResponse, deps, env, info, action)
        }
    }
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> VCResult<Binary> {
    match msg {
        QueryMsg::AccountBase { account_id } => {
            to_json_binary(&queries::handle_account_address_query(deps, account_id)?)
        }
        QueryMsg::Modules { infos } => to_json_binary(&queries::handle_modules_query(deps, infos)?),
        QueryMsg::Namespaces { accounts } => {
            to_json_binary(&queries::handle_namespaces_query(deps, accounts)?)
        }
        QueryMsg::Namespace { namespace } => {
            to_json_binary(&queries::handle_namespace_query(deps, namespace)?)
        }
        QueryMsg::Config {} => {
            let config = CONFIG.load(deps.storage)?;
            to_json_binary(&ConfigResponse {
                account_factory_address: config.account_factory_address,
                allow_direct_module_registration_and_updates: config
                    .allow_direct_module_registration_and_updates,
                namespace_registration_fee: config.namespace_registration_fee,
            })
        }
        QueryMsg::ModuleList {
            filter,
            start_after,
            limit,
        } => to_json_binary(&queries::handle_module_list_query(
            deps,
            start_after,
            limit,
            filter,
        )?),
        QueryMsg::NamespaceList { start_after, limit } => {
            let start_after = start_after.map(Namespace::try_from).transpose()?;
            to_json_binary(&queries::handle_namespace_list_query(
                deps,
                start_after,
                limit,
            )?)
        }
        QueryMsg::Ownership {} => query_ownership!(deps),
    }
    .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use abstract_std::objects::ABSTRACT_ACCOUNT_ID;
    use cosmwasm_std::testing::*;
    use speculoos::prelude::*;

    use super::*;
    use crate::{contract, testing::*};

    mod instantiate {
        use super::*;

        #[test]
        fn sets_abstract_namespace() -> VCResult<()> {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let account_id = NAMESPACES_INFO.load(
                deps.as_ref().storage,
                &Namespace::try_from(ABSTRACT_NAMESPACE)?,
            )?;
            assert_that!(account_id).is_equal_to(ABSTRACT_ACCOUNT_ID);

            Ok(())
        }
    }

    mod migrate {
        use abstract_std::AbstractError;

        use super::*;

        #[test]
        fn disallow_same_version() -> VCResult<()> {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let res = contract::migrate(deps.as_mut(), mock_env(), MigrateMsg {});

            assert_that!(res).is_err().is_equal_to(VCError::Abstract(
                AbstractError::CannotDowngradeContract {
                    contract: VERSION_CONTROL.to_string(),
                    from: version.to_string().parse().unwrap(),
                    to: version.to_string().parse().unwrap(),
                },
            ));

            Ok(())
        }

        #[test]
        fn disallow_downgrade() -> VCResult<()> {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let big_version = "999.999.999";
            cw2::set_contract_version(deps.as_mut().storage, VERSION_CONTROL, big_version)?;

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let res = migrate(deps.as_mut(), mock_env(), MigrateMsg {});

            assert_that!(res).is_err().is_equal_to(VCError::Abstract(
                AbstractError::CannotDowngradeContract {
                    contract: VERSION_CONTROL.to_string(),
                    from: big_version.parse().unwrap(),
                    to: version.to_string().parse().unwrap(),
                },
            ));

            Ok(())
        }

        #[test]
        fn disallow_name_change() -> VCResult<()> {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let old_version = "0.0.0";
            let old_name = "old:contract";
            cw2::set_contract_version(deps.as_mut().storage, old_name, old_version)?;

            let res = migrate(deps.as_mut(), mock_env(), MigrateMsg {});

            assert_that!(res).is_err().is_equal_to(VCError::Abstract(
                AbstractError::ContractNameMismatch {
                    from: old_name.to_string(),
                    to: VERSION_CONTROL.to_string(),
                },
            ));

            Ok(())
        }

        #[test]
        fn works() -> VCResult<()> {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let small_version = Version {
                minor: version.minor - 1,
                ..version.clone()
            }
            .to_string();
            cw2::set_contract_version(deps.as_mut().storage, VERSION_CONTROL, small_version)?;

            let res = migrate(deps.as_mut(), mock_env(), MigrateMsg {})?;
            assert_that!(res.messages).has_length(0);

            assert_that!(cw2::get_contract_version(&deps.storage)?.version)
                .is_equal_to(version.to_string());
            Ok(())
        }
    }
}
