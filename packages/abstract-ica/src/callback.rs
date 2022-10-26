use schemars::JsonSchema;

use cosmwasm_std::{to_binary, Binary, CosmosMsg, StdResult, WasmMsg};

use crate::StdAck;

/// IbcResponseMsg should be de/serialized under `Receive()` variant in a ExecuteMsg
#[cosmwasm_schema::cw_serde]
pub struct IbcResponseMsg {
    /// The ID chosen by the caller in the `callback_id`
    pub id: String,
    pub msg: StdAck,
}

impl IbcResponseMsg {
    /// serializes the message
    pub fn into_binary(self) -> StdResult<Binary> {
        let msg = IbcCallbackMsg::IbcCallback(self);
        to_binary(&msg)
    }

    /// creates a cosmos_msg sending this struct to the named contract
    pub fn into_cosmos_msg<T: Into<String>, C>(self, contract_addr: T) -> StdResult<CosmosMsg<C>>
    where
        C: Clone + std::fmt::Debug + PartialEq + JsonSchema,
    {
        let msg = self.into_binary()?;
        let execute = WasmMsg::Execute {
            contract_addr: contract_addr.into(),
            msg,
            funds: vec![],
        };
        Ok(execute.into())
    }
}

/// This is just a helper to properly serialize the above message.
/// The actual receiver should include this variant in the larger ExecuteMsg enum
#[cosmwasm_schema::cw_serde]
enum IbcCallbackMsg {
    IbcCallback(IbcResponseMsg),
}
