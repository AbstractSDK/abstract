use abstract_oracle_standard::{Identify, OracleCommand, OracleError};

/// Any exchange should be identified by the adapter
/// This allows erroring the execution before sending any IBC message to another chain
/// This provides superior UX in case of an IBC execution
pub(crate) fn identify_oracle(value: &str) -> Result<Box<dyn Identify>, OracleError> {
    match value {
        abstract_pyth_adapter::PYTH => Ok(Box::<abstract_pyth_adapter::Pyth>::default()),
        _ => Err(OracleError::UnknownDex(value.to_owned())),
    }
}

pub(crate) fn resolve_oracle(value: &str) -> Result<Box<dyn OracleCommand>, OracleError> {
    match value {
        #[cfg(feature = "pyth")]
        abstract_pyth_adapter::PYTH => Ok(Box::<abstract_pyth_adapter::Pyth>::default()),
        _ => Err(OracleError::ForeignDex(value.to_owned())),
    }
}
