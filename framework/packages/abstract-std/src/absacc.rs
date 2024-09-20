//! https://github.com/burnt-labs/abstract-account/tree/4e376f2f399f17e50016a932d4e5af7336d952d7/cosmwasm/packages/absacc/src

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{AnyMsg, Binary};

/// Any contract must implement this sudo message (both variants) in order to
/// qualify as an abstract account.
#[cw_serde]
pub enum AccountSudoMsg {
    /// Called by the AnteHandler's BeforeTxDecorator before a tx is executed.
    BeforeTx {
        /// Messages the tx contains
        msgs: Vec<AnyMsg>,

        /// The tx serialized into binary format.
        ///
        /// If the tx authentication requires a signature, this is the bytes to
        /// be signed.
        tx_bytes: Binary,

        /// The credential to prove this tx is authenticated.
        ///
        /// This is taken from the tx's "signature" field, but in the case of
        /// AbstractAccounts, this is not necessarily a cryptographic signature.
        /// The contract is free to interpret this as any data type.
        cred_bytes: Option<Binary>,

        /// Whether the tx is being run in the simulation mode.
        simulate: bool,
    },

    /// Called by the PostHandler's AfterTxDecorator after the tx is executed.
    AfterTx {
        /// Whether the tx is being run in the simulation mode.
        simulate: bool,
    },
}

#[cw_serde]
pub enum Authenticator {
    Secp256K1 { pubkey: Binary },
    Ed25519 { pubkey: Binary },
    EthWallet { address: String },
    Jwt { aud: String, sub: String },
    Secp256R1 { pubkey: Binary },
    Passkey { url: String, passkey: Binary },
}
