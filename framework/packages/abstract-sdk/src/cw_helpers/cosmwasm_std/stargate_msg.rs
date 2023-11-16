use cosmos_sdk_proto::traits::Message;

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
) -> cosmwasm_std::CosmosMsg {
    cosmwasm_std::CosmosMsg::Stargate {
        type_url: type_url.into(),
        value: cosmwasm_std::Binary(msg.encode_to_vec()),
    }
}

#[cfg(test)]
mod test {
    use cosmos_sdk_proto::cosmos;
    use cosmwasm_std::CosmosMsg;

    use super::*;

    #[test]
    fn test_stargate_msg() {
        let revoke_msg = cosmos::feegrant::v1beta1::MsgRevokeAllowance {
            granter: "foo".to_owned(),
            grantee: "bar".to_owned(),
        };
        let type_url = "/cosmos.feegrant.v1beta1.MsgGrantAllowance";
        let msg = prost_stargate_msg(type_url, revoke_msg.clone());

        let expected_msg = CosmosMsg::Stargate {
            type_url: type_url.to_owned(),
            value: cosmwasm_std::Binary(revoke_msg.encode_to_vec()),
        };
        assert_eq!(msg, expected_msg)
    }
}
