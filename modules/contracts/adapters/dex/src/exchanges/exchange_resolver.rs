use abstract_adapter_utils::identity::decompose_platform_name;
use abstract_adapter_utils::identity::is_available_on;
use abstract_adapter_utils::identity::is_current_chain;
use abstract_dex_adapter_traits::{DexCommand, DexError, Identify};
use cosmwasm_std::Env;

/// Any exchange should be identified by the adapter
/// This allows erroring the execution before sending any IBC message to another chain
/// This provides superior UX in case of an IBC execution
pub(crate) fn identify_exchange(value: &str) -> Result<Box<dyn Identify>, DexError> {
    match value {
        crate::exchanges::junoswap::JUNOSWAP => {
            Ok(Box::<crate::exchanges::junoswap::JunoSwap>::default())
        }
        abstract_wyndex_adapter::WYNDEX => {
            Ok(Box::<abstract_wyndex_adapter::dex::WynDex>::default())
        }
        abstract_osmosis_adapter::OSMOSIS => {
            Ok(Box::<abstract_osmosis_adapter::dex::Osmosis>::default())
        }
        crate::exchanges::terraswap::TERRASWAP => {
            Ok(Box::<crate::exchanges::terraswap::Terraswap>::default())
        }
        abstract_astroport_adapter::ASTROPORT => {
            Ok(Box::<abstract_astroport_adapter::dex::Astroport>::default())
        }
        abstract_kujira_adapter::KUJIRA => {
            Ok(Box::<abstract_kujira_adapter::dex::Kujira>::default())
        }
        _ => Err(DexError::UnknownDex(value.to_owned())),
    }
}

pub(crate) fn resolve_exchange(value: &str) -> Result<&'static dyn DexCommand, DexError> {
    match value {
        #[cfg(feature = "juno")]
        crate::exchanges::junoswap::JUNOSWAP => Ok(&crate::exchanges::junoswap::JunoSwap {}),
        #[cfg(feature = "juno")]
        abstract_wyndex_adapter::WYNDEX => Ok(&abstract_wyndex_adapter::dex::WynDex {}),
        #[cfg(feature = "osmosis")]
        abstract_osmosis_adapter::OSMOSIS => Ok(&abstract_osmosis_adapter::dex::Osmosis {
            local_proxy_addr: None,
        }),
        #[cfg(feature = "terra")]
        crate::exchanges::terraswap::TERRASWAP => Ok(&crate::exchanges::terraswap::Terraswap {}),
        #[cfg(any(feature = "terra", feature = "neutron"))]
        abstract_astroport_adapter::ASTROPORT => Ok(&abstract_astroport_adapter::dex::Astroport {}),
        #[cfg(feature = "kujira")]
        abstract_kujira_adapter::KUJIRA => Ok(&abstract_kujira_adapter::dex::Kujira {}),
        _ => Err(DexError::ForeignDex(value.to_owned())),
    }
}

/// Given a FULL provider nam (e.g. juno>wyndex), returns whether the request is local or over IBC
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
