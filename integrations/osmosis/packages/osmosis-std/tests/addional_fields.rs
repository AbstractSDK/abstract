use cosmwasm_std::{from_binary, to_binary};
use osmosis_std_derive::CosmwasmExt;
use prost::Message;

#[derive(
    Clone,
    PartialEq,
    Eq,
    ::prost::Message,
    ::serde::Serialize,
    ::serde::Deserialize,
    schemars::JsonSchema,
    CosmwasmExt,
)]
#[proto_message(type_url = "/osmosis.lockup.MsgBeginUnlockingResponse")]
pub struct MsgBeginUnlockingResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
}

#[derive(
    Clone,
    PartialEq,
    Eq,
    ::prost::Message,
    ::serde::Serialize,
    ::serde::Deserialize,
    schemars::JsonSchema,
    CosmwasmExt,
)]
#[proto_message(type_url = "/osmosis.lockup.MsgBeginUnlockingResponse")]
pub struct NewMsgBeginUnlockingResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
    #[prost(uint64, tag = "2")]
    pub unlocking_lock_id: u64,
}

#[test]
fn test_additional_fields_does_not_break_but_cause_lossy_json_deserialization() {
    let response = NewMsgBeginUnlockingResponse {
        success: true,
        unlocking_lock_id: 1,
    };

    // to_binary() and from_binary() is using `serde_json_wasm` under the hood.
    let serialized = to_binary(&response).unwrap();
    let deserialized: MsgBeginUnlockingResponse = from_binary(&serialized).unwrap();

    // lossy deserialization
    assert_eq!(deserialized, MsgBeginUnlockingResponse { success: true });
}

#[test]
fn test_additional_fields_does_not_break_but_cause_lossy_proto_deserialization() {
    let response = NewMsgBeginUnlockingResponse {
        success: true,
        unlocking_lock_id: 1,
    };
    let serialized = response.encode_to_vec();
    let deserialized = MsgBeginUnlockingResponse::decode(&serialized[..]).unwrap();

    // lossy deserialization
    assert_eq!(deserialized, MsgBeginUnlockingResponse { success: true });
}

mod shim {
    pub struct Any {
        pub type_url: String,
        pub value: Vec<u8>,
    }
}
