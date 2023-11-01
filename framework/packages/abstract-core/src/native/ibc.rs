use cosmwasm_std::{to_json_binary, wasm_execute, Binary, CosmosMsg, StdResult};
use polytone::callbacks::Callback;
use schemars::JsonSchema;

// CallbackInfo from modules, that is turned into an IbcResponseMsg by the ibc client
#[cosmwasm_schema::cw_serde]
pub struct CallbackInfo {
    /// Used to identify the callback that is sent (acts like the reply ID)
    pub id: String,
    /// Used to add information to the callback.
    /// This is usually used to provide information to the ibc callback function for context
    pub msg: Option<Binary>,
    /// Contract that will be called with the callback message
    pub receiver: String,
}

/// IbcResponseMsg should be de/serialized under `IbcCallback()` variant in a ExecuteMsg
#[cosmwasm_schema::cw_serde]
pub struct IbcResponseMsg {
    /// The ID chosen by the caller in the `callback_info.id`
    pub id: String,
    /// The msg sent with the callback request.
    /// This is usually used to provide information to the ibc callback function for context
    pub msg: Option<Binary>,
    pub result: Callback,
}

impl IbcResponseMsg {
    /// serializes the message
    pub fn into_json_binary(self) -> StdResult<Binary> {
        let msg = IbcCallbackMsg::IbcCallback(self);
        to_json_binary(&msg)
    }

    /// creates a cosmos_msg sending this struct to the named contract
    pub fn into_cosmos_msg<T: Into<String>, C>(self, contract_addr: T) -> StdResult<CosmosMsg<C>>
    where
        C: Clone + std::fmt::Debug + PartialEq + JsonSchema,
    {
        Ok(wasm_execute(
            contract_addr.into(),
            &IbcCallbackMsg::IbcCallback(self),
            vec![],
        )?
        .into())
    }
}

/// This is just a helper to properly serialize the above message.
/// The actual receiver should include this variant in the larger ExecuteMsg enum
#[cosmwasm_schema::cw_serde]
enum IbcCallbackMsg {
    IbcCallback(IbcResponseMsg),
}
