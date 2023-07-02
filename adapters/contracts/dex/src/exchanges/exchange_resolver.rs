use abstract_adapter_utils::identity::decompose_platform_name;
use abstract_adapter_utils::identity::is_available_on;
use abstract_adapter_utils::identity::is_current_chain;
use abstract_dex_adapter_traits::{DexCommand, DexError, Identify};
use cosmwasm_std::Env;

use crate::exchanges::junoswap::{JunoSwap, JUNOSWAP};
use crate::exchanges::kujira::{Kujira, KUJIRA};
use crate::exchanges::terraswap::{Terraswap, TERRASWAP};
use abstract_astroport_adapter::{dex::Astroport, ASTROPORT};
use abstract_osmosis_adapter::{dex::Osmosis, OSMOSIS};
use abstract_wyndex_adapter::{dex::WynDex, WYNDEX};

pub(crate) fn identify_exchange(value: &str) -> Result<Box<dyn Identify>, DexError> {
    match value {
        JUNOSWAP => Ok(Box::<JunoSwap>::default()),
        WYNDEX => Ok(Box::<WynDex>::default()),
        OSMOSIS => Ok(Box::<Osmosis>::default()),
        TERRASWAP => Ok(Box::<Terraswap>::default()),
        ASTROPORT => Ok(Box::<Astroport>::default()),
        KUJIRA => Ok(Box::<Kujira>::default()),
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
        #[cfg(feature = "kujira")]
        KUJIRA => Ok(&Kujira {}),
        _ => Err(DexError::ForeignDex(value.to_owned())),
    }
}

/// Given a FULL provider nam (e.g. juno>wyndex), returns wether the request is local or over IBC
pub fn is_over_ibc(env: Env, platform_name: &str) -> Result<(String, bool), DexError> {
    let (chain_name, local_platform_name) = decompose_platform_name(platform_name);
    if chain_name.is_some() && !is_current_chain(env.clone(), &chain_name.clone().unwrap()) {
        Ok((local_platform_name, true))
    } else {
        let platform_id = identify_exchange(&local_platform_name)?;
        // We verify the adapter is available on the current chain
        if !is_available_on(platform_id, env, chain_name.as_deref()) {
            return Err(DexError::UnknownDex(platform_name.to_string()));
        }
        Ok((local_platform_name, false))
    }
}
