use cosmwasm_std::{from_slice, to_binary, Binary, Coin};
use serde::{de::DeserializeOwned, Serialize};

/// This is a generic ICS acknowledgement format.
/// Proto defined [here](https://github.com/cosmos/cosmos-sdk/blob/v0.42.0/proto/ibc/core/channel/v1/channel.proto#L141-L147)
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

#[cfg(test)]
mod test {
    use super::*;
    use speculoos::prelude::*;

    const TEST_DATA_STR: &str = "test data";

    fn test_binary_data() -> Binary {
        to_binary(TEST_DATA_STR).unwrap()
    }

    #[test]
    fn success_should_wrap_in_result_with_binary_data() {
        let expected = StdAck::Result(test_binary_data()).ack();
        let actual = StdAck::success(TEST_DATA_STR);
        assert_that!(&actual).is_equal_to(&expected);
    }

    #[test]
    fn fail_should_wrap_in_error() {
        let err = "my-error";
        let expected = to_binary(&StdAck::Error(err.to_string())).unwrap();
        let actual = StdAck::fail(err.to_string());
        assert_that!(&actual).is_equal_to(&expected);
    }

    #[test]
    fn ack_should_binary_contents() {
        let actual: Binary = StdAck::Result(test_binary_data()).ack();
        let expected = to_binary(&StdAck::Result(test_binary_data())).unwrap();
        assert_that!(&actual).is_equal_to(&expected);
    }

    #[test]
    fn unwrap_with_result_should_return_binary_data() {
        let expected_data = test_binary_data();
        let actual = StdAck::Result(expected_data.clone()).unwrap();
        let expected = expected_data;
        assert_that!(&actual).is_equal_to(&expected);
    }

    #[test]
    #[should_panic]
    fn unwrap_with_error_should_panic() {
        StdAck::Error("my-error".to_string()).unwrap();
    }

    #[test]
    fn unwrap_into_should_return_deserialized_data() {
        let actual = StdAck::Result(test_binary_data()).unwrap_into::<String>();
        let expected = TEST_DATA_STR.to_string();
        assert_that!(&actual).is_equal_to(&expected);
    }

    #[test]
    fn unwrap_err_with_err_should_return_error_message() {
        let err = "my-error";
        let actual = StdAck::Error(err.to_string()).unwrap_err();
        let expected = err.to_string();
        assert_that!(&actual).is_equal_to(&expected);
    }

    #[test]
    #[should_panic]
    fn unwrap_err_with_result_should_panic() {
        let _data = "my-data";
        let _actual = StdAck::Result(Binary::default()).unwrap_err();
    }
}
