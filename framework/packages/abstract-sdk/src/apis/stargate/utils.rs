use cosmos_sdk_proto::cosmos::base;
use cosmwasm_std::{Coin, Timestamp};

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
