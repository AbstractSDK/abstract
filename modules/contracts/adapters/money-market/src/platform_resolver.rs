use abstract_adapter_utils::identity::{
    decompose_platform_name, is_available_on, is_current_chain,
};
use abstract_money_market_standard::{Identify, MoneyMarketCommand, MoneyMarketError};
use cosmwasm_std::Env;

/// Any exchange should be identified by the adapter
/// This allows erroring the execution before sending any IBC message to another chain
/// This provides superior UX in case of an IBC execution
pub(crate) fn identify_money_market(value: &str) -> Result<Box<dyn Identify>, MoneyMarketError> {
    match value {
        abstract_kujira_adapter::KUJIRA => {
            Ok(Box::<abstract_kujira_adapter::money_market::Ghost>::default())
        }
        abstract_mars_adapter::MARS => {
            Ok(Box::<abstract_mars_adapter::money_market::Mars>::default())
        }
        abstract_cavern_adapter::CAVERN => {
            Ok(Box::<abstract_cavern_adapter::money_market::Cavern>::default())
        }
        _ => Err(MoneyMarketError::UnknownMoneyMarket(value.to_owned())),
    }
}

pub(crate) fn resolve_money_market(
    value: &str,
) -> Result<Box<dyn MoneyMarketCommand>, MoneyMarketError> {
    match value {
        #[cfg(feature = "ghost")]
        abstract_kujira_adapter::KUJIRA => {
            Ok(Box::<abstract_kujira_adapter::money_market::Ghost>::default())
        }
        #[cfg(feature = "mars")]
        abstract_mars_adapter::MARS => {
            Ok(Box::<abstract_mars_adapter::money_market::Mars>::default())
        }
        #[cfg(feature = "cavern")]
        abstract_cavern_adapter::CAVERN => {
            Ok(Box::<abstract_cavern_adapter::money_market::Cavern>::default())
        }
        _ => Err(MoneyMarketError::ForeignMoneyMarket(value.to_owned())),
    }
}

/// Given a FULL provider nam (e.g. juno>wyndex), returns whether the request is local or over IBC
pub fn is_over_ibc(env: Env, platform_name: &str) -> Result<(String, bool), MoneyMarketError> {
    let (chain_name, local_platform_name) = decompose_platform_name(platform_name);
    if chain_name.is_some() && !is_current_chain(env.clone(), &chain_name.clone().unwrap()) {
        Ok((local_platform_name, true))
    } else {
        let platform_id = identify_money_market(&local_platform_name)?;
        // We verify the adapter is available on the current chain
        if !is_available_on(platform_id, env, chain_name.as_deref()) {
            return Err(MoneyMarketError::UnknownMoneyMarket(
                platform_name.to_string(),
            ));
        }
        Ok((local_platform_name, false))
    }
}
