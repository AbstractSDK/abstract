use abstract_oracle_standard::{OracleCommand, OracleError};

pub use abstract_pyth_adapter::PYTH;

pub(crate) fn resolve_oracle(value: &str) -> Result<Box<dyn OracleCommand>, OracleError> {
    match value {
        #[cfg(feature = "pyth")]
        abstract_pyth_adapter::PYTH => Ok(Box::<abstract_pyth_adapter::Pyth>::default()),
        _ => Err(OracleError::ForeignDex(value.to_owned())),
    }
}
