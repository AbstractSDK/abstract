use error::AbstractXionError;

pub mod auth;
pub mod error;
pub mod queries;
pub mod state;
pub mod sudo;

// Copied types from https://github.com/burnt-labs/cosmos-rust/tree/d3b51db49b894f1c6b7836bb0a7b14f54f1dfb26/cosmos-sdk-proto/src/prost/xion
pub mod xion_proto;

pub type AbstractXionResult<R = cosmwasm_std::Response> = Result<R, AbstractXionError>;

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
