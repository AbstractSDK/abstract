use abstract_macros::abstract_response;
use abstract_sdk::{
    feature_objects::VersionControlContract,
    std::{
        account::state::ACCOUNT_ID,
        objects::validation::{validate_description, validate_link, validate_name},
        ACCOUNT,
    },
};
use abstract_std::{
    account::{
        state::{
            AccountInfo, Config, WhitelistedModules, CONFIG, INFO, SUSPENSION_STATUS,
            WHITELISTED_MODULES,
        },
        ExecuteMsg, InstantiateMsg, QueryMsg, UpdateSubAccountAction,
    },
    module_factory::SimulateInstallModulesResponse,
    objects::{
        gov_type::GovernanceDetails,
        ownership::{self, GovOwnershipError},
        AccountId,
    },
    version_control::state::LOCAL_ACCOUNT_SEQUENCE,
};
use cosmwasm_std::{
    ensure_eq, wasm_execute, Addr, Binary, Coins, Deps, DepsMut, Env, MessageInfo, Reply, Response,
    StdResult, SubMsgResult,
};

pub use crate::migrate::migrate;
use crate::{
    config::{update_account_status, update_info, update_internal_config},
    error::AccountError,
    execution::{
        execute_ibc_action, execute_module_action, execute_module_action_response, ica_action,
    },
    modules::{
        _install_modules, exec_on_module, install_modules,
        migration::{handle_callback, upgrade_modules},
        uninstall_module, MIGRATE_CONTEXT,
    },
    queries::{
        handle_account_info_query, handle_config_query, handle_module_address_query,
        handle_module_info_query, handle_module_versions_query, handle_sub_accounts_query,
        handle_top_level_owner_query,
    },
    reply::{forward_response_data, register_dependencies},
    sub_account::{
        create_sub_account, handle_sub_account_action, maybe_update_sub_account_governance,
        remove_account_from_contracts,
    },
};

#[abstract_response(ACCOUNT)]
pub struct AccountResponse;

pub type AccountResult<R = Response> = Result<R, AccountError>;

pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const RESPONSE_REPLY_ID: u64 = 1;
pub const REGISTER_MODULES_DEPENDENCIES_REPLY_ID: u64 = 2;

// Need wrapper because cosmwasm_std::entry_point counts arguments in any feature
#[cfg(feature = "xion")]
#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg<crate::absacc::auth::AddAuthenticator>,
) -> AccountResult {
    _instantiate(deps, env, info, msg)
}
#[cfg(not(feature = "xion"))]
#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg<cosmwasm_std::Empty>,
) -> AccountResult {
    _instantiate(deps, env, info, msg)
}

