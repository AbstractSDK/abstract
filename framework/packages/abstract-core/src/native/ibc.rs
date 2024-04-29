use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    from_json, to_json_binary, wasm_execute, Binary, CosmosMsg, Empty, QueryRequest, StdError,
    StdResult,
};
use polytone::callbacks::{Callback, ErrorResponse, ExecutionResponse};
use schemars::JsonSchema;
use serde::de::DeserializeOwned;

use crate::{
    base::ExecuteMsg,
    objects::{chain_name::ChainName, module::ModuleInfo},
};

/// CallbackInfo from modules, that is turned into an IbcResponseMsg by the ibc client
/// A callback can only be sent to itself
#[cosmwasm_schema::cw_serde]
pub struct CallbackInfo {
    /// Used to identify the callback that is sent (acts like the reply ID)
    pub id: String,
    /// Used to add information to the callback.
    /// This is usually used to provide information to the ibc callback function for context
    pub msg: Option<Binary>,
}

/// IbcResponseMsg should be de/serialized under `IbcCallback()` variant in a ExecuteMsg
#[cosmwasm_schema::cw_serde]
pub struct IbcCallbackMsg {
    /// The ID chosen by the caller in the `callback_info.id`
    pub id: String,
    /// The msg sent with the callback request.
    /// This is usually used to provide information to the ibc callback function for context
    pub msg: Option<Binary>,
    pub result: CallbackResult,
}

impl IbcCallbackMsg {
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
pub enum CallbackResult {
    Query {
        query: QueryRequest<Empty>,
        result: Result<Vec<Binary>, ErrorResponse>,
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

impl CallbackResult {
    pub fn from_query(callback: Callback, query: QueryRequest<Empty>) -> Result<Self, StdError> {
        match callback {
            Callback::Query(q) => Ok(Self::Query { query, result: q }),
            Callback::Execute(_) => Err(StdError::generic_err(
                "Expected a query result, got an execute result",
            )),
            Callback::FatalError(e) => Ok(Self::FatalError(e)),
        }
    }

    pub fn from_execute(callback: Callback, initiator_msg: Binary) -> Result<Self, StdError> {
        match callback {
            Callback::Query(_) => Err(StdError::generic_err(
                "Expected an execution result, got a query result",
            )),
            Callback::Execute(e) => Ok(Self::Execute {
                initiator_msg,
                result: e,
            }),
            Callback::FatalError(e) => Ok(Self::FatalError(e)),
        }
    }
}

// ANCHOR: module_ibc_msg
#[cw_serde]
pub struct ModuleIbcMsg {
    pub client_chain: ChainName,
    pub source_module: ModuleInfo,
    pub msg: Binary,
}
// ANCHOR_END: module_ibc_msg

impl ModuleIbcMsg {
    /// Deserializes the underlying message to a type
    /// This message is specified directly by the calling module
    pub fn parse_msg<T: DeserializeOwned>(&self) -> StdResult<T> {
        from_json(&self.msg)
    }
}
