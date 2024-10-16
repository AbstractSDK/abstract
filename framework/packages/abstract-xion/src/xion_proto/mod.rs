pub mod jwk;

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryWebAuthNVerifyAuthenticateRequest {
    #[prost(string, tag = "1")]
    pub addr: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub challenge: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub rp: ::prost::alloc::string::String,
    #[prost(bytes = "vec", tag = "4")]
    pub credential: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "5")]
    pub data: ::prost::alloc::vec::Vec<u8>,
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryWebAuthNVerifyRegisterRequest {
    #[prost(string, tag = "1")]
    pub addr: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub challenge: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub rp: ::prost::alloc::string::String,
    #[prost(bytes = "vec", tag = "4")]
    pub data: ::prost::alloc::vec::Vec<u8>,
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryWebAuthNVerifyRegisterResponse {
    #[prost(bytes = "vec", tag = "1")]
    pub credential: ::prost::alloc::vec::Vec<u8>,
}
