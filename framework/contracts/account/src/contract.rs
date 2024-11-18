use abstract_macros::abstract_response;
use abstract_sdk::{
    feature_objects::RegistryContract,
    std::{
        account::state::ACCOUNT_ID,
        objects::validation::{validate_description, validate_link, validate_name},
        ACCOUNT,
    },
};
use abstract_std::{
    account::{
        state::{
            AccountInfo, WhitelistedModules, AUTH_ADMIN, INFO, SUSPENSION_STATUS,
            WHITELISTED_MODULES,
        },
        UpdateSubAccountAction,
    },
    module_factory::SimulateInstallModulesResponse,
    objects::{
        gov_type::GovernanceDetails,
        module_factory::ModuleFactoryContract,
        ownership::{self, GovOwnershipError},
        AccountId,
    },
    registry::state::LOCAL_ACCOUNT_SEQUENCE,
};

use cosmwasm_std::{
    ensure_eq, wasm_execute, Addr, Binary, Coins, Deps, DepsMut, Env, MessageInfo, Reply, Response,
    StdResult,
};

pub use crate::migrate::migrate;
use crate::{
    config::{update_account_status, update_info, update_internal_config},
    error::AccountError,
    execution::{
        add_auth_method, admin_execute, admin_execute_on_module, execute_msgs,
        execute_msgs_with_data, execute_on_module, ica_action, remove_auth_method,
    },
    modules::{
        _install_modules, install_modules,
        migration::{assert_modules_dependency_requirements, upgrade_modules},
        uninstall_module, MIGRATE_CONTEXT,
    },
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    queries::{
        handle_account_info_query, handle_config_query, handle_module_address_query,
        handle_module_info_query, handle_module_versions_query, handle_sub_accounts_query,
        handle_top_level_owner_query,
    },
    reply::{admin_action_reply, forward_response_reply, register_dependencies},
    sub_account::{
        create_sub_account, handle_sub_account_action, maybe_update_sub_account_governance,
        remove_account_from_contracts,
    },
};

#[abstract_response(ACCOUNT)]
pub struct AccountResponse;

pub type AccountResult<R = Response> = Result<R, AccountError>;

pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const FORWARD_RESPONSE_REPLY_ID: u64 = 1;
pub const ADMIN_ACTION_REPLY_ID: u64 = 2;
pub const REGISTER_MODULES_DEPENDENCIES_REPLY_ID: u64 = 3;
pub const ASSERT_MODULE_DEPENDENCIES_REQUIREMENTS_REPLY_ID: u64 = 4;

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    #[cfg_attr(not(feature = "xion"), allow(unused_variables))] InstantiateMsg {
        code_id,
        account_id,
        owner,
        install_modules,
        name,
        description,
        link,
        namespace,
        authenticator,
    }: InstantiateMsg,
) -> AccountResult {
    // Use CW2 to set the contract version, this is needed for migrations
    cw2::set_contract_version(deps.storage, ACCOUNT, CONTRACT_VERSION)?;

    let registry = RegistryContract::new(deps.as_ref(), code_id)?;
    let module_factory = ModuleFactoryContract::new(deps.as_ref(), code_id)?;

    let account_id = match account_id {
        Some(account_id) => account_id,
        None => {
            AccountId::local(LOCAL_ACCOUNT_SEQUENCE.query(&deps.querier, registry.address.clone())?)
        }
    };

    let mut response = AccountResponse::new(
        "instantiate",
        vec![("account_id".to_owned(), account_id.to_string())],
    );

    ACCOUNT_ID.save(deps.storage, &account_id)?;
    WHITELISTED_MODULES.save(deps.storage, &WhitelistedModules(vec![]))?;

    // Verify info
    validate_description(description.as_deref())?;
    validate_link(link.as_deref())?;
    if let Some(name) = name.as_deref() {
        validate_name(name)?;
    }

    let account_info = AccountInfo {
        name,
        description,
        link,
    };

    if account_info.has_info() {
        INFO.save(deps.storage, &account_info)?;
    }
    MIGRATE_CONTEXT.save(deps.storage, &vec![])?;

    let governance = owner.clone().verify(deps.as_ref())?;
    match governance {
        // Check if the caller is the proposed owner account when creating a sub-account.
        // This prevents other users from creating sub-accounts for accounts they don't own.
        GovernanceDetails::SubAccount { account } => {
            ensure_eq!(
                info.sender,
                account,
                AccountError::SubAccountCreatorNotAccount {
                    caller: info.sender.into(),
                    account: account.into(),
                }
            )
        }
        GovernanceDetails::NFT {
            collection_addr,
            token_id,
        } => verify_nft_ownership(
            deps.as_ref(),
            info.sender.clone(),
            collection_addr,
            token_id,
        )?,
        GovernanceDetails::AbstractAccount { address } => {
            ensure_eq!(
                address,
                env.contract.address,
                AccountError::AbsAccInvalidAddr {
                    abstract_account: address.to_string(),
                    contract: env.contract.address.to_string()
                }
            );
            #[cfg(feature = "xion")]
            {
                let Some(mut add_auth) = authenticator else {
                    return Err(AccountError::AbsAccNoAuth {});
                };
                abstract_xion::execute::add_auth_method(deps.branch(), &env, &mut add_auth)?;

                response = response.add_event(
                    cosmwasm_std::Event::new("create_abstract_account").add_attributes(vec![
                        ("contract_address", env.contract.address.to_string()),
                        ("authenticator", cosmwasm_std::to_json_string(&add_auth)?),
                        ("authenticator_id", add_auth.get_id().to_string()),
                    ]),
                );
            }
            // No Auth possible - error
            #[cfg(not(feature = "xion"))]
            return Err(AccountError::AbsAccNoAuth {});
        }
        _ => (),
    };

    // Set owner
    let cw_gov_owner = ownership::initialize_owner(deps.branch(), owner.clone())?;

    SUSPENSION_STATUS.save(deps.storage, &false)?;

    response = response.add_attribute("owner".to_owned(), cw_gov_owner.owner.to_string());

    let funds_for_namespace_fee = if namespace.is_some() {
        registry
            .namespace_registration_fee(&deps.querier)?
            .into_iter()
            .collect()
    } else {
        vec![]
    };

    let mut total_fee = Coins::try_from(funds_for_namespace_fee.clone()).unwrap();

    response = response.add_message(wasm_execute(
        registry.address,
        &abstract_std::registry::ExecuteMsg::AddAccount {
            namespace,
            creator: info.sender.to_string(),
        },
        funds_for_namespace_fee,
    )?);

    // Register on account if it's sub-account
    if let GovernanceDetails::SubAccount { account } = cw_gov_owner.owner {
        response = response.add_message(wasm_execute(
            account,
            &ExecuteMsg::UpdateSubAccount(UpdateSubAccountAction::RegisterSubAccount {
                id: ACCOUNT_ID.load(deps.storage)?.seq(),
            }),
            vec![],
        )?);
    }

    if !install_modules.is_empty() {
        let simulate_resp: SimulateInstallModulesResponse = deps.querier.query_wasm_smart(
            module_factory.address,
            &abstract_std::module_factory::QueryMsg::SimulateInstallModules {
                modules: install_modules.iter().map(|m| m.module.clone()).collect(),
            },
        )?;

        simulate_resp.total_required_funds.iter().for_each(|funds| {
            total_fee.add(funds.clone()).unwrap();
        });

        // Install modules
        let (install_msgs, install_attribute) = _install_modules(
            deps.branch(),
            install_modules,
            simulate_resp.total_required_funds,
            code_id,
        )?;
        response = response
            .add_submessages(install_msgs)
            .add_attribute(install_attribute.key, install_attribute.value);
    }

    let mut total_received = Coins::try_from(info.funds.clone()).unwrap();

    for fee in total_fee.clone() {
        total_received.sub(fee).map_err(|_| {
            abstract_std::AbstractError::Fee(format!(
                "Invalid fee payment sent. Expected {}, sent {:?}",
                total_fee, info.funds
            ))
        })?;
    }

    Ok(response)
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn execute(mut deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> AccountResult {
    let response = match msg {
        ExecuteMsg::UpdateStatus {
            is_suspended: suspension_status,
        } => update_account_status(deps.branch(), info, suspension_status),
        msg => {
            // Block actions if account is suspended
            let is_suspended = SUSPENSION_STATUS.load(deps.storage)?;
            if is_suspended {
                return Err(AccountError::AccountSuspended {});
            }
            let mut deps = deps.branch();

            match msg {
                // ## Execution ##
                ExecuteMsg::Execute { msgs } => execute_msgs(deps, &info.sender, msgs),
                ExecuteMsg::AdminExecute { addr, msg } => {
                    let addr = deps.api.addr_validate(&addr)?;
                    admin_execute(deps, info, addr, msg)
                }
                ExecuteMsg::ExecuteWithData { msg } => {
                    execute_msgs_with_data(deps, &info.sender, msg)
                }
                ExecuteMsg::ExecuteOnModule {
                    module_id,
                    exec_msg,
                    funds,
                } => execute_on_module(deps, info, module_id, exec_msg, funds),
                ExecuteMsg::AdminExecuteOnModule { module_id, msg } => {
                    admin_execute_on_module(deps, info, module_id, msg)
                }
                ExecuteMsg::IcaAction { action_query_msg } => {
                    ica_action(deps, info, action_query_msg)
                }

                // ## Configuration ##
                ExecuteMsg::UpdateInternalConfig(config) => {
                    update_internal_config(deps, info, config)
                }
                ExecuteMsg::InstallModules { modules } => {
                    install_modules(deps, &env, info, modules)
                }
                ExecuteMsg::UninstallModule { module_id } => {
                    uninstall_module(deps, &env, info, module_id)
                }
                ExecuteMsg::Upgrade { modules } => upgrade_modules(deps, env, info, modules),
                ExecuteMsg::UpdateInfo {
                    name,
                    description,
                    link,
                } => update_info(deps, info, name, description, link),
                ExecuteMsg::UpdateOwnership(action) => {
                    // If sub-account related it may require some messages to be constructed beforehand
                    let msgs = match &action {
                        ownership::GovAction::TransferOwnership { .. } => vec![],
                        ownership::GovAction::AcceptOwnership => {
                            maybe_update_sub_account_governance(deps.branch())?
                        }
                        ownership::GovAction::RenounceOwnership => {
                            remove_account_from_contracts(deps.branch(), &env)?
                        }
                    };

                    let new_owner_attributes =
                        ownership::update_ownership(deps, &env.block, &info.sender, action)?
                            .into_attributes();
                    Ok(
                        AccountResponse::new("update_ownership", new_owner_attributes)
                            .add_messages(msgs),
                    )
                }

                // ## Sub-Accounts ##
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
                ExecuteMsg::UpdateSubAccount(action) => {
                    handle_sub_account_action(deps, &env, info, action)
                }

                // ## Other ##
                ExecuteMsg::UpdateStatus { is_suspended: _ } => {
                    unreachable!("Update status case is reached above")
                }
                ExecuteMsg::AddAuthMethod { add_authenticator } => {
                    add_auth_method(deps, env, add_authenticator)
                }
                #[allow(unused)]
                ExecuteMsg::RemoveAuthMethod { id } => remove_auth_method(deps, env, id),
            }
        }
    }?;
    AUTH_ADMIN.remove(deps.storage);
    Ok(response)
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> AccountResult {
    match msg.id {
        FORWARD_RESPONSE_REPLY_ID => forward_response_reply(msg),
        ADMIN_ACTION_REPLY_ID => admin_action_reply(deps),
        REGISTER_MODULES_DEPENDENCIES_REPLY_ID => register_dependencies(deps),
        ASSERT_MODULE_DEPENDENCIES_REQUIREMENTS_REPLY_ID => {
            assert_modules_dependency_requirements(deps)
        }

        _ => Err(AccountError::UnexpectedReply {}),
    }
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => handle_config_query(deps, &env),
        QueryMsg::ModuleVersions { ids } => handle_module_versions_query(deps, &env, ids),
        QueryMsg::ModuleAddresses { ids } => handle_module_address_query(deps, ids),
        QueryMsg::ModuleInfos { start_after, limit } => {
            handle_module_info_query(deps, &env, start_after, limit)
        }
        QueryMsg::Info {} => handle_account_info_query(deps),
        QueryMsg::SubAccountIds { start_after, limit } => {
            handle_sub_accounts_query(deps, start_after, limit)
        }
        QueryMsg::TopLevelOwner {} => handle_top_level_owner_query(deps, env),
        QueryMsg::Ownership {} => {
            cosmwasm_std::to_json_binary(&ownership::get_ownership(deps.storage)?)
        }
        #[cfg_attr(not(feature = "xion"), allow(unused_variables))]
        QueryMsg::AuthenticatorByID { id } => {
            #[cfg(feature = "xion")]
            return cosmwasm_std::to_json_binary(&abstract_xion::query::authenticator_by_id(
                deps.storage,
                id,
            )?);
            #[cfg(not(feature = "xion"))]
            Ok(Binary::default())
        }
        QueryMsg::AuthenticatorIDs {} => {
            #[cfg(feature = "xion")]
            return cosmwasm_std::to_json_binary(&abstract_xion::query::authenticator_ids(
                deps.storage,
            )?);
            #[cfg(not(feature = "xion"))]
            Ok(Binary::default())
        }
    }
}

#[cfg(feature = "xion")]
#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn sudo(
    deps: DepsMut,
    env: Env,
    msg: abstract_xion::contract::AccountSudoMsg,
) -> abstract_xion::error::ContractResult<Response> {
    if let abstract_xion::contract::AccountSudoMsg::BeforeTx { .. } = &msg {
        AUTH_ADMIN.save(deps.storage, &true)?;
    };
    abstract_xion::contract::sudo(deps, env, msg)
}

/// Verifies that *sender* is the owner of *nft_id* of contract *nft_addr*
fn verify_nft_ownership(
    deps: Deps,
    sender: Addr,
    nft_addr: Addr,
    nft_id: String,
) -> Result<(), AccountError> {
    // get owner of token_id from collection
    let owner: ownership::cw721::OwnerOfResponse = deps.querier.query_wasm_smart(
        nft_addr,
        &ownership::cw721::Cw721QueryMsg::OwnerOf {
            token_id: nft_id,
            include_expired: None,
        },
    )?;
    let owner = deps.api.addr_validate(&owner.owner)?;
    // verify owner
    ensure_eq!(
        sender,
        owner,
        AccountError::Ownership(GovOwnershipError::NotOwner)
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use abstract_std::{
        account,
        objects::{
            account::AccountTrace, gov_type::GovernanceDetails, ownership::GovOwnershipError,
            AccountId,
        },
    };
    use abstract_testing::{
        abstract_mock_querier, abstract_mock_querier_builder, mock_env_validated,
        prelude::AbstractMockAddrs,
    };
    use cosmwasm_std::{
        testing::{message_info, mock_dependencies},
        to_json_binary, wasm_execute, CosmosMsg, SubMsg,
    };

    use crate::error::AccountError;

    use super::verify_nft_ownership;

    #[coverage_helper::test]
    fn successful_instantiate() {
        let mut deps = mock_dependencies();
        deps.querier = abstract_mock_querier(deps.api);

        let abstr = AbstractMockAddrs::new(deps.api);
        let info = message_info(&abstr.owner, &[]);
        let env = mock_env_validated(deps.api);

        let resp = super::instantiate(
            deps.as_mut(),
            env,
            info,
            account::InstantiateMsg {
                code_id: 1,
                account_id: AccountId::new(1, AccountTrace::Local).ok(),
                owner: GovernanceDetails::Monarchy {
                    monarch: abstr.owner.to_string(),
                },
                namespace: None,
                name: Some("test".to_string()),
                description: None,
                link: None,
                install_modules: vec![],
                authenticator: None,
            },
        );

        assert!(resp.is_ok());

        let expected_msg: CosmosMsg = wasm_execute(
            abstr.registry,
            &abstract_std::registry::ExecuteMsg::AddAccount {
                creator: abstr.owner.to_string(),
                namespace: None,
            },
            vec![],
        )
        .unwrap()
        .into();

        assert_eq!(resp.unwrap().messages, vec![SubMsg::new(expected_msg)]);
    }

    #[coverage_helper::test]
    fn verify_nft() {
        let mut deps = mock_dependencies();
        let nft_addr = deps.api.addr_make("nft");
        deps.querier = abstract_mock_querier_builder(deps.api)
            .with_smart_handler(&nft_addr, move |_| {
                Ok(
                    to_json_binary(&abstract_std::objects::ownership::cw721::OwnerOfResponse {
                        owner: deps.api.addr_make("owner").to_string(),
                    })
                    .unwrap(),
                )
            })
            .build();

        // Owner
        let res = verify_nft_ownership(
            deps.as_ref(),
            deps.api.addr_make("owner"),
            nft_addr.clone(),
            "foo".to_owned(),
        );
        assert!(res.is_ok());

        // Not owner
        let res = verify_nft_ownership(
            deps.as_ref(),
            deps.api.addr_make("not_owner"),
            nft_addr,
            "foo".to_owned(),
        );
        assert_eq!(
            res,
            Err(AccountError::Ownership(GovOwnershipError::NotOwner))
        );
    }
}
