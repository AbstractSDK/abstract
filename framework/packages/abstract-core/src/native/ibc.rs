use cosmwasm_schema::cw_serde;
use cosmwasm_std::{to_json_binary, wasm_execute, Binary, CosmosMsg, Empty, StdResult};
use polytone::callbacks::Callback;
use schemars::JsonSchema;

use crate::{
    base::ExecuteMsg,
    objects::{chain_name::ChainName, module::ModuleInfo},
};

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
pub struct IbcCallbackMsg {
    /// The ID chosen by the caller in the `callback_info.id`
    pub id: String,
    /// The msg sent with the callback request.
    /// This is usually used to provide information to the ibc callback function for context
    pub msg: Option<Binary>,
    /// The msg that initiated the ibc callback
    pub initiator_msg: Binary,
    /// This identifies the module that called the action initially
    /// This SHOULD be used by the callback function to identify the callback sender
    pub sender_module: ModuleInfo,
    pub result: Callback,
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

// ANCHOR: module_ibc_msg
#[cw_serde]
pub struct ModuleIbcMsg {
    pub client_chain: ChainName,
    pub source_module: ModuleInfo,
    pub msg: Binary,
}
// ANCHOR_END: module_ibc_msg
