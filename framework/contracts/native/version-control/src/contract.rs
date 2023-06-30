use cosmwasm_std::{to_binary, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, Uint128};

use cw_semver::Version;

use abstract_core::objects::namespace::Namespace;
use abstract_core::version_control::Config;
use abstract_macros::abstract_response;
use abstract_sdk::core::{
    objects::{module_version::assert_cw_contract_upgrade, ABSTRACT_ACCOUNT_ID},
    version_control::namespaces_info,
    version_control::{
        state::{CONFIG, FACTORY},
        ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
    },
    VERSION_CONTROL,
};
use abstract_sdk::{execute_update_ownership, query_ownership};

use crate::commands::*;
use crate::error::VCError;
use crate::queries;

pub(crate) use abstract_core::objects::namespace::ABSTRACT_NAMESPACE;

pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type VCResult<T = Response> = Result<T, VCError>;

#[abstract_response(VERSION_CONTROL)]
pub struct VcResponse;

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> VCResult {
    let to_version: Version = CONTRACT_VERSION.parse()?;

    assert_cw_contract_upgrade(deps.storage, VERSION_CONTROL, to_version)?;
    cw2::set_contract_version(deps.storage, VERSION_CONTROL, CONTRACT_VERSION)?;
    Ok(VcResponse::action("migrate"))
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn instantiate(deps: DepsMut, _env: Env, info: MessageInfo, msg: InstantiateMsg) -> VCResult {
    cw2::set_contract_version(deps.storage, VERSION_CONTROL, CONTRACT_VERSION)?;

    let InstantiateMsg {
        allow_direct_module_registration,
        namespace_limit,
        namespace_registration_fee,
    } = msg;

    CONFIG.save(
        deps.storage,
        &Config {
            allow_direct_module_registration: allow_direct_module_registration.unwrap_or(false),
            namespace_limit,
            namespace_registration_fee: namespace_registration_fee.unwrap_or(Coin {
                denom: "none".to_string(),
                amount: Uint128::zero(),
            }),
        },
    )?;

    // Set up the admin as the creator of the contract
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(info.sender.as_str()))?;

    // Save the abstract namespace to the Abstract admin account
    namespaces_info().save(
        deps.storage,
        &Namespace::new(ABSTRACT_NAMESPACE)?,
        &ABSTRACT_ACCOUNT_ID,
    )?;

    FACTORY.set(deps, None)?;

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
        ExecuteMsg::SetModuleMonetization {
            module_name,
            namespace,
            monetization,
        } => set_module_monetization(deps, info, module_name, namespace, monetization),
        ExecuteMsg::ClaimNamespaces {
            account_id,
            namespaces,
        } => claim_namespaces(deps, info, account_id, namespaces),
        ExecuteMsg::RemoveNamespaces { namespaces } => remove_namespaces(deps, info, namespaces),
        ExecuteMsg::AddAccount {
            account_id,
            account_base: base,
        } => add_account(deps, info, account_id, base),
        ExecuteMsg::UpdateConfig {
            allow_direct_module_registration,
            namespace_limit,
            namespace_registration_fee,
        } => update_config(
            deps,
            info,
            allow_direct_module_registration,
            namespace_limit,
            namespace_registration_fee,
        ),
        ExecuteMsg::SetFactory { new_factory } => set_factory(deps, info, new_factory),
        ExecuteMsg::UpdateOwnership(action) => {
            execute_update_ownership!(VcResponse, deps, env, info, action)
        }
    }
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> VCResult<Binary> {
    match msg {
        QueryMsg::AccountBase { account_id } => {
            to_binary(&queries::handle_account_address_query(deps, account_id)?)
        }
        QueryMsg::Modules { infos } => to_binary(&queries::handle_modules_query(deps, infos)?),
        QueryMsg::Namespaces { accounts } => {
            to_binary(&queries::handle_namespaces_query(deps, accounts)?)
        }
        QueryMsg::Namespace { namespace } => {
            to_binary(&queries::handle_namespace_query(deps, namespace)?)
        }
        QueryMsg::Config {} => {
            let factory = FACTORY.get(deps)?.unwrap();
            to_binary(&ConfigResponse { factory })
        }
        QueryMsg::ModuleList {
            filter,
            start_after,
            limit,
        } => to_binary(&queries::handle_module_list_query(
            deps,
            start_after,
            limit,
            filter,
        )?),
        QueryMsg::NamespaceList {
            filter,
            start_after,
            limit,
        } => {
            let start_after = start_after.map(Namespace::try_from).transpose()?;
            to_binary(&queries::handle_namespace_list_query(
                deps,
                start_after,
                limit,
                filter,
            )?)
        }
        QueryMsg::Ownership {} => to_binary(&query_ownership!(deps)?),
    }
    .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract;
    use crate::testing::*;
    use abstract_core::objects::ABSTRACT_ACCOUNT_ID;
    use cosmwasm_std::testing::*;
    use speculoos::prelude::*;

    mod instantiate {
        use super::*;

        #[test]
        fn sets_abstract_namespace() -> VCResult<()> {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let account_id = namespaces_info().load(
                deps.as_ref().storage,
                &Namespace::try_from(ABSTRACT_NAMESPACE)?,
            )?;
            assert_that!(account_id).is_equal_to(ABSTRACT_ACCOUNT_ID);

            Ok(())
        }
    }

    mod migrate {
        use super::*;
        use abstract_core::AbstractError;

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

            let small_version = "0.0.0";
            cw2::set_contract_version(deps.as_mut().storage, VERSION_CONTROL, small_version)?;

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let res = migrate(deps.as_mut(), mock_env(), MigrateMsg {})?;
            assert_that!(res.messages).has_length(0);

            assert_that!(cw2::get_contract_version(&deps.storage)?.version)
                .is_equal_to(version.to_string());
            Ok(())
        }
    }
}
