use cosmwasm_std::{
    to_binary, Addr, Binary, CosmosMsg, DepsMut, Empty, Env, MessageInfo, QueryRequest, ReplyOn,
    Response, StdError, StdResult, SubMsg, SubMsgResult, WasmMsg, WasmQuery,
};

use abstract_os::manager::ExecuteMsg as ManagerMsg;
use abstract_os::modules::{Module, ModuleInfo, ModuleInitMsg, ModuleKind};
use abstract_sdk::version_control::verify_os_manager;

use cw2::ContractVersion;
use protobuf::Message;

use crate::contract::ModuleFactoryResult;

use crate::error::ModuleFactoryError;
use crate::response::MsgInstantiateContractResponse;
use crate::state::*;

use abstract_os::version_control::{
    QueryApiAddressResponse, QueryCodeIdResponse, QueryMsg as VCQuery,
};

pub const CREATE_INTERNAL_DAPP_RESPONSE_ID: u64 = 1u64;
pub const CREATE_EXTERNAL_DAPP_RESPONSE_ID: u64 = 2u64;
pub const CREATE_SERVICE_RESPONSE_ID: u64 = 3u64;
pub const CREATE_PERK_RESPONSE_ID: u64 = 4u64;

/// Function that starts the creation of the OS
pub fn execute_create_module(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    mut module: Module,
    root_init_msg: Option<Binary>,
) -> ModuleFactoryResult {
    let config = CONFIG.load(deps.storage)?;

    // Verify sender is active OS manager
    let core = verify_os_manager(&deps.querier, &info.sender, &config.version_control_address)?;

    if module.kind == ModuleKind::API {
        // Query version_control for api address
        let api_addr_response: QueryApiAddressResponse =
            deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: config.version_control_address.to_string(),
                msg: to_binary(&VCQuery::QueryApiAddress {
                    module: module.info.clone(),
                })?,
            }))?;

        module.info.version = Some(api_addr_response.info.version);

        let register_msg: CosmosMsg<Empty> = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: core.manager.into_string(),
            funds: vec![],
            msg: to_binary(&ManagerMsg::RegisterModule {
                module_addr: api_addr_response.address.to_string(),
                module,
            })?,
        });
        return Ok(Response::new().add_message(register_msg));
    }

    // Query version_control for code_id Module
    let module_code_id_response: QueryCodeIdResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.version_control_address.to_string(),
            msg: to_binary(&VCQuery::QueryCodeId {
                module: module.info,
            })?,
        }))?;

    // Update module info
    module.info = ModuleInfo::from(module_code_id_response.info.clone());
    // Get factory binary
    let ContractVersion { contract, version } = &module_code_id_response.info;

    // Todo: check if this can be generalised for some contracts
    // aka have default values for each kind of module that only get overwritten if a specific init_msg is saved.
    let fixed_binary = MODULE_INIT_BINARIES.may_load(deps.storage, (contract, version))?;
    let init_msg = ModuleInitMsg {
        fixed_init: fixed_binary,
        root_init: root_init_msg,
    }
    .format()?;

    // Set context for after init
    CONTEXT.save(
        deps.storage,
        &Context {
            core: Some(core.clone()),
            module: Some(module.clone()),
        },
    )?;

    // Match Module type
    match module {
        Module {
            kind: ModuleKind::AddOn,
            ..
        } => create_add_on(
            deps,
            env,
            module_code_id_response.code_id.u64(),
            init_msg,
            module,
            core.manager,
        ),
        Module {
            kind: ModuleKind::Service,
            ..
        } => create_service(
            deps,
            env,
            module_code_id_response.code_id.u64(),
            init_msg,
            module,
        ),
        Module {
            kind: ModuleKind::Perk,
            ..
        } => create_perk(
            deps,
            env,
            module_code_id_response.code_id.u64(),
            init_msg,
            module,
        ),
        _ => Err(ModuleFactoryError::Std(StdError::generic_err(
            "don't enter here!",
        ))),
    }
}

pub fn create_add_on(
    _deps: DepsMut,
    _env: Env,
    code_id: u64,
    init_msg: Binary,
    module: Module,
    manager: Addr,
) -> ModuleFactoryResult {
    let response = Response::new();

    Ok(response
        .add_attributes(vec![
            ("action", "create internal dapp"),
            ("initmsg:", &init_msg.to_string()),
        ])
        // Create manager
        .add_submessage(SubMsg {
            id: CREATE_INTERNAL_DAPP_RESPONSE_ID,
            gas_limit: None,
            msg: WasmMsg::Instantiate {
                code_id,
                funds: vec![],
                // This contract should be able to migrate the contract
                admin: Some(manager.to_string()),
                label: format!("Module: --{}--", module),
                msg: init_msg,
            }
            .into(),
            reply_on: ReplyOn::Success,
        }))
}

