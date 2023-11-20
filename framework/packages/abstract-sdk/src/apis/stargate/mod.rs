use cosmos_sdk_proto::traits::{Message, Name};
use prost_types::Any;

mod feegrant_impl;
pub mod feegrant {
    pub use super::feegrant_impl::*;
}

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
