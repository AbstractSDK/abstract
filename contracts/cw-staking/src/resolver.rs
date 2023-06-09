use cosmwasm_std::Env;
use abstract_adapter_utils::identity::decompose_platform_name;
use abstract_adapter_utils::identity::is_current_chain;
use abstract_staking_adapter_traits::StakingError;

use crate::StakingCommand;
use crate::contract::StakingResult;

use abstract_staking_adapter_traits::Identify;


use abstract_astroport_adapter::{ASTROPORT, staking::{Astroport}};
use abstract_osmosis_adapter::{OSMOSIS, staking::Osmosis};
use abstract_wyndex_adapter::staking::{WynDex, WYNDEX};

pub(crate) fn identify_provider(value: &str) -> Result<Box<dyn Identify>, StakingError> {
    match value {
        WYNDEX => Ok(Box::<WynDex>::default()),
        ASTROPORT => Ok(Box::<Astroport>::default()),
        OSMOSIS => Ok(Box::<Osmosis>::default()),
        _ => Err(StakingError::UnknownDex(value.to_string())),
    }
}

/// Given the provider name, return the local provider implementation
pub(crate) fn resolve_local_provider(name: &str) -> Result<Box<dyn StakingCommand>, StakingError> {
    match name {
        #[cfg(feature = "juno")]
        WYNDEX => Ok(Box::<WynDex>::default()),
        #[cfg(feature = "osmosis")]
        OSMOSIS => Ok(Box::<Osmosis>::default()),
        #[cfg(feature = "terra")]
        ASTROPORT => Ok(Box::<Astroport>::default()),
        _ => Err(StakingError::ForeignDex(name.to_owned())),
    }
}

/// Given a FULL provider nam (e.g. juno>wyndex), returns wether the request is local or over IBC
pub fn is_over_ibc(env: Env, platform_name: &str) -> StakingResult<(String, bool)>{
    let (chain_name, local_platform_name) = decompose_platform_name(platform_name);
    if !is_current_chain(env, &chain_name) {
        Ok((local_platform_name, true))
    } else {
        let platform_id = identify_provider(&local_platform_name)?;
        // We verify the adapter is available on the current chain
        if !platform_id.is_available_on(&chain_name){
            return Err(StakingError::UnknownDex(platform_name.to_string()))
        }
        Ok((local_platform_name, false))
    }
}
