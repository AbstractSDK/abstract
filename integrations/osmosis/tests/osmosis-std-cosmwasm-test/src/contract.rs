use std::fmt::Debug;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, to_vec, Binary, ContractResult, Deps, DepsMut, Empty, Env, MessageInfo, Reply,
    Response, StdError, StdResult, SystemResult,
};
use cw2::set_contract_version;
use osmosis_std::types::osmosis::epochs::v1beta1::{
    QueryEpochsInfoRequest, QueryEpochsInfoResponse,
};
use osmosis_std::types::osmosis::gamm::v1beta1::{
    QueryNumPoolsRequest, QueryNumPoolsResponse, QueryPoolParamsRequest, QueryPoolParamsResponse,
    QueryPoolRequest, QueryPoolResponse,
};
use osmosis_std::types::osmosis::twap::v1beta1::{
    ArithmeticTwapToNowResponse, GeometricTwapToNowResponse,
};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMapResponse, QueryMsg};
use crate::state::{DEBUG, MAP, OWNER};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:osmosis-std-cosmwasm-test";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Handling contract instantiation
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    DEBUG.save(deps.storage, &msg.debug)?;
    OWNER.save(deps.storage, &info.sender)?;

    // With `Response` type, it is possible to dispatch message to invoke external logic.
    // See: https://github.com/CosmWasm/cosmwasm/blob/main/SEMANTICS.md#dispatching-messages
    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

/// Handling contract migration
/// To make a contract migratable, you need
/// - this entry_point implemented
/// - only contract admin can migrate, so admin has to be set at contract initiation time
/// Handling contract execution
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    match msg {
        // Find matched incoming message variant and execute them with your custom logic.
        //
        // With `Response` type, it is possible to dispatch message to invoke external logic.
        // See: https://github.com/CosmWasm/cosmwasm/blob/main/SEMANTICS.md#dispatching-messages
    }
}

/// Handling contract execution
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SetMap { key, value } => {
            if OWNER.load(deps.storage)? != info.sender {
                return Err(ContractError::Unauthorized {});
            }
            MAP.save(deps.storage, key, &value)?;
            Ok(Response::new().add_attribute("method", "set_map"))
        }
    }
}

/// Handling contract query
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryNumPools {} => {
            query_and_debug::<QueryNumPoolsResponse>(&deps, QueryNumPoolsRequest {})
        }
        QueryMsg::QueryEpochsInfo {} => {
            query_and_debug::<QueryEpochsInfoResponse>(&deps, QueryEpochsInfoRequest {})
        }
        QueryMsg::QueryPool { pool_id } => {
            query_and_debug::<QueryPoolResponse>(&deps, QueryPoolRequest { pool_id })
        }
        QueryMsg::QueryPoolParams { pool_id } => {
            query_and_debug::<QueryPoolParamsResponse>(&deps, QueryPoolParamsRequest { pool_id })
        }
        QueryMsg::QueryArithmeticTwapToNow(arithmetic_twap_request) => {
            query_and_debug::<ArithmeticTwapToNowResponse>(&deps, arithmetic_twap_request)
        }
        QueryMsg::QueryGeometricTwapToNow(geometric_twap_request) => {
            query_and_debug::<GeometricTwapToNowResponse>(&deps, geometric_twap_request)
        }
        QueryMsg::QueryMap { key } => to_binary(&QueryMapResponse {
            value: MAP.load(deps.storage, key)?,
        }),
    }
}

fn query_and_debug<T>(
    deps: &Deps,
    q: impl Into<cosmwasm_std::QueryRequest<Empty>>,
) -> StdResult<Binary>
where
    T: Serialize + DeserializeOwned + Debug,
{
    to_binary(&{
        let request: cosmwasm_std::QueryRequest<Empty> = q.into();
        let raw = to_vec(&request).map_err(|serialize_err| {
            StdError::generic_err(format!("Serializing QueryRequest: {}", serialize_err))
        })?;
        let res: T = match deps.querier.raw_query(&raw) {
            SystemResult::Err(system_err) => Err(StdError::generic_err(format!(
                "Querier system error: {}",
                system_err
            ))),
            SystemResult::Ok(ContractResult::Err(contract_err)) => Err(StdError::generic_err(
                format!("Querier contract error: {}", contract_err),
            )),
            SystemResult::Ok(ContractResult::Ok(value)) => {
                if DEBUG.load(deps.storage)? {
                    let json_str = std::str::from_utf8(value.as_slice()).unwrap();
                    let json_str = jsonformat::format(json_str, jsonformat::Indentation::TwoSpace);

                    deps.api.debug("========================");
                    match request {
                        cosmwasm_std::QueryRequest::Stargate { path, data: _ } => {
                            deps.api
                                .debug(format!("Stargate Query :: {}", path).as_str());
                        }
                        request => {
                            deps.api.debug(format!("{:?}", request).as_str());
                        }
                    };

                    deps.api.debug("");
                    deps.api.debug("```");
                    deps.api.debug(&json_str);
                    deps.api.debug("```");
                    deps.api.debug("========================");
                }
                cosmwasm_std::from_binary(&value)
            }
        }?;
        res
    })
}

/// Handling submessage reply.
/// For more info on submessage and reply, see https://github.com/CosmWasm/cosmwasm/blob/main/SEMANTICS.md#submessages
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, _msg: Reply) -> Result<Response, ContractError> {
    // With `Response` type, it is still possible to dispatch message to invoke external logic.
    // See: https://github.com/CosmWasm/cosmwasm/blob/main/SEMANTICS.md#dispatching-messages

    todo!()
}
