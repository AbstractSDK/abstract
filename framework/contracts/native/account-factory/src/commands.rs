use abstract_core::objects::account::AccountTrace;
use abstract_core::objects::{AccountId, ABSTRACT_ACCOUNT_ID};
use abstract_core::{manager::ExecuteMsg, objects::module::assert_module_data_validity};
use cosmwasm_std::{
    ensure_eq, to_binary, wasm_execute, Addr, CosmosMsg, DepsMut, Empty, Env, MessageInfo,
    QuerierWrapper, ReplyOn, StdError, SubMsg, SubMsgResult, WasmMsg,
};
use protobuf::Message;

use abstract_sdk::{
    core::{
        manager::{InstantiateMsg as ManagerInstantiateMsg, InternalConfigAction},
        objects::{
            gov_type::GovernanceDetails, module::Module, module::ModuleInfo,
            module_reference::ModuleReference,
        },
        proxy::{ExecuteMsg as ProxyExecMsg, InstantiateMsg as ProxyInstantiateMsg},
        version_control::{
            AccountBase, ExecuteMsg as VCExecuteMsg, ModulesResponse, QueryMsg as VCQuery,
        },
        AbstractResult, MANAGER, PROXY,
    },
    cw_helpers::wasm_smart_query,
};

use crate::contract::AccountFactoryResponse;
use crate::{
    contract::AccountFactoryResult, error::AccountFactoryError,
    response::MsgInstantiateContractResponse, state::*,
};

pub const CREATE_ACCOUNT_MANAGER_MSG_ID: u64 = 1u64;
pub const CREATE_ACCOUNT_PROXY_MSG_ID: u64 = 2u64;

/// Function that starts the creation of the Account
#[allow(clippy::too_many_arguments)]
pub fn execute_create_account(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    governance: GovernanceDetails<Addr>,
    name: String,
    description: Option<String>,
    link: Option<String>,
    account_id: Option<AccountId>,
) -> AccountFactoryResult {
    let config = CONFIG.load(deps.storage)?;

    // If an origin is provided, assert the caller is the ibc host and return the account_id.
    // Else get the next account id and set the origin to local.
    let account_id = if let Some(account_id) = account_id {
        // if the account_id is provided, assert that the caller is the ibc host
        let ibc_host = config.ibc_host.ok_or(AccountFactoryError::IbcHostNotSet)?;
        ensure_eq!(
            info.sender,
            ibc_host,
            AccountFactoryError::SenderNotIbcHost(info.sender.into(), ibc_host.into())
        );
        // then assert that the account trace is remote and properly formatted
        account_id.trace().verify_remote()?;
        account_id
    } else {
        // else the call is local so we need to look up the account sequence
        // and set the origin to local
        let origin = AccountTrace::Local;

        // load the next account id
        // if it doesn't exist then it's the first account so set it to 0.
        let next_sequence = LOCAL_ACCOUNT_SEQUENCE.may_load(deps.storage)?.unwrap_or(0);

        // Check if the caller is the owner when instantiating the abstract account
        if next_sequence == ABSTRACT_ACCOUNT_ID.seq() {
            cw_ownable::assert_owner(deps.storage, &info.sender)?;
        }
        AccountId::new(next_sequence, origin)?
    };

    // Query version_control for code_id of Manager contract
    let module: Module = query_module(&deps.querier, &config.version_control_contract, MANAGER)?;

    // save module for after-init check
    CONTEXT.save(
        deps.storage,
        &Context {
            account_id: account_id.clone(),
            account_manager_address: None,
            manager_module: Some(module.clone()),
            proxy_module: None,
        },
    )?;

    if let ModuleReference::AccountBase(manager_code_id) = module.reference {
        Ok(AccountFactoryResponse::new(
            "create_account",
            vec![
                ("account_sequence", &account_id.seq().to_string()),
                ("trace", &account_id.trace().to_string()),
            ],
        )
        // Create manager
        .add_submessage(SubMsg {
            id: CREATE_ACCOUNT_MANAGER_MSG_ID,
            gas_limit: None,
            msg: WasmMsg::Instantiate {
                code_id: manager_code_id,
                funds: vec![],
                // Currently set admin to self, update later when we know the contract's address.
                admin: Some(env.contract.address.to_string()),
                // guarantee uniqueness of label
                label: format!("Abstract Account: {}", account_id),
                msg: to_binary(&ManagerInstantiateMsg {
                    account_id,
                    version_control_address: config.version_control_contract.to_string(),
                    module_factory_address: config.module_factory_address.to_string(),
                    name,
                    description,
                    link,
                    owner: governance.into(),
                })?,
            }
            .into(),
            reply_on: ReplyOn::Success,
        }))
    } else {
        Err(AccountFactoryError::WrongModuleKind(
            module.info.to_string(),
            "account_base".to_string(),
        ))
    }
}

