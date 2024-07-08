use abstract_adapter_utils::identity::{
    decompose_platform_name, is_available_on, is_current_chain,
};
use abstract_dex_standard::{DexCommand, DexError, Identify};
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
        abstract_astroport_adapter::ASTROPORT => {
            Ok(Box::<abstract_astroport_adapter::dex::Astroport>::default())
        }
        abstract_kujira_adapter::dex::FIN => {
            Ok(Box::<abstract_kujira_adapter::dex::Fin>::default())
        }
        abstract_astrovault_adapter::ASTROVAULT => {
            Ok(Box::<abstract_astrovault_adapter::dex::Astrovault>::default())
        }
        _ => Err(DexError::UnknownDex(value.to_owned())),
    }
}

pub(crate) fn resolve_exchange(value: &str) -> Result<Box<dyn DexCommand>, DexError> {
    match value {
        #[cfg(feature = "wynd")]
        crate::exchanges::junoswap::JUNOSWAP => {
            Ok(Box::<crate::exchanges::junoswap::JunoSwap>::default())
        }
        #[cfg(feature = "wynd")]
        abstract_wyndex_adapter::WYNDEX => {
            Ok(Box::<abstract_wyndex_adapter::dex::WynDex>::default())
        }
        #[cfg(feature = "osmosis")]
        abstract_osmosis_adapter::OSMOSIS => {
            Ok(Box::<abstract_osmosis_adapter::dex::Osmosis>::default())
        }
        #[cfg(feature = "astroport")]
        abstract_astroport_adapter::ASTROPORT => {
            Ok(Box::<abstract_astroport_adapter::dex::Astroport>::default())
        }
        #[cfg(feature = "fin")]
        abstract_kujira_adapter::dex::FIN => {
            Ok(Box::<abstract_kujira_adapter::dex::Fin>::default())
        }
        #[cfg(feature = "astrovault")]
        abstract_astrovault_adapter::ASTROVAULT => {
            Ok(Box::<abstract_astrovault_adapter::dex::Astrovault>::default())
        }
        _ => Err(DexError::ForeignDex(value.to_owned())),
    }
}

/// Given a FULL provider nam (e.g. juno>wyndex), returns whether the request is local or over IBC
pub fn is_over_ibc(env: &Env, platform_name: &str) -> Result<(String, bool), DexError> {
    let (chain_name, local_platform_name) = decompose_platform_name(platform_name);
    if chain_name.is_some() && !is_current_chain(env, &chain_name.clone().unwrap()) {
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
