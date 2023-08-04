use cosmwasm_std::to_binary;
use cosmwasm_std::CosmosMsg;
use cosmwasm_std::StdResult;
use serde::Serialize;

/// Shortcut helper as the construction of CosmosMsg::Stargate {...} can be quite verbose in contract code
pub fn stargate_msg<T: Serialize + ?Sized>(
    type_url: impl ToString,
    msg: &T,
) -> StdResult<CosmosMsg> {
    Ok(CosmosMsg::Stargate {
        type_url: type_url.to_string(),
        value: to_binary(msg)?,
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_stargate_msg() {
        let _msg = stargate_msg("/cosmos.feegrant.v1beta1.MsgGrantAllowance", "key").unwrap();
    }
}
