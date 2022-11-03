use cosmwasm_std::{
    to_binary, Addr, Binary, CosmosMsg, DepsMut, Empty, Env, MessageInfo, ReplyOn, Response,
    StdError, StdResult, SubMsg, SubMsgResult, WasmMsg,
};

use abstract_os::{
    manager::ExecuteMsg as ManagerMsg,
    objects::{module::ModuleInfo, module_reference::ModuleReference},
};
use abstract_sdk::{get_module, verify_os_manager};

use protobuf::Message;

use crate::{contract::ModuleFactoryResult, error::ModuleFactoryError};

use crate::{response::MsgInstantiateContractResponse, state::*};

pub const CREATE_ADD_ON_RESPONSE_ID: u64 = 1u64;
pub const CREATE_SERVICE_RESPONSE_ID: u64 = 3u64;
pub const CREATE_PERK_RESPONSE_ID: u64 = 4u64;

/// Function that starts the creation of the OS
pub fn execute_create_module(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    module_info: ModuleInfo,
    root_init_msg: Option<Binary>,
) -> ModuleFactoryResult {
    let config = CONFIG.load(deps.storage)?;
    // Verify sender is active OS manager
    let core = verify_os_manager(&deps.querier, &info.sender, &config.version_control_address)?;
    let new_module = get_module(&deps.querier, module_info, &config.version_control_address)?;

    // Todo: check if this can be generalized for some contracts
    // aka have default values for each kind of module that only get overwritten if a specific init_msg is saved.
    // let fixed_binary = MODULE_INIT_BINARIES.may_load(deps.storage, new_module.info.clone())?;
    // let init_msg = ModuleInitMsg {
    //     fixed_init: fixed_binary,
    //     root_init: root_init_msg,
    // }
    // .format()?;

    // Set context for after init
    CONTEXT.save(
        deps.storage,
        &Context {
            core: Some(core.clone()),
            module: Some(new_module.clone()),
        },
    )?;
    let block_height = env.block.height;
    match &new_module.reference {
        ModuleReference::App(code_id) => instantiate_contract(
            block_height,
            *code_id,
            root_init_msg.unwrap(),
            Some(core.manager),
            CREATE_ADD_ON_RESPONSE_ID,
            new_module.info,
        ),
        ModuleReference::Perk(code_id) => instantiate_contract(
            block_height,
            *code_id,
            root_init_msg.unwrap(),
            None,
            CREATE_PERK_RESPONSE_ID,
            new_module.info,
        ),
        ModuleReference::Service(code_id) => instantiate_contract(
            block_height,
            *code_id,
            root_init_msg.unwrap(),
            Some(core.manager),
            CREATE_SERVICE_RESPONSE_ID,
            new_module.info,
        ),
        ModuleReference::Extension(addr) => {
            let register_msg: CosmosMsg<Empty> = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: core.manager.into_string(),
                funds: vec![],
                msg: to_binary(&ManagerMsg::RegisterModule {
                    module_addr: addr.to_string(),
                    module: new_module,
                })?,
            });
            Ok(Response::new().add_message(register_msg))
        }
        _ => Err(ModuleFactoryError::ModuleNotInstallable {}),
    }
}

fn instantiate_contract(
    block_height: u64,
    code_id: u64,
    init_msg: Binary,
    admin: Option<Addr>,
    reply_id: u64,
    module_info: ModuleInfo,
) -> ModuleFactoryResult {
    let response = Response::new();
    Ok(response.add_submessage(SubMsg {
        id: reply_id,
        gas_limit: None,
        msg: WasmMsg::Instantiate {
            code_id,
            funds: vec![],
            admin: admin.map(Into::into),
            label: format!("Module: {}, Height {}", module_info, block_height),
            msg: init_msg,
        }
        .into(),
        reply_on: ReplyOn::Success,
    }))
}

pub fn register_contract(deps: DepsMut, result: SubMsgResult) -> ModuleFactoryResult {
    let context: Context = CONTEXT.load(deps.storage)?;
    // Get address of add_on contract
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
    to_add: Vec<(ModuleInfo, Binary)>,
    to_remove: Vec<ModuleInfo>,
) -> ModuleFactoryResult {
    // Only Admin can call this method
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    for (key, binary) in to_add.into_iter() {
        // Update function for new or existing keys
        key.assert_version_variant()?;
        let insert = |_| -> StdResult<Binary> { Ok(binary) };
        MODULE_INIT_BINARIES.update(deps.storage, key, insert)?;
    }

    for key in to_remove {
        key.assert_version_variant()?;
        MODULE_INIT_BINARIES.remove(deps.storage, key);
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
