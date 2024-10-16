#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryValidateJwtRequest {
    #[prost(string, tag = "1")]
    pub aud: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub sub: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub sig_bytes: ::prost::alloc::string::String,
}
