use abstract_adapter_utils::identity::{
    decompose_platform_name, is_available_on, is_current_chain,
};
use abstract_oracle_standard::{Identify, OracleCommand, OracleError};
use cosmwasm_std::Env;

/// Any oracle should be identified by the adapter
/// This allows erroring the execution before sending any IBC message to another chain
/// This provides superior UX in case of an IBC execution
pub(crate) fn identify_oracle(value: &str) -> Result<Box<dyn Identify>, OracleError> {
    match value {
        _ => Err(OracleError::UnknownProvider(value.to_owned())),
    }
}

pub(crate) fn resolve_exchange(value: &str) -> Result<impl OracleCommand, OracleError> {
    match value {
        _ => Err(OracleError::ForeignOracle(value.to_owned())),
    }
}

/// Given a FULL provider name (e.g. juno>wyndex), returns whether the request is local or over IBC
pub fn is_over_ibc(env: Env, platform_name: &str) -> Result<(String, bool), OracleError> {
    let (chain_name, local_platform_name) = decompose_platform_name(platform_name);
    if chain_name.is_some() && !is_current_chain(env.clone(), &chain_name.clone().unwrap()) {
        Ok((local_platform_name, true))
    } else {
        let platform_id = identify_oracle(&local_platform_name)?;
        // We verify the adapter is available on the current chain
        if !is_available_on(platform_id, env, chain_name.as_deref()) {
            return Err(OracleError::UnknownProvider(platform_name.to_string()));
        }
        Ok((local_platform_name, false))
    }
}
