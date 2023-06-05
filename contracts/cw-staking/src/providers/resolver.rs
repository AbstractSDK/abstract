use abstract_staking_adapter_traits::StakingError;
use crate::StakingCommand;
use cosmwasm_std::{StdError, StdResult};

#[cfg(feature = "terra")]
use abstract_astroport_adapter::{AstroportStaking, ASTROPORT_STAKING};

#[cfg(any(feature = "juno", feature = "osmosis"))]
pub use crate::providers::osmosis::{Osmosis, OSMOSIS};

#[cfg(feature = "juno")]
pub use crate::providers::{
    junoswap::{JunoSwap, JUNOSWAP},
};
#[cfg(feature = "juno")]
use abstract_wyndex_adapter::staking::{WynDex, WYNDEX};

pub(crate) fn is_over_ibc(provider: &str) -> StdResult<bool> {
    match provider {
        #[cfg(feature = "juno")]
        JUNOSWAP => Ok(false),
        #[cfg(feature = "juno")]
        WYNDEX => Ok(false),
        #[cfg(feature = "terra")]
        ASTROPORT_STAKING => Ok(false),
        #[cfg(feature = "juno")]
        OSMOSIS => Ok(true),
        _ => Err(StdError::generic_err(format!(
            "Unknown provider {provider}"
        ))),
    }
}

/// Given the provider name, return the local provider implementation
pub(crate) fn resolve_local_provider(name: &str) -> Result<Box<dyn StakingCommand>, StakingError> {
    match name {
        #[cfg(feature = "juno")]
        JUNOSWAP => Ok(Box::<JunoSwap>::default()),
        #[cfg(feature = "juno")]
        WYNDEX => Ok(Box::<WynDex>::default()),
        #[cfg(feature = "osmosis")]
        OSMOSIS => Ok(Box::new(Osmosis::default())),
        #[cfg(feature = "terra")]
        ASTROPORT_STAKING => Ok(Box::<AstroportStaking>::default()),
        _ => Err(StakingError::ForeignDex(name.to_owned())),
    }
}
