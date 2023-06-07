use crate::StakingCommand;
#[cfg(any(feature = "terra", feature = "juno"))]
use abstract_staking_adapter_traits::Identify;

use abstract_staking_adapter_traits::StakingError;
use cosmwasm_std::{StdError, StdResult};

#[cfg(feature = "terra")]
use abstract_astroport_adapter::staking::{Astroport, ASTROPORT};

#[cfg(any(feature = "juno", feature = "osmosis"))]
pub use abstract_osmosis_adapter::staking::{Osmosis, OSMOSIS};

#[cfg(feature = "juno")]
use abstract_wyndex_adapter::staking::{WynDex, WYNDEX};

pub(crate) fn is_over_ibc(provider: &str) -> StdResult<bool> {
    match provider {
        #[cfg(feature = "juno")]
        WYNDEX => Ok(WynDex::default().over_ibc()),
        #[cfg(feature = "terra")]
        ASTROPORT => Ok(Astroport::default().over_ibc()),
        #[cfg(feature = "juno")]
        OSMOSIS => Ok(Osmosis::default().over_ibc()),
        _ => Err(StdError::generic_err(format!(
            "Unknown provider {provider}"
        ))),
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
