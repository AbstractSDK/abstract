use abstract_adapter_utils::identity::decompose_platform_name;
use abstract_adapter_utils::identity::is_available_on;
use abstract_adapter_utils::identity::is_current_chain;
use abstract_staking_adapter_traits::{CwStakingCommand, CwStakingError};
use cosmwasm_std::Env;

use crate::contract::StakingResult;

use abstract_staking_adapter_traits::Identify;

use crate::providers::kujira::{Kujira, KUJIRA};
use abstract_astroport_adapter::{staking::Astroport, ASTROPORT};
use abstract_osmosis_adapter::{staking::Osmosis, OSMOSIS};
use abstract_wyndex_adapter::staking::{WynDex, WYNDEX};

pub(crate) fn identify_provider(value: &str) -> Result<Box<dyn Identify>, CwStakingError> {
    match value {
        WYNDEX => Ok(Box::<WynDex>::default()),
        ASTROPORT => Ok(Box::<Astroport>::default()),
        OSMOSIS => Ok(Box::<Osmosis>::default()),
        KUJIRA => Ok(Box::<Kujira>::default()),
        _ => Err(CwStakingError::UnknownDex(value.to_string())),
    }
}

/// Given the provider name, return the local provider implementation
pub(crate) fn resolve_local_provider(
    name: &str,
) -> Result<Box<dyn CwStakingCommand>, CwStakingError> {
    match name {
        #[cfg(feature = "juno")]
        WYNDEX => Ok(Box::<WynDex>::default()),
        #[cfg(feature = "osmosis")]
        OSMOSIS => Ok(Box::<Osmosis>::default()),
        #[cfg(feature = "terra")]
        ASTROPORT => Ok(Box::<Astroport>::default()),
        #[cfg(feature = "kujira")]
        KUJIRA => Ok(Box::<Kujira>::default()),
        _ => Err(CwStakingError::ForeignDex(name.to_owned())),
    }
}

/// Given a FULL provider nam (e.g. juno>wyndex), returns wether the request is local or over IBC
pub fn is_over_ibc(env: Env, platform_name: &str) -> StakingResult<(String, bool)> {
    let (chain_name, local_platform_name) = decompose_platform_name(platform_name);
    if chain_name.is_some() && !is_current_chain(env.clone(), &chain_name.clone().unwrap()) {
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
