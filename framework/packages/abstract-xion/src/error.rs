use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum AbstractXionError {
    #[error("{0}")]
    Std(#[from] cosmwasm_std::StdError),

    #[error(transparent)]
    DecodeError(#[from] prost::DecodeError),

    #[error(transparent)]
    Verification(#[from] cosmwasm_std::VerificationError),

    #[error(transparent)]
    FromHex(#[from] hex::FromHexError),

    #[error(transparent)]
    Bech32(#[from] bech32::Error),

    #[error(transparent)]
    Base64Decode(#[from] base64::DecodeError),

    #[error(transparent)]
    Rsa(#[from] rsa::Error),

    #[error(transparent)]
    P256EllipticCurve(#[from] p256::elliptic_curve::Error),

    #[error("{0}")]
    P256EcdsaCurve(String),

    #[error(transparent)]
    RecoverPubkey(#[from] cosmwasm_std::RecoverPubkeyError),

    #[error("The pubkey recovered from the signature does not match")]
    RecoveredPubkeyMismatch {},

    #[error("Signature is empty")]
    EmptySignature {},

    #[error("Short signature")]
    ShortSignature {},

    #[error("Signature is invalid")]
    InvalidSignature {},

    #[error("Signature is invalid. expected: {expected}, received {received}")]
    InvalidSignatureDetail { expected: String, received: String },

    #[error("Recovery id can only be one of 0, 1, 27, 28")]
    InvalidRecoveryId {},

    #[error("Invalid token")]
    InvalidToken {},

    #[error("url parse error: {url}")]
    URLParse { url: String },

    #[error("cannot override existing authenticator at index {index}")]
    OverridingIndex { index: u8 },

    #[error("cannot delete the last authenticator")]
    MinimumAuthenticatorCount {},

    #[error("Authenticator id should be in range from 0 to 127")]
    TooBigAuthId {},

    #[error(transparent)]
    FromUTF8(#[from] std::string::FromUtf8Error),
}

impl From<p256::ecdsa::Error> for AbstractXionError {
    fn from(value: p256::ecdsa::Error) -> Self {
        Self::P256EcdsaCurve(value.to_string())
    }
}
