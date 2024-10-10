pub mod authz;
pub mod feegrant;
pub mod gov;
use cosmos_sdk_proto::{cosmos::base, traits::Message, Any};
use cosmwasm_std::{Coin, Timestamp};

/// This trait allows generate `Any` and proto message from Stargate API message
pub trait StargateMessage {
    /// Returned proto type
    type ProtoType: Message;

    fn type_url() -> String;

    /// Get `Any`
    fn to_any(&self) -> Any {
        Any {
            type_url: Self::type_url(),
            value: self.to_proto().encode_to_vec(),
        }
    }

    /// Get `Self::ProtoType`
    fn to_proto(&self) -> Self::ProtoType;
}

pub(crate) fn convert_coins(coins: Vec<Coin>) -> Vec<base::v1beta1::Coin> {
    coins.into_iter().map(convert_coin).collect()
}

pub(crate) fn convert_coin(coin: Coin) -> base::v1beta1::Coin {
    base::v1beta1::Coin {
        denom: coin.denom,
        amount: coin.amount.to_string(),
    }
}

pub(crate) fn convert_ibc_coin(coin: Coin) -> ibc_proto::cosmos::base::v1beta1::Coin {
    ibc_proto::cosmos::base::v1beta1::Coin {
        denom: coin.denom,
        amount: coin.amount.to_string(),
    }
}

pub(crate) fn convert_stamp(stamp: Timestamp) -> cosmos_sdk_proto::Timestamp {
    cosmos_sdk_proto::Timestamp {
        seconds: stamp.seconds() as i64,
        nanos: stamp.nanos() as i32,
    }
}
