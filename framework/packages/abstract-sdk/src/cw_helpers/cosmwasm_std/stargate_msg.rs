use cosmos_sdk_proto::traits::Message;
use cosmwasm_std::to_binary;
use cosmwasm_std::CosmosMsg;
use cosmwasm_std::StdResult;
use serde::Serialize;

/// Shortcut helper as the construction of CosmosMsg::Stargate {...} can be quite verbose in contract code.
/// ```rust,norun
///         let msg = MsgCreateDenom {
///             sender: self.sender()?,
///             subdenom: self.subdenom.to_string(),
///         };
///
///         let msg = prost_stargate_msg(MsgCreateDenom::TYPE_URL, &msg)?;
/// ```
pub fn prost_stargate_msg(
    type_url: impl Into<String>,
    msg: impl Message,
) -> StdResult<cosmwasm_std::CosmosMsg> {
    Ok(cosmwasm_std::CosmosMsg::Stargate {
        type_url: type_url.into(),
        value: cosmwasm_std::Binary(msg.encode_to_vec()),
    })
}

#[cfg(test)]
mod test {
    use cosmos_sdk_proto::cosmos;

    use super::*;

    #[test]
    fn test_stargate_msg() {
        let msg = cosmos::feegrant::v1beta1::MsgRevokeAllowance {
            granter: "foo".to_owned(),
            grantee: "bar".to_owned(),
        };
        let _msg = prost_stargate_msg("/cosmos.feegrant.v1beta1.MsgGrantAllowance", &msg).unwrap();
    }
}
