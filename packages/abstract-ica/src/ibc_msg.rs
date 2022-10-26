use cosmwasm_std::{from_slice, to_binary, Binary, Coin};

use serde::de::DeserializeOwned;
use serde::Serialize;

/// This is a generic ICS acknowledgement format.
/// Proto defined here: https://github.com/cosmos/cosmos-sdk/blob/v0.42.0/proto/ibc/core/channel/v1/channel.proto#L141-L147
/// If ibc_receive_packet returns Err(), then x/wasm runtime will rollback the state and return an error message in this format
#[cosmwasm_schema::cw_serde]
pub enum StdAck {
    Result(Binary),
    Error(String),
}

impl StdAck {
    // create a serialized success message
    pub fn success(data: impl Serialize) -> Binary {
        let res = to_binary(&data).unwrap();
        StdAck::Result(res).ack()
    }

    // create a serialized error message
    pub fn fail(err: String) -> Binary {
        StdAck::Error(err).ack()
    }

    pub fn ack(&self) -> Binary {
        to_binary(self).unwrap()
    }

    pub fn unwrap(self) -> Binary {
        match self {
            StdAck::Result(data) => data,
            StdAck::Error(err) => panic!("{}", err),
        }
    }

    pub fn unwrap_into<T: DeserializeOwned>(self) -> T {
        from_slice(&self.unwrap()).unwrap()
    }

    pub fn unwrap_err(self) -> String {
        match self {
            StdAck::Result(_) => panic!("not an error"),
            StdAck::Error(err) => err,
        }
    }
}

/// Return the data field for each message
#[cosmwasm_schema::cw_serde]
pub struct DispatchResponse {
    pub results: Vec<Binary>,
}

#[cosmwasm_schema::cw_serde]
pub struct SendAllBackResponse {}

/// Identify the host chain
#[cosmwasm_schema::cw_serde]
pub struct WhoAmIResponse {
    pub chain: String,
}

/// Return the data field for each message
#[cosmwasm_schema::cw_serde]
pub struct IbcQueryResponse {
    pub results: Vec<Binary>,
}

/// This is the success response we send on ack for PacketMsg::Register.
/// Return the caller's account address on the remote chain
#[cosmwasm_schema::cw_serde]
pub struct RegisterResponse {
    pub account: String,
}

/// This is the success response we send on ack for PacketMsg::Balance.
/// Just acknowledge success or error
#[cosmwasm_schema::cw_serde]
pub struct BalancesResponse {
    pub account: String,
    pub balances: Vec<Coin>,
}
