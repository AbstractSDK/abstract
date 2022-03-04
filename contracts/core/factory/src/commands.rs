use cosmwasm_std::{
    to_binary, Addr, DepsMut, Env, MessageInfo, QueryRequest, ReplyOn, Response, StdError, SubMsg,
    WasmMsg, WasmQuery,
};
use cosmwasm_std::{ContractResult, CosmosMsg, SubMsgExecutionResponse};
use pandora::governance::gov_type::GovernanceDetails;
use pandora::manager::helper::register_module_on_manager;
use pandora::modules::ModuleInfo;
use protobuf::Message;

use crate::contract::OsFactoryResult;

use crate::response::MsgInstantiateContractResponse;

use crate::state::*;
use pandora::manager::msg::InstantiateMsg as ManagerInstantiateMsg;
use pandora::treasury::msg::InstantiateMsg as TreasuryInstantiateMsg;
use pandora::version_control::msg::{
    CodeIdResponse, ExecuteMsg as VCExecuteMsg, QueryMsg as VCQuery,
};

pub const CREATE_OS_MANAGER_MSG_ID: u64 = 1u64;
pub const CREATE_OS_TREASURY_MSG_ID: u64 = 2u64;
use pandora::registery::{MANAGER, TREASURY};

/// Function that starts the creation of the OS
pub fn execute_create_os(
    deps: DepsMut,
    env: Env,
    governance: GovernanceDetails,
) -> OsFactoryResult {
    // TODO: Add check if fee was paid

    // Get address of OS root user, depends on gov-type
    let root_user: Addr = match governance {
        GovernanceDetails::Monarchy { monarch } => deps.api.addr_validate(&monarch)?,
        _ => return Err(StdError::generic_err("Not Implemented").into()),
    };

    let config = CONFIG.load(deps.storage)?;
    let response = Response::new();

    // Query version_control for code_id of Manager contract
    let manager_code_id_response: CodeIdResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.version_control_contract.to_string(),
            msg: to_binary(&VCQuery::QueryCodeId {
                module: ModuleInfo {
                    name: String::from(MANAGER),
                    version: None,
                },
            })?,
        }))?;

    Ok(response
        .add_attributes(vec![
            ("action", "create os"),
            ("os_id:", &config.next_os_id.to_string()),
        ])
        // Create manager
        .add_submessage(SubMsg {
            id: CREATE_OS_MANAGER_MSG_ID,
            gas_limit: None,
            msg: WasmMsg::Instantiate {
                code_id: manager_code_id_response.code_id.u64(),
                funds: vec![],
                // TODO: Review
                // This contract is able to upgrade the manager contract
                admin: Some(env.contract.address.to_string()),
                label: format!("CosmWasm OS: {}", config.next_os_id),
                msg: to_binary(&ManagerInstantiateMsg {
                    os_id: config.next_os_id,
                    root_user: root_user.to_string(),
                    version_control_address: config.version_control_contract.to_string(),
                    module_factory_address: config.module_factory_address.to_string(),
                })?,
            }
            .into(),
            reply_on: ReplyOn::Success,
        }))
}

/// Registers the DAO on the version_control contract and
/// instantiates the Treasury contract of the newly created DAO
pub fn after_manager_create_treasury(
    deps: DepsMut,
    result: ContractResult<SubMsgExecutionResponse>,
) -> OsFactoryResult {
    let config = CONFIG.load(deps.storage)?;

    // Get address of Manager contract
    let res: MsgInstantiateContractResponse =
        Message::parse_from_bytes(result.unwrap().data.unwrap().as_slice()).map_err(|_| {
            StdError::parse_err("MsgInstantiateContractResponse", "failed to parse data")
        })?;
    let manager_address = res.get_contract_address();

    // Add OS to version_control
    let response = Response::new().add_message(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.version_control_contract.to_string(),
        funds: vec![],
        msg: to_binary(&VCExecuteMsg::AddOs {
            os_id: config.next_os_id,
            os_manager_address: manager_address.to_string(),
        })?,
    }));

    // Query version_control for code_id of Treasury
    // TODO: replace with raw-query from package.
    let treasury_code_id_response: CodeIdResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.version_control_contract.to_string(),
            msg: to_binary(&VCQuery::QueryCodeId {
                module: ModuleInfo {
                    name: String::from(TREASURY),
                    version: None,
                },
            })?,
        }))?;

    Ok(response
        .add_attribute("Manager Address:", &manager_address.to_string())
        // Instantiate Treasury contract
        .add_submessage(SubMsg {
            id: CREATE_OS_TREASURY_MSG_ID,
            gas_limit: None,
            msg: WasmMsg::Instantiate {
                code_id: treasury_code_id_response.code_id.u64(),
                funds: vec![],
                admin: Some(manager_address.to_string()),
                label: format!("Treasury of OS: {}", config.next_os_id),
                msg: to_binary(&TreasuryInstantiateMsg {})?,
            }
            .into(),
            reply_on: ReplyOn::Success,
        }))
}

/// Adds treasury contract address and name to Manager
/// contract of OS
pub fn after_treasury_add_to_manager(
    deps: DepsMut,
    result: ContractResult<SubMsgExecutionResponse>,
) -> OsFactoryResult {
    let mut config = CONFIG.load(deps.storage)?;

    let res: MsgInstantiateContractResponse =
        Message::parse_from_bytes(result.unwrap().data.unwrap().as_slice()).map_err(|_| {
            StdError::parse_err("MsgInstantiateContractResponse", "failed to parse data")
        })?;

    // TODO: Should we store the manager address in the local state between the previous step and this?
    // Get address of manager
    let manager_address: String = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: config.version_control_contract.to_string(),
        msg: to_binary(&VCQuery::QueryOsAddress {
            os_id: config.next_os_id,
        })?,
    }))?;

    // Update id sequence
    config.next_os_id += 1;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("Treasury Address: ", res.get_contract_address())
        .add_message(register_module_on_manager(
            manager_address,
            TREASURY.to_string(),
            res.get_contract_address().to_string(),
        )?))
}

// Only owner can execute it
#[allow(clippy::too_many_arguments)]
pub fn execute_update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    admin: Option<String>,
    memory_contract: Option<String>,
    version_control_contract: Option<String>,
    module_factory_address: Option<String>,
    creation_fee: Option<u32>,
) -> OsFactoryResult {
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    let mut config: Config = CONFIG.load(deps.storage)?;

    if let Some(memory_contract) = memory_contract {
        // validate address format
        config.memory_contract = deps.api.addr_validate(&memory_contract)?;
    }

    if let Some(version_control_contract) = version_control_contract {
        // validate address format
        config.version_control_contract = deps.api.addr_validate(&version_control_contract)?;
    }

    if let Some(module_factory_address) = module_factory_address {
        // validate address format
        config.module_factory_address = deps.api.addr_validate(&module_factory_address)?;
    }

    if let Some(creation_fee) = creation_fee {
        config.creation_fee = creation_fee;
    }

    CONFIG.save(deps.storage, &config)?;

    if let Some(admin) = admin {
        let addr = deps.api.addr_validate(&admin)?;
        ADMIN.set(deps, Some(addr))?;
    }

    Ok(Response::new().add_attribute("action", "update_config"))
}
