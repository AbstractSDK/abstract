use cosmwasm_std::{
    to_binary, Addr, Coin, DepsMut, Empty, Env, MessageInfo, QuerierWrapper, QueryRequest, ReplyOn,
    Response, StdError, StdResult, SubMsg, WasmMsg, WasmQuery,
};
use cosmwasm_std::{ContractResult, CosmosMsg, SubMsgExecutionResponse};
use pandora_os::core::manager::helper::register_module_on_manager;
use pandora_os::core::modules::ModuleInfo;
use pandora_os::governance::gov_type::GovernanceDetails;
use pandora_os::modules::add_ons::subscription::msg::{
    ExecuteMsg as SubscriptionExecMsg, QueryMsg as SubscriptionQuery, SubscriptionFeeResponse,
};
use protobuf::Message;

use crate::contract::OsFactoryResult;

use crate::error::OsFactoryError;
use crate::response::MsgInstantiateContractResponse;

use crate::state::*;
use pandora_os::core::manager::msg::InstantiateMsg as ManagerInstantiateMsg;
use pandora_os::core::proxy::msg::{
    ExecuteMsg as ProxyExecMsg, InstantiateMsg as ProxyInstantiateMsg,
};

use cw_asset::Asset;
use pandora_os::native::version_control::msg::{
    CodeIdResponse, ExecuteMsg as VCExecuteMsg, QueryMsg as VCQuery,
};

pub const CREATE_OS_MANAGER_MSG_ID: u64 = 1u64;
pub const CREATE_OS_TREASURY_MSG_ID: u64 = 2u64;
use pandora_os::registery::{MANAGER, PROXY};

/// Function that starts the creation of the OS
pub fn execute_create_os(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    governance: GovernanceDetails,
) -> OsFactoryResult {
    let config = CONFIG.load(deps.storage)?;

    let subscription_fee: SubscriptionFeeResponse =
        query_subscription_fee(&deps.querier, &config.subscription_address)?;
    let received_payment_coin = info.funds.clone().pop().unwrap_or_default();
    let received_payment = Asset::from(received_payment_coin.clone());

    let mut msgs = vec![];

    if !subscription_fee.fee.amount.is_zero() {
        if subscription_fee.fee.amount != received_payment.amount {
            return Err(OsFactoryError::WrongAmount(
                subscription_fee.fee.to_string(),
            ));
        } else if subscription_fee.fee.info != received_payment.info {
            return Err(OsFactoryError::UnsupportedAsset());
        } else if config.next_os_id != 0 {
            // Forward payment to subscription module, also registers the OS
            // Don't do this for the first OS we create
            let forward_payment_to_module: CosmosMsg<Empty> = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: config.subscription_address.to_string(),
                funds: vec![received_payment_coin],
                msg: to_binary(&SubscriptionExecMsg::Pay {
                    asset: received_payment,
                    os_id: config.next_os_id,
                })?,
            });
            msgs.push(forward_payment_to_module)
        }
    }

    // Get address of OS root user, depends on gov-type
    let root_user: Addr = match governance {
        GovernanceDetails::Monarchy { monarch } => deps.api.addr_validate(&monarch)?,
        _ => return Err(StdError::generic_err("Not Implemented").into()),
    };

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

    Ok(Response::new()
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
                    subscription_address: config.subscription_address.to_string(),
                    module_factory_address: config.module_factory_address.to_string(),
                })?,
            }
            .into(),
            reply_on: ReplyOn::Success,
        })
        // Add as subscription registration as last. Gets called after the reply sequence is done.
        .add_messages(msgs))
}

/// instantiates the Treasury contract of the newly created DAO
pub fn after_manager_create_proxy(
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

    CONTEXT.save(
        deps.storage,
        &Context {
            os_manager_address: deps.api.addr_validate(manager_address)?,
        },
    )?;

    // Query version_control for code_id of Treasury
    let proxy_code_id_response: CodeIdResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.version_control_contract.to_string(),
            msg: to_binary(&VCQuery::QueryCodeId {
                module: ModuleInfo {
                    name: String::from(PROXY),
                    version: None,
                },
            })?,
        }))?;
    Ok(Response::new()
        .add_attribute("Manager Address:", &manager_address.to_string())
        // Instantiate Treasury contract
        .add_submessage(SubMsg {
            id: CREATE_OS_TREASURY_MSG_ID,
            gas_limit: None,
            msg: WasmMsg::Instantiate {
                code_id: proxy_code_id_response.code_id.u64(),
                funds: vec![],
                admin: Some(manager_address.to_string()),
                label: format!("Proxy of OS: {}", config.next_os_id),
                msg: to_binary(&ProxyInstantiateMsg {
                    os_id: config.next_os_id,
                })?,
            }
            .into(),
            reply_on: ReplyOn::Success,
        }))
}
/// Registers the DAO on the version_control contract and
/// adds proxy contract address to Manager
pub fn after_proxy_add_to_manager_and_set_admin(
    deps: DepsMut,
    result: ContractResult<SubMsgExecutionResponse>,
) -> OsFactoryResult {
    let mut config = CONFIG.load(deps.storage)?;
    let context = CONTEXT.load(deps.storage)?;

    let res: MsgInstantiateContractResponse =
        Message::parse_from_bytes(result.unwrap().data.unwrap().as_slice()).map_err(|_| {
            StdError::parse_err("MsgInstantiateContractResponse", "failed to parse data")
        })?;

    let proxy_address = res.get_contract_address();

    // Add OS core to version_control
    let add_os_core_to_version_control_msg: CosmosMsg<Empty> = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.version_control_contract.to_string(),
        funds: vec![],
        msg: to_binary(&VCExecuteMsg::AddOs {
            os_id: config.next_os_id,
            manager_address: context.os_manager_address.to_string(),
            proxy_address: deps.api.addr_validate(proxy_address)?.into_string(),
        })?,
    });

    let set_manager_as_admin_msg: CosmosMsg<Empty> = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: proxy_address.to_string(),
        funds: vec![],
        msg: to_binary(&ProxyExecMsg::SetAdmin {
            admin: context.os_manager_address.to_string(),
        })?,
    });

    // Update id sequence
    config.next_os_id += 1;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_message(add_os_core_to_version_control_msg)
        .add_attribute("Proxy Address: ", res.get_contract_address())
        .add_message(register_module_on_manager(
            context.os_manager_address.to_string(),
            PROXY.to_string(),
            proxy_address.to_string(),
        )?)
        .add_message(set_manager_as_admin_msg))
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
    subscription_address: Option<String>,
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

    if let Some(subscription_address) = subscription_address {
        config.subscription_address = deps.api.addr_validate(&subscription_address)?;
    }

    CONFIG.save(deps.storage, &config)?;

    if let Some(admin) = admin {
        let addr = deps.api.addr_validate(&admin)?;
        ADMIN.set(deps, Some(addr))?;
    }

    Ok(Response::new().add_attribute("action", "update_config"))
}

fn query_subscription_fee(
    querier: &QuerierWrapper,
    subscription_address: &Addr,
) -> StdResult<SubscriptionFeeResponse> {
    let subscription_fee_response: SubscriptionFeeResponse =
        querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: subscription_address.to_string(),
            msg: to_binary(&SubscriptionQuery::Fee {})?,
        }))?;
    Ok(subscription_fee_response)
}
