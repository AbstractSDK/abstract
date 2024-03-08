use abstract_adapter_utils::identity::{
    decompose_platform_name, is_available_on, is_current_chain,
};
use abstract_moneymarket_standard::{Identify, MoneymarketCommand, MoneymarketError};
use cosmwasm_std::Env;

/// Any exchange should be identified by the adapter
/// This allows erroring the execution before sending any IBC message to another chain
/// This provides superior UX in case of an IBC execution
pub(crate) fn identify_moneymarket(value: &str) -> Result<Box<dyn Identify>, MoneymarketError> {
    match value {
        abstract_kujira_adapter::KUJIRA => {
            Ok(Box::<abstract_kujira_adapter::dex::Kujira>::default())
        }
        abstract_mars_adapter::MARS => {
            Ok(Box::<abstract_mars_adapter::moneymarket::Mars>::default())
        }
        _ => Err(MoneymarketError::UnknownMoneymarket(value.to_owned())),
    }
}

pub(crate) fn resolve_moneymarket(
    value: &str,
) -> Result<Box<dyn MoneymarketCommand>, MoneymarketError> {
    match value {
        #[cfg(feature = "ghost")]
        abstract_kujira_adapter::KUJIRA => {
            Ok(Box::<abstract_kujira_adapter::dex::Kujira>::default())
        }
        #[cfg(feature = "mars")]
        abstract_mars_adapter::MARS => {
            Ok(Box::<abstract_mars_adapter::moneymarket::Mars>::default())
        }
        _ => Err(MoneymarketError::ForeignMoneymarket(value.to_owned())),
    }
}

/// Given a FULL provider nam (e.g. juno>wyndex), returns whether the request is local or over IBC
pub fn is_over_ibc(env: Env, platform_name: &str) -> Result<(String, bool), MoneymarketError> {
    let (chain_name, local_platform_name) = decompose_platform_name(platform_name);
    if chain_name.is_some() && !is_current_chain(env.clone(), &chain_name.clone().unwrap()) {
        Ok((local_platform_name, true))
    } else {
        let platform_id = identify_moneymarket(&local_platform_name)?;
        // We verify the adapter is available on the current chain
        if !is_available_on(platform_id, env, chain_name.as_deref()) {
            return Err(MoneymarketError::UnknownMoneymarket(
                platform_name.to_string(),
            ));
        }
        Ok((local_platform_name, false))
    }
}
