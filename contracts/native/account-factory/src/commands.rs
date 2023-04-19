use cosmwasm_std::{
    to_binary, wasm_execute, Addr, CosmosMsg, DepsMut, Empty, Env, MessageInfo, QuerierWrapper,
    ReplyOn, StdError, SubMsg, SubMsgResult, WasmMsg,
};
use protobuf::Message;

use abstract_sdk::core::{MANAGER, PROXY};
use abstract_sdk::{
    core::{
        manager::{ExecuteMsg::UpdateModuleAddresses, InstantiateMsg as ManagerInstantiateMsg},
        objects::{
            gov_type::GovernanceDetails, module::Module, module::ModuleInfo,
            module_reference::ModuleReference,
        },
        proxy::{ExecuteMsg as ProxyExecMsg, InstantiateMsg as ProxyInstantiateMsg},
        version_control::{
            AccountBase, ExecuteMsg as VCExecuteMsg, ModulesResponse, QueryMsg as VCQuery,
        },
        AbstractResult,
    },
    cw_helpers::cosmwasm_std::wasm_smart_query,
};

use crate::contract::AccountFactoryResponse;
use crate::{
    contract::AccountFactoryResult, error::AccountFactoryError,
    response::MsgInstantiateContractResponse, state::*,
};

pub const CREATE_ACCOUNT_MANAGER_MSG_ID: u64 = 1u64;
pub const CREATE_ACCOUNT_PROXY_MSG_ID: u64 = 2u64;

/// Function that starts the creation of the Account
pub fn execute_create_account(
    deps: DepsMut,
    env: Env,
    governance: GovernanceDetails<Addr>,
    name: String,
    description: Option<String>,
    link: Option<String>,
) -> AccountFactoryResult {
    let config = CONFIG.load(deps.storage)?;

    // Query version_control for code_id of Manager contract
    let module: Module = query_module(&deps.querier, &config.version_control_contract, MANAGER)?;

    if let ModuleReference::AccountBase(manager_code_id) = module.reference {
        Ok(AccountFactoryResponse::new(
            "create_account",
            vec![("account_id", &config.next_account_id.to_string())],
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
                label: format!("Abstract Account: {}", config.next_account_id),
                msg: to_binary(&ManagerInstantiateMsg {
                    account_id: config.next_account_id,
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
            "core".to_string(),
        ))
    }
}

/// instantiates the Treasury contract of the newly created DAO
pub fn after_manager_create_proxy(deps: DepsMut, result: SubMsgResult) -> AccountFactoryResult {
    let config = CONFIG.load(deps.storage)?;

    // Get address of Manager contract
    let res: MsgInstantiateContractResponse =
        Message::parse_from_bytes(result.unwrap().data.unwrap().as_slice()).map_err(|_| {
            StdError::parse_err("MsgInstantiateContractResponse", "failed to parse data")
        })?;
    let manager_address = res.get_contract_address();

    CONTEXT.save(
        deps.storage,
        &Context {
            account_manager_address: deps.api.addr_validate(manager_address)?,
        },
    )?;

    // Query version_control for code_id of proxy
    let module: Module = query_module(&deps.querier, &config.version_control_contract, PROXY)?;

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
                label: format!("Proxy of Account: {}", config.next_account_id),
                msg: to_binary(&ProxyInstantiateMsg {
                    account_id: config.next_account_id,
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

    Ok(modules.swap_remove(0))
}

/// Registers the DAO on the version_control contract and
/// adds proxy contract address to Manager
pub fn after_proxy_add_to_manager_and_set_admin(
    deps: DepsMut,
    result: SubMsgResult,
) -> AccountFactoryResult {
    let mut config = CONFIG.load(deps.storage)?;
    let context = CONTEXT.load(deps.storage)?;

    let res: MsgInstantiateContractResponse =
        Message::parse_from_bytes(result.unwrap().data.unwrap().as_slice()).map_err(|_| {
            StdError::parse_err("MsgInstantiateContractResponse", "failed to parse data")
        })?;

    let proxy_address = res.get_contract_address();

    // construct Account base
    let account_base = AccountBase {
        manager: context.account_manager_address.clone(),
        proxy: deps.api.addr_validate(proxy_address)?,
    };

    // Add Account base to version_control
    let add_account_to_version_control_msg: CosmosMsg<Empty> = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.version_control_contract.to_string(),
        funds: vec![],
        msg: to_binary(&VCExecuteMsg::AddAccount {
            account_id: config.next_account_id,
            account_base,
        })?,
    });

    // add manager to whitelisted addresses
    let whitelist_manager: CosmosMsg<Empty> = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: proxy_address.to_string(),
        funds: vec![],
        msg: to_binary(&ProxyExecMsg::AddModule {
            module: context.account_manager_address.to_string(),
        })?,
    });

    let set_proxy_admin_msg: CosmosMsg<Empty> = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: proxy_address.to_string(),
        funds: vec![],
        msg: to_binary(&ProxyExecMsg::SetAdmin {
            admin: context.account_manager_address.to_string(),
        })?,
    });

    let set_manager_admin_msg: CosmosMsg<Empty> = CosmosMsg::Wasm(WasmMsg::UpdateAdmin {
        contract_addr: context.account_manager_address.to_string(),
        admin: context.account_manager_address.to_string(),
    });

    // Update id sequence
    config.next_account_id += 1;
    CONFIG.save(deps.storage, &config)?;

    Ok(AccountFactoryResponse::new(
        "create_proxy",
        vec![("proxy_address", res.get_contract_address())],
    )
    .add_message(add_account_to_version_control_msg)
    .add_message(wasm_execute(
        context.account_manager_address.to_string(),
        &UpdateModuleAddresses {
            to_add: Some(vec![(PROXY.to_string(), proxy_address.to_string())]),
            to_remove: None,
        },
        vec![],
    )?)
    .add_message(whitelist_manager)
    .add_message(set_proxy_admin_msg)
    .add_message(set_manager_admin_msg))
}

// Only owner can execute it
#[allow(clippy::too_many_arguments)]
pub fn execute_update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    ans_host_contract: Option<String>,
    version_control_contract: Option<String>,
    module_factory_address: Option<String>,
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

    CONFIG.save(deps.storage, &config)?;

    Ok(AccountFactoryResponse::action("update_config"))
}
