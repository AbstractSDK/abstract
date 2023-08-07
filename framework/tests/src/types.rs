#![allow(non_snake_case)] // This is used for prost::Message that raises warnings

pub mod ibc {

    use cosmrs::tendermint::block::Height;
    use cosmrs::{proto, ErrorReport};
    use cosmrs::{tx::Msg, Result};
    use proto::traits::TypeUrl;

    /// MsgTransfer defines a msg to transfer fungible tokens (i.e Coins) between
    /// ICS20 enabled chains. See ICS Spec here:
    /// <https://github.com/cosmos/ibc/tree/master/spec/app/ics-020-fungible-token-transfer#data-structures>
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, prost::Message)]
    pub struct ProtoMsgTransfer {
        /// the port on which the packet will be sent
        #[prost(string, tag = "1")]
        pub source_port: ::prost::alloc::string::String,
        /// the channel by which the packet will be sent
        #[prost(string, tag = "2")]
        pub source_channel: ::prost::alloc::string::String,
        /// the tokens to be transferred
        #[prost(message, optional, tag = "3")]
        pub token: ::core::option::Option<proto::cosmos::base::v1beta1::Coin>,
        /// the sender address
        #[prost(string, tag = "4")]
        pub sender: ::prost::alloc::string::String,
        /// the recipient address on the destination chain
        #[prost(string, tag = "5")]
        pub receiver: ::prost::alloc::string::String,
        /// Timeout height relative to the current block height.
        /// The timeout is disabled when set to 0.
        #[prost(message, optional, tag = "6")]
        pub timeout_height: ::core::option::Option<proto::ibc::core::client::v1::Height>,
        /// Timeout timestamp in absolute nanoseconds since unix epoch.
        /// The timeout is disabled when set to 0.
        #[prost(uint64, tag = "7")]
        pub timeout_timestamp: u64,
        /// Optional memo
        /// whole reason we are copying this from its original (proto::ibc::applications::transfer::v1::MsgTransfer)
        #[prost(string, optional, tag = "8")]
        pub memo: ::core::option::Option<::prost::alloc::string::String>,
    }

    impl TypeUrl for ProtoMsgTransfer {
        const TYPE_URL: &'static str = "/ibc.applications.transfer.v1.MsgTransfer";
    }

    /// MsgSend represents a message to send coins from one account to another.
    #[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
    pub struct MsgTransfer {
        /// Sender's address.
        pub source_port: String,
        pub source_channel: String,
        pub token: Option<cosmrs::Coin>,
        pub sender: cosmrs::AccountId,
        pub receiver: cosmrs::AccountId,
        pub timeout_height: Option<Height>,
        pub timeout_revision: Option<u64>,
        pub timeout_timestamp: u64,
        pub memo: Option<String>,
    }

    impl Msg for MsgTransfer {
        type Proto = ProtoMsgTransfer;
    }

    impl TryFrom<ProtoMsgTransfer> for MsgTransfer {
        type Error = ErrorReport;

        fn try_from(proto: ProtoMsgTransfer) -> Result<MsgTransfer> {
            MsgTransfer::try_from(&proto)
        }
    }

    impl TryFrom<&ProtoMsgTransfer> for MsgTransfer {
        type Error = ErrorReport;

        fn try_from(proto: &ProtoMsgTransfer) -> Result<MsgTransfer> {
            Ok(MsgTransfer {
                source_port: proto.source_port.parse()?,
                source_channel: proto.source_channel.parse()?,
                token: proto.token.clone().map(TryFrom::try_from).transpose()?,
                sender: proto.sender.parse()?,
                receiver: proto.receiver.parse()?,
                timeout_height: proto
                    .timeout_height
                    .clone()
                    .map(|h| h.revision_height.try_into())
                    .transpose()?,
                timeout_revision: proto.timeout_height.clone().map(|h| h.revision_number),
                timeout_timestamp: proto.timeout_timestamp,
                memo: proto.memo.clone(),
            })
        }
    }

    impl From<MsgTransfer> for ProtoMsgTransfer {
        fn from(coin: MsgTransfer) -> ProtoMsgTransfer {
            ProtoMsgTransfer::from(&coin)
        }
    }

    impl From<&MsgTransfer> for ProtoMsgTransfer {
        fn from(msg: &MsgTransfer) -> ProtoMsgTransfer {
            ProtoMsgTransfer {
                source_port: msg.source_port.clone(),
                source_channel: msg.source_channel.clone(),
                token: msg.token.clone().map(Into::into),
                sender: msg.sender.to_string(),
                receiver: msg.receiver.to_string(),
                timeout_height: msg
                    .timeout_height
                    .map(|h| proto::ibc::core::client::v1::Height {
                        revision_number: msg.timeout_revision.unwrap(),
                        revision_height: h.value(),
                    }),
                timeout_timestamp: msg.timeout_timestamp,
                memo: msg.memo.clone(),
            }
        }
    }
}

