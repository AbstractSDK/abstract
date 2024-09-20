pub mod auth;
pub mod sudo;

mod proto {

    use cosmos_sdk_proto::cosmos;
    use cosmwasm_std::{AnyMsg, CosmosMsg};
    use prost::{Message, Name};

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct MsgRegisterAccount {
        #[prost(string, tag = "1")]
        pub sender: String,

        #[prost(uint64, tag = "2")]
        pub code_id: u64,

        #[prost(bytes = "vec", tag = "3")]
        pub msg: Vec<u8>,

        #[prost(message, repeated, tag = "4")]
        pub funds: Vec<cosmos::base::v1beta1::Coin>,

        #[prost(bytes = "vec", tag = "5")]
        pub salt: Vec<u8>,
    }

    impl From<MsgRegisterAccount> for CosmosMsg {
        fn from(msg: MsgRegisterAccount) -> Self {
            let any_msg: AnyMsg = AnyMsg {
                type_url: MsgRegisterAccount::type_url(),
                value: msg.encode_to_vec().into(),
            };
            CosmosMsg::Any(any_msg)
        }
    }

    impl Name for MsgRegisterAccount {
        const NAME: &'static str = "MsgRegisterAccount";
        const PACKAGE: &'static str = "abstractaccount.v1";
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct MsgRegisterAccountResponse {
        #[prost(string, tag = "1")]
        pub address: String,

        #[prost(bytes = "vec", tag = "2")]
        pub data: Vec<u8>,
    }

    impl Name for MsgRegisterAccountResponse {
        const NAME: &'static str = "MsgRegisterAccountResponse";
        const PACKAGE: &'static str = "abstractaccount.v1";
    }
}

/// Any contract must implement this sudo message (both variants) in order to
/// qualify as an abstract account.
#[cosmwasm_schema::cw_serde]
pub enum AccountSudoMsg {
    /// Called by the AnteHandler's BeforeTxDecorator before a tx is executed.
    BeforeTx {
        /// Messages the tx contains
        msgs: Vec<cosmwasm_std::AnyMsg>,

        /// The tx serialized into binary format.
        ///
        /// If the tx authentication requires a signature, this is the bytes to
        /// be signed.
        tx_bytes: cosmwasm_std::Binary,

        /// The credential to prove this tx is authenticated.
        ///
        /// This is taken from the tx's "signature" field, but in the case of
        /// AbstractAccounts, this is not necessarily a cryptographic signature.
        /// The contract is free to interpret this as any data type.
        cred_bytes: Option<cosmwasm_std::Binary>,

        /// Whether the tx is being run in the simulation mode.
        simulate: bool,
    },

    /// Called by the PostHandler's AfterTxDecorator after the tx is executed.
    AfterTx {
        /// Whether the tx is being run in the simulation mode.
        simulate: bool,
    },
}