pub fn create_perk(
    _deps: DepsMut,
    _env: Env,
    code_id: u64,
    init_msg: Binary,
    module: Module,
) -> ModuleFactoryResult {
    let response = Response::new();

    Ok(response
        .add_attributes(vec![
            ("action", "create perk"),
            ("initmsg:", &init_msg.to_string()),
        ])
        // Create manager
        .add_submessage(SubMsg {
            id: CREATE_PERK_RESPONSE_ID,
            gas_limit: None,
            msg: WasmMsg::Instantiate {
                code_id,
                funds: vec![],
                // Not migratable
                admin: None,
                label: format!("Module: --{}--", module),
                msg: init_msg,
            }
            .into(),
            reply_on: ReplyOn::Success,
        }))
}

pub fn create_service(
    _deps: DepsMut,
    env: Env,
    code_id: u64,
    init_msg: Binary,
    module: Module,
) -> ModuleFactoryResult {
    let response = Response::new();

    Ok(response
        .add_attributes(vec![
            ("action", "create service"),
            ("initmsg:", &init_msg.to_string()),
        ])
        // Create manager
        .add_submessage(SubMsg {
            id: CREATE_SERVICE_RESPONSE_ID,
            gas_limit: None,
            msg: WasmMsg::Instantiate {
                code_id,
                funds: vec![],
                // This contract should be able to migrate the contract
                admin: Some(env.contract.address.to_string()),
                label: format!("Module: --{}--", module),
                msg: init_msg,
            }
            .into(),
            reply_on: ReplyOn::Success,
        }))
}

pub fn handle_add_on_init_result(deps: DepsMut, result: SubMsgResult) -> ModuleFactoryResult {
    let context: Context = CONTEXT.load(deps.storage)?;
    // Get address of Manager contract
    let res: MsgInstantiateContractResponse =
        Message::parse_from_bytes(result.unwrap().data.unwrap().as_slice()).map_err(|_| {
            StdError::parse_err("MsgInstantiateContractResponse", "failed to parse data")
        })?;
    let dapp_address = res.get_contract_address();

    let register_msg: CosmosMsg<Empty> = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: context.core.unwrap().manager.into_string(),
        funds: vec![],
        msg: to_binary(&ManagerMsg::RegisterModule {
            module_addr: dapp_address.to_string(),
            module: context.module.unwrap(),
        })?,
    });

    clear_context(deps)?;

    Ok(
        Response::new()
            .add_attribute("new module:", &dapp_address.to_string())
            .add_message(register_msg), // Instantiate Treasury contract
    )
}

pub fn handle_api_init_result(deps: DepsMut, result: SubMsgResult) -> ModuleFactoryResult {
    let context: Context = CONTEXT.load(deps.storage)?;
    // Get address of Manager contract
    let res: MsgInstantiateContractResponse =
        Message::parse_from_bytes(result.unwrap().data.unwrap().as_slice()).map_err(|_| {
            StdError::parse_err("MsgInstantiateContractResponse", "failed to parse data")
        })?;
    let dapp_address = res.get_contract_address();

    let register_msg: CosmosMsg<Empty> = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: context.core.unwrap().manager.into_string(),
        funds: vec![],
        msg: to_binary(&ManagerMsg::RegisterModule {
            module_addr: dapp_address.to_string(),
            module: context.module.unwrap(),
        })?,
    });

    clear_context(deps)?;

    Ok(Response::new()
        .add_attribute("new module:", &dapp_address.to_string())
        .add_message(register_msg))
}

// Only owner can execute it
pub fn execute_update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    admin: Option<String>,
    memory_address: Option<String>,
    version_control_address: Option<String>,
) -> ModuleFactoryResult {
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    let mut config: Config = CONFIG.load(deps.storage)?;

    if let Some(memory_address) = memory_address {
        // validate address format
        config.memory_address = deps.api.addr_validate(&memory_address)?;
    }

    if let Some(version_control_address) = version_control_address {
        // validate address format
        config.version_control_address = deps.api.addr_validate(&version_control_address)?;
    }

    CONFIG.save(deps.storage, &config)?;

    if let Some(admin) = admin {
        let addr = deps.api.addr_validate(&admin)?;
        ADMIN.set(deps, Some(addr))?;
    }

    Ok(Response::new().add_attribute("action", "update_config"))
}

// Only owner can execute it
pub fn update_factory_binaries(
    deps: DepsMut,
    info: MessageInfo,
    to_add: Vec<((String, String), Binary)>,
    to_remove: Vec<(String, String)>,
) -> ModuleFactoryResult {
    // Only Admin can call this method
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    for (key, binary) in to_add.into_iter() {
        // Update function for new or existing keys
        let insert = |_| -> StdResult<Binary> { Ok(binary) };
        MODULE_INIT_BINARIES.update(deps.storage, (&key.0, &key.1), insert)?;
    }

    for key in to_remove {
        MODULE_INIT_BINARIES.remove(deps.storage, (&key.0, &key.1));
    }
    Ok(Response::new().add_attribute("Action: ", "update binaries"))
}

fn clear_context(deps: DepsMut) -> Result<(), StdError> {
    // Set context for after init
    CONTEXT.save(
        deps.storage,
        &Context {
            core: None,
            module: None,
        },
    )
}
