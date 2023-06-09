use abstract_dex_adapter_traits::{DexCommand, DexError, Identify};

use crate::exchanges::junoswap::{JunoSwap, JUNOSWAP};
use abstract_wyndex_adapter::dex::{WynDex, WYNDEX};
use crate::exchanges::terraswap::{Terraswap, TERRASWAP};
use abstract_astroport_adapter::{ ASTROPORT, dex::{Astroport}};
use abstract_osmosis_adapter::{OSMOSIS, dex::Osmosis};

pub(crate) fn identify_exchange(value: &str) -> Result<&'static dyn Identify, DexError> {
    match value {
        JUNOSWAP => Ok(&JunoSwap {}),
        WYNDEX => Ok(&WynDex {}),
        OSMOSIS => Ok(&Osmosis {
            local_proxy_addr: None,
        }),
        TERRASWAP => Ok(&Terraswap {}),
        ASTROPORT => Ok(&Astroport {}),
        _ => Err(DexError::UnknownDex(value.to_owned())),
    }
}

pub(crate) fn resolve_exchange(value: &str) -> Result<&'static dyn DexCommand, DexError> {
    match value {
        #[cfg(feature = "juno")]
        JUNOSWAP => Ok(&JunoSwap {}),
        #[cfg(feature = "juno")]
        WYNDEX => Ok(&WynDex {}),
        #[cfg(feature = "osmosis")]
        OSMOSIS => Ok(&Osmosis {
            local_proxy_addr: None,
        }),
        #[cfg(feature = "terra")]
        TERRASWAP => Ok(&Terraswap {}),
        #[cfg(feature = "terra")]
        ASTROPORT => Ok(&Astroport {}),
        _ => Err(DexError::ForeignDex(value.to_owned())),
    }
}