/// instantiates the Proxy contract of the newly created Account
pub fn after_manager_create_proxy(deps: DepsMut, result: SubMsgResult) -> AccountFactoryResult {
    let config = CONFIG.load(deps.storage)?;

    // Get address of Manager contract
    let res: MsgInstantiateContractResponse =
        Message::parse_from_bytes(result.unwrap().data.unwrap().as_slice()).map_err(|_| {
            StdError::parse_err("MsgInstantiateContractResponse", "failed to parse data")
        })?;
    let manager_address = deps.api.addr_validate(res.get_contract_address())?;

    // Query version_control for code_id of proxy
    let module: Module = query_module(&deps.querier, &config.version_control_contract, PROXY)?;

    // Update the manager address and proxy module in the context.
    let context = CONTEXT.update(deps.storage, |c| {
        Result::<_, StdError>::Ok(Context {
            account_manager_address: Some(manager_address.clone()),
            proxy_module: Some(module.clone()),
            ..c
        })
    })?;

    if let ModuleReference::AccountBase(proxy_code_id) = module.reference {
        Ok(AccountFactoryResponse::new(
            "create_manager",
            vec![("manager_address", manager_address.to_string())],
        )
        // Instantiate proxy contract
        .add_submessage(SubMsg {
            id: CREATE_ACCOUNT_PROXY_MSG_ID,
            gas_limit: None,
            msg: WasmMsg::Instantiate {
                code_id: proxy_code_id,
                funds: vec![],
                admin: Some(manager_address.to_string()),
                label: format!("Proxy of Account: {}", context.account_id),
                msg: to_binary(&ProxyInstantiateMsg {
                    account_id: context.account_id,
                    ans_host_address: config.ans_host_contract.to_string(),
                })?,
            }
            .into(),
            reply_on: ReplyOn::Success,
        }))
    } else {
        Err(AccountFactoryError::WrongModuleKind(
            module.info.to_string(),
            "app".to_string(),
        ))
    }
}

fn query_module(
    querier: &QuerierWrapper,
    version_control_addr: &Addr,
    module_id: &str,
) -> AbstractResult<Module> {
    let ModulesResponse { mut modules } = querier.query(&wasm_smart_query(
        version_control_addr.to_string(),
        &VCQuery::Modules {
            infos: vec![ModuleInfo::from_id_latest(module_id)?],
        },
    )?)?;

    Ok(modules.swap_remove(0).module)
}

