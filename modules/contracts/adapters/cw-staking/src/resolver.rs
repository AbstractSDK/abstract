use abstract_adapter_utils::identity::{
    decompose_platform_name, is_available_on, is_current_chain,
};
use abstract_staking_standard::{CwStakingCommand, CwStakingError, Identify};
use cosmwasm_std::Env;

use crate::contract::StakingResult;

/// Any cw-staking provider should be identified by the adapter
/// This allows erroring the execution before sending any IBC message to another chain
/// This provides superior UX in case of an IBC execution
pub(crate) fn identify_provider(value: &str) -> Result<Box<dyn Identify>, CwStakingError> {
    match value {
        // TODO: revive integrations
        // abstract_wyndex_adapter::WYNDEX => {
        //     Ok(Box::<abstract_wyndex_adapter::staking::WynDex>::default())
        // }
        // abstract_astroport_adapter::ASTROPORT => {
        //     Ok(Box::<abstract_astroport_adapter::staking::Astroport>::default())
        // }
        // abstract_kujira_adapter::staking::BOW => {
        //     Ok(Box::<abstract_kujira_adapter::staking::Bow>::default())
        // }
        abstract_osmosis_adapter::OSMOSIS => {
            Ok(Box::<abstract_osmosis_adapter::staking::Osmosis>::default())
        }
        abstract_astrovault_adapter::ASTROVAULT => {
            Ok(Box::<abstract_astrovault_adapter::staking::Astrovault>::default())
        }
        _ => Err(CwStakingError::UnknownDex(value.to_string())),
    }
}

/// Given the provider name, return the local provider implementation
pub(crate) fn resolve_local_provider(
    name: &str,
) -> Result<Box<dyn CwStakingCommand>, CwStakingError> {
    match name {
        #[cfg(feature = "wynd")]
        abstract_wyndex_adapter::WYNDEX => {
            Ok(Box::<abstract_wyndex_adapter::staking::WynDex>::default())
        }
        #[cfg(feature = "osmosis")]
        abstract_osmosis_adapter::OSMOSIS => {
            Ok(Box::<abstract_osmosis_adapter::staking::Osmosis>::default())
        }
        #[cfg(feature = "astroport")]
        abstract_astroport_adapter::ASTROPORT => {
            Ok(Box::<abstract_astroport_adapter::staking::Astroport>::default())
        }
        #[cfg(feature = "bow")]
        abstract_kujira_adapter::staking::BOW => {
            Ok(Box::<abstract_kujira_adapter::staking::Bow>::default())
        }
        #[cfg(feature = "astrovault")]
        abstract_astrovault_adapter::ASTROVAULT => {
            Ok(Box::<abstract_astrovault_adapter::staking::Astrovault>::default())
        }
        _ => Err(CwStakingError::ForeignDex(name.to_owned())),
    }
}

/// Given a FULL provider nam (e.g. juno>wyndex), returns wether the request is local or over IBC
pub fn is_over_ibc(env: &Env, platform_name: &str) -> StakingResult<(String, bool)> {
    let (chain_name, local_platform_name) = decompose_platform_name(platform_name);
    if chain_name.is_some() && !is_current_chain(env, &chain_name.clone().unwrap()) {
        Ok((local_platform_name, true))
    } else {
        let platform_id = identify_provider(&local_platform_name)?;
        // We verify the adapter is available on the current chain
        if !is_available_on(platform_id, env, chain_name.as_deref()) {
            return Err(CwStakingError::UnknownDex(platform_name.to_string()));
        }
        Ok((local_platform_name, false))
    }
}
