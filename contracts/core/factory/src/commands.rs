use cosmwasm_std::{
    to_binary, Addr, DepsMut, Env, MessageInfo, ReplyOn, Response, StdError, SubMsg, WasmMsg,
};
use cosmwasm_std::{ContractResult, CosmosMsg, SubMsgExecutionResponse};
use dao_os::governance::gov_type::GovernanceDetails;
use dao_os::manager::helper::register_module_on_manager;
use protobuf::Message;

use crate::contract::OsFactoryResult;

use crate::response::MsgInstantiateContractResponse;

use crate::state::*;
use dao_os::manager::msg::InstantiateMsg as ManagerInstantiateMsg;
use dao_os::treasury::msg::InstantiateMsg as TreasuryInstantiateMsg;
use dao_os::version_control::msg::ExecuteMsg as VCExecuteMsg;
use dao_os::version_control::queries::query_code_id;

const TREASURY_VERSION: &str = "v0.1.0";
const MANAGER_VERSION: &str = "v0.1.0";

pub const MANAGER_CREATE_ID: u64 = 1u64;
pub const TREASURY_CREATE_ID: u64 = 2u64;

pub const TREASURY_NAME: &str = "Treasury";

// Only owner can execute it
pub fn execute_create_os(
    deps: DepsMut,
    env: Env,
    governance: GovernanceDetails,
) -> OsFactoryResult {
    // TODO: Add check if fee was paid

    // Get address of OS root account

    let root_user: Addr = match governance {
        GovernanceDetails::Monarchy { owner } => deps.api.addr_validate(&owner)?,
        _ => Err(StdError::generic_err("Not Implemented"))?,
    };

    let config = CONFIG.load(deps.storage)?;
    let response = Response::new();

    // Query version_control for code_id of Manager
    let manager_code_id = query_code_id(
        deps.as_ref(),
        &config.version_control_contract,
        String::from("Manager"),
        String::from(MANAGER_VERSION),
    )?;

    // Create manager
    Ok(response
        .add_attributes(vec![
            ("action", "create os"),
            ("os_id:", &config.os_id_sequence.to_string()),
        ])
        .add_submessage(SubMsg {
            id: MANAGER_CREATE_ID,
            gas_limit: None,
            msg: WasmMsg::Instantiate {
                code_id: manager_code_id,
                funds: vec![],
                // TODO: Review
                // This contract is able to upgrade the manager contract
                admin: Some(env.contract.address.to_string()),
                label: format!("CosmWasm OS: {}", config.os_id_sequence),
                msg: to_binary(&ManagerInstantiateMsg {
                    os_id: config.os_id_sequence,
                    root_user: root_user.to_string(),
                })?,
            }
            .into(),
            reply_on: ReplyOn::Success,
        }))
}

pub fn after_manager_create_treasury(
    deps: DepsMut,
    result: ContractResult<SubMsgExecutionResponse>,
) -> OsFactoryResult {
    let mut config = CONFIG.load(deps.storage)?;

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
            os_id: config.os_id_sequence,
            os_manager_address: manager_address.to_string(),
        })?,
    }));

    // Update id sequence
    config.os_id_sequence += 1;
    CONFIG.save(deps.storage, &config)?;

    let treasury_code_id = query_code_id(
        deps.as_ref(),
        &config.version_control_contract,
        String::from(TREASURY_NAME),
        String::from(TREASURY_VERSION),
    )?;

    Ok(response
        .add_attribute("Manager Address:", &manager_address.to_string())
        .add_submessage(SubMsg {
        id: TREASURY_CREATE_ID,
        gas_limit: None,
        msg: WasmMsg::Instantiate {
            code_id: treasury_code_id,
            funds: vec![],
            admin: Some(manager_address.to_string()),
            label: format!("Treasury of OS: {}", config.os_id_sequence - 1u32),
            msg: to_binary(&TreasuryInstantiateMsg {})?,
        }
        .into(),
        reply_on: ReplyOn::Success,
    }))
}

pub fn after_treasury_add_to_manager(
    env: Env,
    result: ContractResult<SubMsgExecutionResponse>,
) -> OsFactoryResult {
    let res: MsgInstantiateContractResponse =
        Message::parse_from_bytes(result.unwrap().data.unwrap().as_slice()).map_err(|_| {
            StdError::parse_err("MsgInstantiateContractResponse", "failed to parse data")
        })?;

    Ok(Response::new()
    .add_attribute("Treasury Address: ", res.get_contract_address())
    .add_message(register_module_on_manager(
        res.get_contract_address().to_string(),
        TREASURY_NAME.to_string(),
        env,
    )?))
}

// Only owner can execute it
pub fn execute_update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    admin: Option<String>,
    memory_contract: Option<String>,
    version_control_contract: Option<String>,
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
