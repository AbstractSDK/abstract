use schemars::JsonSchema;

use cosmwasm_std::{to_binary, wasm_execute, Binary, CosmosMsg, StdResult};

use crate::StdAck;

/// IbcResponseMsg should be de/serialized under `IbcCallback()` variant in a ExecuteMsg
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
        Ok(wasm_execute(contract_addr.into(), &IbcCallbackMsg::IbcCallback(self), vec![])?.into())
    }
}

/// This is just a helper to properly serialize the above message.
/// The actual receiver should include this variant in the larger ExecuteMsg enum
#[cosmwasm_schema::cw_serde]
enum IbcCallbackMsg {
    IbcCallback(IbcResponseMsg),
}

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::WasmMsg;
    use speculoos::prelude::*;

    #[test]
    fn into_binary_should_wrap_in_callback() {
        let msg = IbcResponseMsg {
            id: "my-id".to_string(),
            msg: StdAck::Result(Binary::default()),
        };

        let actual = msg.clone().into_binary().unwrap();
        let expected = to_binary(&IbcCallbackMsg::IbcCallback(msg)).unwrap();
        assert_that(&actual).is_equal_to(&expected);
    }

    #[test]
    fn into_cosmos_msg_should_build_wasm_execute() {
        let msg = IbcResponseMsg {
            id: "my-id".to_string(),
            msg: StdAck::Result(Binary::default()),
        };

        let actual = msg.clone().into_cosmos_msg("my-addr").unwrap();
        let funds = vec![];
        let payload = to_binary(&IbcCallbackMsg::IbcCallback(msg)).unwrap();
        let expected: CosmosMsg = WasmMsg::Execute {
            contract_addr: "my-addr".into(),
            msg: payload,
            funds,
        }
        .into();
        assert_that(&actual).is_equal_to(&expected);
    }
}
