use cosmwasm_std::to_binary;
use cosmwasm_std::CosmosMsg;
use cosmwasm_std::StdResult;
use serde::Serialize;

/// Shortcut helper as the construction of CosmosMsg::Stargate {...} can be quite verbose in contract code.
/// ```rust,norun
///         let msg = MsgCreateDenom {
///             sender: self.sender()?,
///             subdenom: self.subdenom.to_string(),
///         }
///         .encode_to_vec();
///
///         let msg = prost_stargate_msg(MsgCreateDenom::TYPE_URL, &msg)?;
/// ```
pub fn prost_stargate_msg(type_url: &str, msg: &[u8]) -> StdResult<cosmwasm_std::CosmosMsg> {
    Ok(cosmwasm_std::CosmosMsg::Stargate {
        type_url: type_url.to_string(),
        value: cosmwasm_std::Binary(msg.to_vec()),
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_stargate_msg() {
        let _msg = prost_stargate_msg("/cosmos.feegrant.v1beta1.MsgGrantAllowance", "key").unwrap();
    }
}