pub mod token_factory {

    use cosmos_sdk_proto::{cosmos::base::v1beta1::Coin, traits::TypeUrl};
    use cosmrs::{tx::Msg, AccountId, ErrorReport, Result};

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

    /// MsgCreateDenom represents a message to send coins from one account to another.
    #[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
    pub struct MsgCreateDenom {
        /// Sender's address.
        pub sender: AccountId,

        /// Subdenom name
        pub subdenom: String,
    }

    impl TryFrom<ProtoMsgCreateDenom> for MsgCreateDenom {
        type Error = ErrorReport;

        fn try_from(proto: ProtoMsgCreateDenom) -> Result<MsgCreateDenom> {
            MsgCreateDenom::try_from(&proto)
        }
    }

    impl TryFrom<&ProtoMsgCreateDenom> for MsgCreateDenom {
        type Error = ErrorReport;

        fn try_from(proto: &ProtoMsgCreateDenom) -> Result<MsgCreateDenom> {
            Ok(MsgCreateDenom {
                sender: proto.sender.parse()?,
                subdenom: proto.subdenom.parse()?,
            })
        }
    }

    impl From<MsgCreateDenom> for ProtoMsgCreateDenom {
        fn from(coin: MsgCreateDenom) -> ProtoMsgCreateDenom {
            ProtoMsgCreateDenom::from(&coin)
        }
    }

    impl From<&MsgCreateDenom> for ProtoMsgCreateDenom {
        fn from(msg: &MsgCreateDenom) -> ProtoMsgCreateDenom {
            ProtoMsgCreateDenom {
                sender: msg.sender.to_string(),
                subdenom: msg.subdenom.to_string(),
            }
        }
    }

    impl Msg for MsgCreateDenom {
        type Proto = ProtoMsgCreateDenom;
    }

    // This struct is an exact copy of https://github.com/osmosis-labs/osmosis-rust/blob/5997b8797a3210df0b1ab017025506a7645ff961/packages/osmosis-std/src/types/osmosis/tokenfactory/v1beta1.rs#L231
    #[derive(Clone, PartialEq, prost::Message)]
    pub struct ProtoMsgMint {
        #[prost(string, tag = "1")]
        pub sender: ::prost::alloc::string::String,
        #[prost(message, optional, tag = "2")]
        pub amount: ::core::option::Option<Coin>,
        #[prost(string, tag = "3")]
        pub mint_to_address: ::prost::alloc::string::String,
    }

    impl TypeUrl for ProtoMsgMint {
        const TYPE_URL: &'static str = "/osmosis.tokenfactory.v1beta1.MsgMint";
    }

    /// MsgMint represents a message to send coins from one account to another.
    #[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
    pub struct MsgMint {
        /// Sender's address.
        pub sender: AccountId,

        /// Amount to mint
        pub amount: Option<cosmrs::Coin>,

        /// Recipient
        pub mint_to_address: AccountId,
    }

    impl TryFrom<ProtoMsgMint> for MsgMint {
        type Error = ErrorReport;

        fn try_from(proto: ProtoMsgMint) -> Result<MsgMint> {
            MsgMint::try_from(&proto)
        }
    }

    impl TryFrom<&ProtoMsgMint> for MsgMint {
        type Error = ErrorReport;

        fn try_from(proto: &ProtoMsgMint) -> Result<MsgMint> {
            Ok(MsgMint {
                sender: proto.sender.parse()?,
                amount: proto.amount.clone().map(TryFrom::try_from).transpose()?,
                mint_to_address: proto.mint_to_address.parse()?,
            })
        }
    }

    impl From<MsgMint> for ProtoMsgMint {
        fn from(coin: MsgMint) -> ProtoMsgMint {
            ProtoMsgMint::from(&coin)
        }
    }

    impl From<&MsgMint> for ProtoMsgMint {
        fn from(msg: &MsgMint) -> ProtoMsgMint {
            ProtoMsgMint {
                sender: msg.sender.to_string(),
                amount: msg.amount.clone().map(Into::into),
                mint_to_address: msg.mint_to_address.to_string(),
            }
        }
    }

    impl Msg for MsgMint {
        type Proto = ProtoMsgMint;
    }
}
