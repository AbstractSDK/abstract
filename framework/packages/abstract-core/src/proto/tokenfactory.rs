#![allow(non_snake_case)]
use cosmos_sdk_proto::traits::TypeUrl;

// This struct is an exact copy of https://github.com/osmosis-labs/osmosis-rust/blob/5997b8797a3210df0b1ab017025506a7645ff961/packages/osmosis-std/src/types/osmosis/tokenfactory/v1beta1.rs#L231
#[derive(Clone, PartialEq, prost::Message)]
pub struct ProtoMsgCreateDenom {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    /// subdenom can be up to 44 "alphanumeric" characters long.
    #[prost(string, tag = "2")]
    pub subdenom: ::prost::alloc::string::String,
}

impl TypeUrl for ProtoMsgCreateDenom {
    const TYPE_URL: &'static str = "/osmosis.tokenfactory.v1beta1.MsgCreateDenom";
}

// This struct is an exact copy of https://github.com/osmosis-labs/osmosis-rust/blob/5997b8797a3210df0b1ab017025506a7645ff961/packages/osmosis-std/src/types/osmosis/tokenfactory/v1beta1.rs#L231
#[derive(Clone, PartialEq, prost::Message)]
pub struct ProtoMsgMint {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub amount: ::core::option::Option<cosmos_sdk_proto::cosmos::base::v1beta1::Coin>,
    #[prost(string, tag = "3")]
    pub mint_to_address: ::prost::alloc::string::String,
}

impl TypeUrl for ProtoMsgMint {
    const TYPE_URL: &'static str = "/osmosis.tokenfactory.v1beta1.MsgMint";
}
