use cosmos_sdk_proto::{
    cosmos::base,
    traits::{Message, Name},
};
use cosmwasm_std::{Coin, Timestamp};
use prost_types::Any;

pub mod feegrant;

/// This trait allows generate `Any` and proto message from Stargate API message
pub trait StargateMessage {
    /// Returned proto type
    type ProtoType: Message + Name + Sized;

    /// Get `Any`
    fn to_any(&self) -> Any {
        Any {
            type_url: Self::ProtoType::type_url(),
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

pub(crate) fn convert_stamp(stamp: Timestamp) -> prost_types::Timestamp {
    prost_types::Timestamp {
        seconds: stamp.seconds() as i64,
        nanos: stamp.nanos() as i32,
    }
}
