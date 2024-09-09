use super::*;

use cosmwasm_std::{Addr, SubMsgResponse, Uint64};

#[cw_serde]
pub enum Callback {
    /// Result of executing the requested query, or an error.
    ///
    /// result[i] corresponds to the i'th query and contains the
    /// base64 encoded query response.
    Query(Result<Vec<Binary>, ErrorResponse>),

    /// Result of executing the requested messages, or an error.
    ///
    /// 14/04/23: if a submessage errors the reply handler can see
    /// `codespace: wasm, code: 5`, but not the actual error. as a
    /// result, we can't return good errors for Execution and this
    /// error string will only tell you the error's codespace. for
    /// example, an out-of-gas error is code 11 and looks like
    /// `codespace: sdk, code: 11`.
    Execute(Result<ExecutionResponse, String>),

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

#[cw_serde]
pub struct ErrorResponse {
    /// The index of the first message who's execution failed.
    pub message_index: Uint64,
    /// The error that occured executing the message.
    pub error: String,
}

#[cw_serde]
pub struct ExecutionResponse {
    /// The address on the remote chain that executed the messages.
    pub executed_by: String,
    /// Index `i` corresponds to the result of executing the `i`th
    /// message.
    pub result: Vec<SubMsgResponse>,
}

/// A request for a callback.
#[cw_serde]
pub struct CallbackRequest {
    pub receiver: String,
    pub msg: Binary,
}

/// Executed on the callback receiver upon message completion. When
/// being executed, the message will be tagged with "callback":
///
/// ```json
/// {"callback": {
///       "initiator": ...,
///       "initiator_msg": ...,
///       "result": ...,
/// }}
/// ```
#[cw_serde]
pub struct CallbackMessage {
    /// Initaitor on the note chain.
    pub initiator: Addr,
    /// Message sent by the initaitor. This _must_ be base64 encoded
    /// or execution will fail.
    pub initiator_msg: Binary,
    /// Data from the host chain.
    pub result: Callback,
}
