use abstract_macros::abstract_response;
use abstract_sdk::{execute_update_ownership, query_ownership};
pub(crate) use abstract_std::objects::namespace::ABSTRACT_NAMESPACE;
use abstract_std::{
    objects::namespace::Namespace,
    registry::{
        state::{LOCAL_ACCOUNT_SEQUENCE, NAMESPACES_INFO},
        Config,
    },
};
use abstract_std::{
    objects::ABSTRACT_ACCOUNT_ID,
    registry::{state::CONFIG, ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg},
    REGISTRY,
};
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response};

use crate::{commands::*, error::RegistryError, queries};

pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type VCResult<T = Response> = Result<T, RegistryError>;

#[abstract_response(REGISTRY)]
pub struct VcResponse;

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn instantiate(deps: DepsMut, _env: Env, _info: MessageInfo, msg: InstantiateMsg) -> VCResult {
    cw2::set_contract_version(deps.storage, REGISTRY, CONTRACT_VERSION)?;

    let InstantiateMsg {
        admin,
        security_disabled,
        namespace_registration_fee,
    } = msg;

    CONFIG.save(
        deps.storage,
        &Config {
            // Account factory should be set by `update_config`
            security_disabled: security_disabled.unwrap_or(false),
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

    LOCAL_ACCOUNT_SEQUENCE.save(deps.storage, &0)?;

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
        ExecuteMsg::ForgoNamespace { namespaces } => forgo_namespace(deps, info, namespaces),
        ExecuteMsg::AddAccount { namespace, creator } => {
            add_account(deps, info, namespace, creator)
        }
        ExecuteMsg::UpdateConfig {
            security_disabled,
            namespace_registration_fee,
        } => update_config(deps, info, security_disabled, namespace_registration_fee),
        ExecuteMsg::UpdateOwnership(action) => {
            execute_update_ownership!(VcResponse, deps, env, info, action)
        }
    }
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> VCResult<Binary> {
    match msg {
        QueryMsg::Account { account_id } => {
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
            let local_account_sequence = LOCAL_ACCOUNT_SEQUENCE.load(deps.storage)?;
            to_json_binary(&ConfigResponse {
                security_disabled: config.security_disabled,
                namespace_registration_fee: config.namespace_registration_fee,
                local_account_sequence,
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
    use super::*;

    #[cfg(test)]
    mod testing {
        use abstract_std::registry;
        use abstract_testing::prelude::*;
        use cosmwasm_std::{
            testing::{message_info, mock_dependencies, MockApi},
            OwnedDeps, Response,
        };
        use speculoos::prelude::*;

        use crate::{contract, error::RegistryError};

        /// Initialize the registry with admin as creator and factory
        pub fn mock_init(
            deps: &mut OwnedDeps<MockStorage, MockApi, MockQuerier>,
        ) -> Result<Response, RegistryError> {
            let abstr = AbstractMockAddrs::new(deps.api);
            let info = message_info(&abstr.owner, &[]);
            let env = mock_env_validated(deps.api);
            let admin = info.sender.to_string();

            contract::instantiate(
                deps.as_mut(),
                env,
                info,
                registry::InstantiateMsg {
                    admin,
                    security_disabled: Some(true),
                    namespace_registration_fee: None,
                },
            )
        }

        mod migrate {
            use abstract_std::{registry::MigrateMsg, AbstractError, REGISTRY};
            use contract::{VCResult, CONTRACT_VERSION};
            use semver::Version;

            use super::*;

            #[test]
            fn disallow_same_version() -> VCResult<()> {
                let mut deps = mock_dependencies();
                let env = mock_env_validated(deps.api);
                mock_init(&mut deps)?;

                let version: Version = CONTRACT_VERSION.parse().unwrap();

                let res = crate::migrate::migrate(deps.as_mut(), env, MigrateMsg::Migrate {});

                assert_that!(res)
                    .is_err()
                    .is_equal_to(RegistryError::Abstract(
                        AbstractError::CannotDowngradeContract {
                            contract: REGISTRY.to_string(),
                            from: version.to_string().parse().unwrap(),
                            to: version.to_string().parse().unwrap(),
                        },
                    ));

                Ok(())
            }

            #[test]
            fn disallow_downgrade() -> VCResult<()> {
                let mut deps = mock_dependencies();
                let env = mock_env_validated(deps.api);
                mock_init(&mut deps)?;

                let big_version = "999.999.999";
                cw2::set_contract_version(deps.as_mut().storage, REGISTRY, big_version)?;

                let version: Version = CONTRACT_VERSION.parse().unwrap();

                let res = crate::migrate::migrate(deps.as_mut(), env, MigrateMsg::Migrate {});

                assert_that!(res)
                    .is_err()
                    .is_equal_to(RegistryError::Abstract(
                        AbstractError::CannotDowngradeContract {
                            contract: REGISTRY.to_string(),
                            from: big_version.parse().unwrap(),
                            to: version.to_string().parse().unwrap(),
                        },
                    ));

                Ok(())
            }

            #[test]
            fn disallow_name_change() -> VCResult<()> {
                let mut deps = mock_dependencies();
                let env = mock_env_validated(deps.api);
                mock_init(&mut deps)?;

                let old_version = "0.0.0";
                let old_name = "old:contract";
                cw2::set_contract_version(deps.as_mut().storage, old_name, old_version)?;

                let res = crate::migrate::migrate(deps.as_mut(), env, MigrateMsg::Migrate {});

                assert_that!(res)
                    .is_err()
                    .is_equal_to(RegistryError::Abstract(
                        AbstractError::ContractNameMismatch {
                            from: old_name.to_string(),
                            to: REGISTRY.to_string(),
                        },
                    ));

                Ok(())
            }

            #[test]
            fn works() -> VCResult<()> {
                let mut deps = mock_dependencies();
                let env = mock_env_validated(deps.api);
                mock_init(&mut deps)?;

                let version: Version = CONTRACT_VERSION.parse().unwrap();

                let small_version = Version {
                    minor: version.minor - 1,
                    ..version.clone()
                }
                .to_string();
                cw2::set_contract_version(deps.as_mut().storage, REGISTRY, small_version)?;

                let res = crate::migrate::migrate(deps.as_mut(), env, MigrateMsg::Migrate {})?;
                assert_that!(res.messages).has_length(0);

                assert_that!(cw2::get_contract_version(&deps.storage)?.version)
                    .is_equal_to(version.to_string());
                Ok(())
            }
        }

        mod instantiate {
            use abstract_std::{
                objects::{
                    namespace::{Namespace, ABSTRACT_NAMESPACE},
                    ABSTRACT_ACCOUNT_ID,
                },
                registry::state::LOCAL_ACCOUNT_SEQUENCE,
            };
            use abstract_testing::prelude::AbstractMockAddrs;
            use contract::{VCResult, VcResponse};
            use registry::state::NAMESPACES_INFO;

            use super::*;

            #[test]
            fn sets_abstract_namespace() -> VCResult<()> {
                let mut deps = mock_dependencies();
                let abstr = AbstractMockAddrs::new(deps.api);
                let info = message_info(&abstr.owner, &[]);
                let env = mock_env_validated(deps.api);
                let admin = info.sender.to_string();

                let resp = super::super::instantiate(
                    deps.as_mut(),
                    env,
                    info.clone(),
                    registry::InstantiateMsg {
                        admin,
                        security_disabled: Some(true),
                        namespace_registration_fee: None,
                    },
                )?;

                let account_id = NAMESPACES_INFO.load(
                    deps.as_ref().storage,
                    &Namespace::try_from(ABSTRACT_NAMESPACE)?,
                )?;

                assert_that!(account_id).is_equal_to(ABSTRACT_ACCOUNT_ID);
                assert_eq!(resp, VcResponse::action("instantiate"));
                assert_eq!(LOCAL_ACCOUNT_SEQUENCE.load(&deps.storage).unwrap(), 0);

                Ok(())
            }
        }
    }
}
