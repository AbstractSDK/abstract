use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    to_json_binary, wasm_execute, Binary, CosmosMsg, Empty, Event, QueryRequest, StdError,
    StdResult,
};
use polytone::callbacks::{Callback as PolytoneCallback, ErrorResponse, ExecutionResponse};
use schemars::JsonSchema;
use serde::Serialize;

use crate::{
    base::ExecuteMsg,
    ibc_client,
    objects::{chain_name::ChainName, module::ModuleInfo},
};

/// Callback from modules, that is turned into an IbcResponseMsg by the ibc client
/// A callback can only be sent to itself
#[cosmwasm_schema::cw_serde]
// ANCHOR: callback-info
pub struct Callback {
    /// Used to add information to the callback.
    /// This is usually used to provide information to the ibc callback function for context
    pub msg: Binary,
}
// ANCHOR_END: callback-info

impl Callback {
    pub fn new<T: Serialize>(msg: &T) -> StdResult<Self> {
        Ok(Self {
            msg: to_json_binary(msg)?,
        })
    }
}

/// IbcResponseMsg should be de/serialized under `IbcCallback()` variant in a ExecuteMsg
#[cosmwasm_schema::cw_serde]
// ANCHOR: response-msg
pub struct IbcResponseMsg {
    /// The msg sent with the callback request.
    pub callback: Callback,
    pub result: IbcResult,
}
// ANCHOR_END: response-msg

impl IbcResponseMsg {
    /// serializes the message
    pub fn into_json_binary(self) -> StdResult<Binary> {
        let msg = ExecuteMsg::IbcCallback::<Empty, Empty>(self);
        to_json_binary(&msg)
    }

    /// creates a cosmos_msg sending this struct to the named contract
    pub fn into_cosmos_msg<T: Into<String>, C>(self, contract_addr: T) -> StdResult<CosmosMsg<C>>
    where
        C: Clone + std::fmt::Debug + PartialEq + JsonSchema,
    {
        Ok(wasm_execute(
            contract_addr.into(),
            &ExecuteMsg::IbcCallback::<Empty, Empty>(self),
            vec![],
        )?
        .into())
    }
}

#[cosmwasm_schema::cw_serde]
pub enum IbcResult {
    Query {
        queries: Vec<QueryRequest<ModuleQuery>>,
        results: Result<Vec<Binary>, ErrorResponse>,
    },

    Execute {
        initiator_msg: Binary,
        result: Result<ExecutionResponse, String>,
    },

    /// An error occured that could not be recovered from. The only
    /// known way that this can occur is message handling running out
    /// of gas, in which case the error will be `codespace: sdk, code:
    /// 11`.
    ///
    /// This error is not named becuase it could also occur due to a
    /// panic or unhandled error during message processing. We don't
    /// expect this to happen and have carefully written the code to
    /// avoid it.
    FatalError(String),
}

impl IbcResult {
    pub fn from_query(
        callback: PolytoneCallback,
        queries: Vec<QueryRequest<ModuleQuery>>,
    ) -> Result<Self, StdError> {
        match callback {
            PolytoneCallback::Query(q) => Ok(Self::Query {
                queries,
                results: q,
            }),
            PolytoneCallback::Execute(_) => Err(StdError::generic_err(
                "Expected a query result, got an execute result",
            )),
            PolytoneCallback::FatalError(e) => Ok(Self::FatalError(e)),
        }
    }

    pub fn from_execute(
        callback: PolytoneCallback,
        initiator_msg: Binary,
    ) -> Result<Self, StdError> {
        match callback {
            PolytoneCallback::Query(_) => Err(StdError::generic_err(
                "Expected an execution result, got a query result",
            )),
            PolytoneCallback::Execute(e) => Ok(Self::Execute {
                initiator_msg,
                result: e,
            }),
            PolytoneCallback::FatalError(e) => Ok(Self::FatalError(e)),
        }
    }

    /// Get query result
    pub fn get_query_result(&self, index: usize) -> StdResult<(QueryRequest<ModuleQuery>, Binary)> {
        match &self {
            IbcResult::Query { queries, results } => {
                let results = results
                    .as_ref()
                    .map_err(|err| StdError::generic_err(err.error.clone()))?;
                Ok((queries[index].clone(), results[index].clone()))
            }
            IbcResult::Execute { .. } => Err(StdError::generic_err(
                "expected query, got execute ibc result",
            )),
            IbcResult::FatalError(err) => Err(StdError::generic_err(err.to_owned())),
        }
    }

    /// Get execute result
    pub fn get_execute_events(&self) -> StdResult<Vec<Event>> {
        match &self {
            IbcResult::Execute { result, .. } => {
                let result = result
                    .as_ref()
                    .map_err(|err| StdError::generic_err(err.clone()))?;
                // result should always be size 1 (proxy -> ibc-host --multiple-msgs-> module)
                let res = result
                    .result
                    .get(0)
                    .expect("execution response without submsg");
                Ok(res.events.clone())
            }
            IbcResult::Query { .. } => Err(StdError::generic_err(
                "expected execute, got query ibc result",
            )),
            IbcResult::FatalError(err) => Err(StdError::generic_err(err.to_owned())),
        }
    }
}

#[cw_serde]
pub struct ModuleIbcMsg {
    /// Sender Module Identification
    pub src_module_info: ModuleIbcInfo,
    /// The message sent by the module
    pub msg: Binary,
}

// ANCHOR: module_ibc_msg
#[cw_serde]
pub struct ModuleIbcInfo {
    /// Remote chain identification
    pub chain: ChainName,
    /// Information about the module that called ibc action on this module
    pub module: ModuleInfo,
}
// ANCHOR_END: module_ibc_msg

// ANCHOR: module_ibc_query
#[cw_serde]
pub struct ModuleQuery {
    /// Information about the module that gets queried through ibc
    pub target_module: ibc_client::InstalledModuleIdentification,
    /// The WasmQuery::Smart request to the module
    pub msg: Binary,
}
// ANCHOR_END: module_ibc_query
