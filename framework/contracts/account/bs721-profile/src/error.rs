use cosmwasm_std::StdError;
use cw_controllers::AdminError;
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Payment(#[from] PaymentError),

    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("{0}")]
    Base(#[from] bs721_base::ContractError),

    #[error("NameNotFound")]
    NameNotFound {},

    #[error("AddressAlreadyMapped")]
    AddressAlreadyMapped {},

    #[error("RecordNameAlreadyExists")]
    RecordNameAlreadyExists {},

    #[error("RecordNameEmpty")]
    RecordNameEmpty {},

    #[error("RecordNameTooLong")]
    RecordNameTooLong {},

    #[error("RecordValueTooLong")]
    RecordValueTooLong {},

    #[error("RecordValueEmpty")]
    RecordValueEmpty {},

    #[error("UnauthorizedVerification")]
    UnauthorizedVerification {},

    #[error("Invalid Metadata")]
    InvalidMetadata {},

    #[error("Unauthorized: Not contract creator or admin")]
    UnauthorizedCreatorOrAdmin {},

    #[error("TooManyRecords max: {max}")]
    TooManyRecords { max: u32 },

    #[error("NotImplemented")]
    NotImplemented {},
}
