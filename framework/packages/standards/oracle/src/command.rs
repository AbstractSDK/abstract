use abstract_adapter_utils::Identify;

/// # OracleCommand
/// ensures Oracle adapters support the expected functionality.
///
/// Implements the usual Oracle operations.
pub trait OracleCommand: Identify {}