/// Registers the Account on the version_control contract and
/// adds proxy contract address to Manager
pub fn after_proxy_add_to_manager_and_set_admin(
    deps: DepsMut,
    result: SubMsgResult,
) -> AccountFactoryResult {
    let config = CONFIG.load(deps.storage)?;
    let context = CONTEXT.load(deps.storage)?;
    CONTEXT.remove(deps.storage);

    let res: MsgInstantiateContractResponse =
        Message::parse_from_bytes(result.unwrap().data.unwrap().as_slice()).map_err(|_| {
            StdError::parse_err("MsgInstantiateContractResponse", "failed to parse data")
        })?;

    let proxy_address = deps.api.addr_validate(res.get_contract_address())?;
    let manager_address = context
        .account_manager_address
        .expect("manager address set in context");
    let account_id = context.account_id;

    // assert proxy and manager contract information is correct
    assert_module_data_validity(
        &deps.querier,
        &context
            .manager_module
            .expect("manager module set in context"),
        Some(manager_address.clone()),
    )?;
    assert_module_data_validity(
        &deps.querier,
        &context.proxy_module.expect("proxy module set in context"),
        Some(proxy_address.clone()),
    )?;

    // construct Account base
    let account_base = AccountBase {
        manager: manager_address.clone(),
        proxy: proxy_address.clone(),
    };

    // Add Account base to version_control
    let add_account_to_version_control_msg: CosmosMsg<Empty> = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.version_control_contract.to_string(),
        funds: vec![],
        msg: to_binary(&VCExecuteMsg::AddAccount {
            account_id: account_id.clone(),
            account_base,
        })?,
    });

    // add manager to whitelisted addresses
    let whitelist_manager: CosmosMsg<Empty> = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: proxy_address.to_string(),
        funds: vec![],
        msg: to_binary(&ProxyExecMsg::AddModule {
            module: manager_address.to_string(),
        })?,
    });

    let set_proxy_admin_msg: CosmosMsg<Empty> = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: proxy_address.to_string(),
        funds: vec![],
        msg: to_binary(&ProxyExecMsg::SetAdmin {
            admin: manager_address.to_string(),
        })?,
    });

    let set_manager_admin_msg: CosmosMsg<Empty> = CosmosMsg::Wasm(WasmMsg::UpdateAdmin {
        contract_addr: manager_address.to_string(),
        admin: manager_address.to_string(),
    });

    // Add 1 to account sequence for local origin
    if account_id.is_local() {
        LOCAL_ACCOUNT_SEQUENCE.save(deps.storage, &account_id.seq().checked_add(1).unwrap())?;
    }

    let add_proxy_address_msg = wasm_execute(
        manager_address.to_string(),
        &ExecuteMsg::UpdateInternalConfig(
            to_binary(&InternalConfigAction::UpdateModuleAddresses {
                to_add: Some(vec![(PROXY.to_string(), proxy_address.to_string())]),
                to_remove: None,
            })
            .unwrap(),
        ),
        vec![],
    )?;

    Ok(AccountFactoryResponse::new(
        "create_proxy",
        vec![("proxy_address", res.get_contract_address())],
    )
    .add_message(add_account_to_version_control_msg)
    .add_message(add_proxy_address_msg)
    .add_message(whitelist_manager)
    .add_message(set_proxy_admin_msg)
    .add_message(set_manager_admin_msg))
}

// Only owner can execute it
#[allow(clippy::too_many_arguments)]
pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    ans_host_contract: Option<String>,
    version_control_contract: Option<String>,
    module_factory_address: Option<String>,
    ibc_host: Option<String>,
) -> AccountFactoryResult {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    let mut config: Config = CONFIG.load(deps.storage)?;

    if let Some(ans_host_contract) = ans_host_contract {
        // validate address format
        config.ans_host_contract = deps.api.addr_validate(&ans_host_contract)?;
    }

    if let Some(version_control_contract) = version_control_contract {
        // validate address format
        config.version_control_contract = deps.api.addr_validate(&version_control_contract)?;
    }

    if let Some(module_factory_address) = module_factory_address {
        // validate address format
        config.module_factory_address = deps.api.addr_validate(&module_factory_address)?;
    }

    if let Some(ibc_host) = ibc_host {
        // validate address format
        config.ibc_host = Some(deps.api.addr_validate(&ibc_host)?);
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(AccountFactoryResponse::action("update_config"))
}
