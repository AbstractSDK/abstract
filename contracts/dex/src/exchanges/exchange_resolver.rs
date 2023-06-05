use abstract_dex_adapter_traits::{Identify, DexCommand, DexError};

// Supported exchanges on Juno
#[cfg(feature = "juno")]
pub use crate::exchanges::junoswap::{JunoSwap, JUNOSWAP};

#[cfg(feature = "juno")]
pub use abstract_wyndex_dex_adapter::{WynDex, WYNDEX};

#[cfg(feature = "terra")]
pub use crate::exchanges::terraswap::{Terraswap, TERRASWAP};

#[cfg(feature = "terra")]
pub use abstract_astroport_adapter::{AstroportDex, ASTROPORT_DEX};

#[cfg(any(feature = "juno", feature = "osmosis"))]
pub use abstract_osmosis_dex_adapter::{Osmosis, OSMOSIS};

pub(crate) fn identify_exchange(value: &str) -> Result<&'static dyn Identify, DexError> {
    match value {
        #[cfg(feature = "juno")]
        JUNOSWAP => Ok(&JunoSwap {}),
        #[cfg(feature = "juno")]
        WYNDEX => Ok(&WynDex {}),
        #[cfg(feature = "juno")]
        OSMOSIS => Ok(&Osmosis {
            local_proxy_addr: None,
        }),
        #[cfg(feature = "terra")]
        TERRASWAP => Ok(&Terraswap {}),
        #[cfg(feature = "terra")]
        ASTROPORT_DEX => Ok(&AstroportDex {}),
        _ => Err(DexError::UnknownDex(value.to_owned())),
    }
}

pub(crate) fn resolve_exchange(value: &str) -> Result<&'static dyn DexCommand, DexError> {
    match value {
        #[cfg(feature = "juno")]
        JUNOSWAP => Ok(&JunoSwap {}),
        #[cfg(feature = "juno")]
        WYNDEX => Ok(&WynDex {}),
        // #[cfg(feature = "osmosis")]
        // OSMOSIS => Ok(&Osmosis {
        //     local_proxy_addr: None,
        // }),
        #[cfg(feature = "terra")]
        TERRASWAP => Ok(&Terraswap {}),
        #[cfg(feature = "terra")]
        ASTROPORT_DEX => Ok(&AstroportDex {}),
        _ => Err(DexError::ForeignDex(value.to_owned())),
    }
}