#[doc(hidden)]
pub fn _instantiate(
    mut deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    #[cfg(feature = "xion")] InstantiateMsg {
        account_id,
        owner,
        install_modules,
        name,
        description,
        link,
        module_factory_address,
        version_control_address,
        namespace,

        authenticator,
    }: InstantiateMsg<crate::absacc::auth::AddAuthenticator>,
    #[cfg(not(feature = "xion"))] InstantiateMsg {
        account_id,
        owner,
        install_modules,
        name,
        description,
        link,
        module_factory_address,
        version_control_address,
        namespace,

        authenticator,
    }: InstantiateMsg<cosmwasm_std::Empty>,
) -> AccountResult {
    // Use CW2 to set the contract version, this is needed for migrations
    cw2::set_contract_version(deps.storage, ACCOUNT, CONTRACT_VERSION)?;

    let module_factory_address = deps.api.addr_validate(&module_factory_address)?;
    let version_control_address = deps.api.addr_validate(&version_control_address)?;

    let account_id = match account_id {
        Some(account_id) => account_id,
        None => AccountId::local(
            LOCAL_ACCOUNT_SEQUENCE.query(&deps.querier, version_control_address.clone())?,
        ),
    };

    let mut response = AccountResponse::new(
        "instantiate",
        vec![("account_id".to_owned(), account_id.to_string())],
    );

    ACCOUNT_ID.save(deps.storage, &account_id)?;
    WHITELISTED_MODULES.save(deps.storage, &WhitelistedModules(vec![]))?;

    // Save config
    let config = Config {
        version_control_address: version_control_address.clone(),
        module_factory_address: module_factory_address.clone(),
    };
    abstract_std::account::state::CONFIG.save(deps.storage, &config)?;

    // Verify info
    validate_description(description.as_deref())?;
    validate_link(link.as_deref())?;
    validate_name(&name)?;

    let account_info = AccountInfo {
        name,
        description,
        link,
    };

    INFO.save(deps.storage, &account_info)?;
    MIGRATE_CONTEXT.save(deps.storage, &vec![])?;

    let governance = owner
        .clone()
        .verify(deps.as_ref(), version_control_address.clone())?;
    if let Some(mut add_auth) = authenticator {
        ensure_eq!(
            GovernanceDetails::Renounced {},
            governance,
            AccountError::GovernanceWithAuth {}
        );
        #[cfg(feature = "xion")]
        {
            crate::absacc::auth::execute::add_auth_method(deps.branch(), &_env, &mut add_auth)?;

            response = response.add_event(
                cosmwasm_std::Event::new("create_abstract_account").add_attributes(vec![
                    ("contract_address", _env.contract.address.to_string()),
                    ("authenticator", cosmwasm_std::to_json_string(&add_auth)?),
                    ("authenticator_id", add_auth.get_id().to_string()),
                ]),
            );
        }
    } else {
        match governance {
            // Check if the caller is the manager the proposed owner account when creating a sub-account.
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
            _ => (),
        };
    }

    // Set owner
    let cw_gov_owner = ownership::initialize_owner(
        deps.branch(),
        // TODO: support no owner here (ownership handled in SUDO)
        // Or do we want to add a `Sudo` governance type?
        owner.clone(),
        config.version_control_address.clone(),
    )?;

    SUSPENSION_STATUS.save(deps.storage, &false)?;

    response = response.add_attribute("owner".to_owned(), cw_gov_owner.owner.to_string());

    let version_control = VersionControlContract::new(config.version_control_address.clone());

    let funds_for_namespace_fee = if namespace.is_some() {
        version_control
            .namespace_registration_fee(&deps.querier)?
            .into_iter()
            .collect()
    } else {
        vec![]
    };

    let mut total_fee = Coins::try_from(funds_for_namespace_fee.clone()).unwrap();

    response = response.add_message(wasm_execute(
        version_control_address,
        &abstract_std::version_control::ExecuteMsg::AddAccount {
            namespace,
            creator: info.sender.to_string(),
        },
        funds_for_namespace_fee,
    )?);

    // Register on account if it's sub-account
    if let GovernanceDetails::SubAccount { account } = cw_gov_owner.owner {
        response = response.add_message(wasm_execute(
            account,
            &ExecuteMsg::UpdateSubAccount::<cosmwasm_std::Empty>(
                UpdateSubAccountAction::RegisterSubAccount {
                    id: ACCOUNT_ID.load(deps.storage)?.seq(),
                },
            ),
            vec![],
        )?);
    }

    if !install_modules.is_empty() {
        let simulate_resp: SimulateInstallModulesResponse = deps.querier.query_wasm_smart(
            config.module_factory_address.to_string(),
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
            config.module_factory_address,
            config.version_control_address.clone(),
            simulate_resp.total_required_funds,
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

// Need wrapper because cosmwasm_std::entry_point counts arguments in any feature
#[cfg(feature = "xion")]
#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg<crate::absacc::auth::AddAuthenticator>,
) -> AccountResult {
    _execute(deps, env, info, msg)
}
#[cfg(not(feature = "xion"))]
#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg<cosmwasm_std::Empty>,
) -> AccountResult {
    _execute(deps, env, info, msg)
}

#[doc(hidden)]
pub fn _execute(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    #[cfg(feature = "xion")] msg: ExecuteMsg<crate::absacc::auth::AddAuthenticator>,
    #[cfg(not(feature = "xion"))] msg: ExecuteMsg,
) -> AccountResult {
    match msg {
        ExecuteMsg::UpdateStatus {
            is_suspended: suspension_status,
        } => update_account_status(deps, info, suspension_status).map_err(AccountError::from),
        mut msg => {
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
                ExecuteMsg::UpdateStatus { is_suspended: _ } => {
                    unreachable!("Update status case is reached above")
                }
                ExecuteMsg::AddAuthMethod {
                    ref mut add_authenticator,
                } => {
                    #[cfg(feature = "xion")]
                    {
                        crate::absacc::auth::execute::add_auth_method(deps, &env, add_authenticator)
                    }
                    #[cfg(not(feature = "xion"))]
                    {
                        Ok(AccountResponse::action("add_auth"))
                    }
                }
                ExecuteMsg::RemoveAuthMethod { id } => {
                    #[cfg(feature = "xion")]
                    {
                        crate::absacc::auth::execute::remove_auth_method(deps, env, id)
                    }
                    #[cfg(not(feature = "xion"))]
                    {
                        Ok(AccountResponse::action("remove_auth"))
                    }
                }
                ExecuteMsg::Callback(_) => handle_callback(deps, env, info),
            }
        }
    }
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> AccountResult {
    match msg {
        Reply {
            id: RESPONSE_REPLY_ID,
            result: SubMsgResult::Ok(_),
            ..
        } => forward_response_data(msg),
        Reply {
            id: REGISTER_MODULES_DEPENDENCIES_REPLY_ID,
            result: SubMsgResult::Ok(_),
            ..
        } => register_dependencies(deps),
        _ => Err(AccountError::UnexpectedReply {}),
    }
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => handle_config_query(deps),
        QueryMsg::ModuleVersions { ids } => handle_module_versions_query(deps, ids),
        QueryMsg::ModuleAddresses { ids } => handle_module_address_query(deps, ids),
        QueryMsg::ModuleInfos { start_after, limit } => {
            handle_module_info_query(deps, start_after, limit)
        }
        QueryMsg::Info {} => handle_account_info_query(deps),
        QueryMsg::SubAccountIds { start_after, limit } => {
            handle_sub_accounts_query(deps, start_after, limit)
        }
        QueryMsg::TopLevelOwner {} => handle_top_level_owner_query(deps, env),
        QueryMsg::Ownership {} => {
            cosmwasm_std::to_json_binary(&ownership::get_ownership(deps.storage)?)
        }
        QueryMsg::AuthenticatorByID { id } => {
            #[cfg(feature = "xion")]
            {
                cosmwasm_std::to_json_binary(&crate::state::AUTHENTICATORS.load(deps.storage, id)?)
            }
            #[cfg(not(feature = "xion"))]
            Ok(Binary::default())
        }
        QueryMsg::AuthenticatorIDs {} => {
            #[cfg(feature = "xion")]
            {
                cosmwasm_std::to_json_binary(
                    &crate::state::AUTHENTICATORS
                        .keys(deps.storage, None, None, cosmwasm_std::Order::Ascending)
                        .collect::<Result<Vec<_>, _>>()?,
                )
            }
            #[cfg(not(feature = "xion"))]
            Ok(Binary::default())
        }
    }
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
        objects::{account::AccountTrace, gov_type::GovernanceDetails, AccountId},
    };
    use abstract_testing::prelude::AbstractMockAddrs;
    use cosmwasm_std::{
        testing::{message_info, mock_dependencies, mock_env},
        wasm_execute, CosmosMsg, SubMsg,
    };

    #[test]
    fn successful_instantiate() {
        let mut deps = mock_dependencies();

        let abstr = AbstractMockAddrs::new(deps.api);
        let info = message_info(&abstr.owner, &[]);

        let resp = super::instantiate(
            deps.as_mut(),
            mock_env(),
            info,
            account::InstantiateMsg {
                account_id: AccountId::new(1, AccountTrace::Local).ok(),
                owner: GovernanceDetails::Monarchy {
                    monarch: abstr.owner.to_string(),
                },
                version_control_address: abstr.version_control.to_string(),
                module_factory_address: abstr.module_factory.to_string(),
                namespace: None,
                name: "test".to_string(),
                description: None,
                link: None,
                install_modules: vec![],
                authenticator: None,
            },
        );

        assert!(resp.is_ok());

        let expected_msg: CosmosMsg = wasm_execute(
            abstr.version_control,
            &abstract_std::version_control::ExecuteMsg::AddAccount {
                creator: abstr.owner.to_string(),
                namespace: None,
            },
            vec![],
        )
        .unwrap()
        .into();

        assert_eq!(resp.unwrap().messages, vec![SubMsg::new(expected_msg)]);
    }
}
